use crate::{Error, Result, Users};

use chrono::DateTime;
use goji::{Board, Credentials, EditIssue, Issue, Jira, SearchOptions, Sprint};
use lazy_static::lazy_static;
use prettytable::{cell, format, row, Table};
use serde::Serialize;

use std::collections::BTreeMap;

lazy_static! {
    static ref DEFAULT_TABLE_FORMAT: format::TableFormat = format::FormatBuilder::new()
        .column_separator('│')
        .separators(
            &[format::LinePosition::Title],
            format::LineSeparator::new('─', '┼', '├', '┤'),
        )
        .padding(1, 1)
        .build();
}

pub struct Client {
    jira: Jira,
    width: Option<f32>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimeTracking {
    pub original_estimate: u64,
    pub remaining_estimate: u64,
}

impl Client {
    pub fn new(options: &clap::ArgMatches) -> Result<Self> {
        let (organization, user, token) = (
            options
                .value_of("organization")
                .ok_or(Error::Config("organization".to_owned()))?,
            options
                .value_of("user")
                .ok_or(Error::Config("user".to_owned()))?,
            options
                .value_of("token")
                .ok_or(Error::Config("token".to_owned()))?,
        );

        let width = match term_size::dimensions() {
            None => None,
            Some((term_width, _)) => match term_width {
                term_width if term_width < 188 => Some(80.0),
                _ => Some(term_width as f32 - 108.0),
            },
        };

        Ok(Self {
            jira: Jira::new(
                format!("https://{}.atlassian.net", organization),
                Credentials::Basic(user.to_owned(), token.to_owned()),
            )?,
            width,
        })
    }

    pub fn boards(&self) -> Result<()> {
        let mut boards: Vec<Board> = self.jira.boards().iter(&Default::default())?.collect();
        boards.sort_by(|a, b| a.id.cmp(&b.id));

        let mut table = Table::new();
        table.set_format(*DEFAULT_TABLE_FORMAT);
        table.set_titles(row!["ID", "Name", "Type"]);

        for board in boards {
            table.add_row(row![board.id, board.name, board.type_name]);
        }

        Ok(self.print_table(table, "No boards were found which you have access to"))
    }

    pub fn sprints(&self, options: &clap::ArgMatches) -> Result<()> {
        let (board_id, all, active, future) = (
            options
                .value_of("board")
                .ok_or(Error::Config("board".to_owned()))?,
            options.is_present("all"),
            options.is_present("active"),
            options.is_present("future"),
        );

        let board = self.jira.boards().get(board_id)?;
        let state = match (all, active, future) {
            (true, false, false) => "",
            (false, true, false) => "active",
            (false, false, true) => "future",
            (_, _, _) => "active,future",
        };

        let search = SearchOptions::builder().state(state).build();
        let mut sprints: Vec<Sprint> = self.jira.sprints().iter(&board, &search)?.collect();
        sprints.sort_by(|a, b| b.id.cmp(&a.id));

        let mut table = Table::new();
        table.set_format(*DEFAULT_TABLE_FORMAT);
        table.set_titles(row!["ID", "Name", "State", "Start", "End"]);

        for sprint in sprints {
            table.add_row(row![
                sprint.id,
                sprint.name,
                sprint.state.unwrap_or("unknown".to_owned()),
                self.parse_date(sprint.start_date),
                self.parse_date(sprint.end_date),
            ]);
        }

        Ok(self.print_table(table, "No sprints were found for this board"))
    }

    pub fn issues(&self, options: &clap::ArgMatches) -> Result<()> {
        let (board_id, sprint_id, assignee, issue_key, all, no_subtasks) = (
            options.value_of("board"),
            options.value_of("sprint"),
            options.value_of("assignee"),
            options.value_of("issue"),
            options.is_present("all"),
            options.is_present("no-subtasks"),
        );

        let board_id = match board_id {
            Some(board_id) => board_id.to_owned(),
            None => {
                let sprint_id = sprint_id.ok_or(Error::Config("sprint".to_owned()))?;
                format!(
                    "{}",
                    self.jira
                        .sprints()
                        .get(sprint_id)?
                        .origin_board_id
                        .ok_or(Error::Config("board".to_owned()))?
                )
            }
        };
        let board = self.jira.boards().get(board_id)?;

        let mut filter = match (issue_key, all, no_subtasks) {
            (None, false, false) => vec!["status!=Done".to_owned()],
            (None, true, true) => vec!["issuetype!=Sub-Task".to_owned()],
            (None, false, true) => {
                vec!["status!=Done".to_owned(), "issuetype!=Sub-Task".to_owned()]
            }
            _ => Vec::new(),
        };

        if let Some(id) = sprint_id {
            filter.push(format!("sprint={}", id));
        }

        let search = SearchOptions::builder()
            .fields(vec![
                "assignee",
                "issuetype",
                "key",
                "parent",
                "status",
                "summary",
                "timetracking",
            ])
            .jql(&format!("{} ORDER BY issuekey", filter.join(" AND ")))
            .build();

        let issues: Vec<Issue> = self.jira.issues().iter(&board, &search)?.collect();
        let (issues, subtasks) = self.subtasks(issues, assignee, issue_key);

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);
        table.set_titles(row![
            "Key",
            "Type",
            "Summary",
            "Sub-Tasks",
            "Status",
            "Assignee",
            "Estimated",
            "Remaining",
            "Time Spent",
        ]);

        for issue in issues {
            if let Some(assignee) = assignee {
                if issue
                    .assignee()
                    .map(|v| v.display_name)
                    .unwrap_or("Unassigned".to_owned())
                    != assignee
                    && subtasks
                        .get(&issue.key)
                        .and_then(|v| {
                            v.iter().find(|v| {
                                v.assignee()
                                    .map(|v| v.display_name)
                                    .unwrap_or("Unassigned".to_owned())
                                    == assignee
                            })
                        })
                        .is_none()
                {
                    continue;
                }
            }
            if let Some(issue_key) = issue_key {
                if issue.key != issue_key
                    && subtasks
                        .get(&issue.key)
                        .and_then(|v| v.iter().find(|v| v.key == issue_key))
                        .is_none()
                {
                    continue;
                }
            }

            table.add_row(row![
                issue.key,
                issue
                    .issue_type()
                    .map(|v| v.name)
                    .unwrap_or("Unknown".to_owned()),
                self.summary(40.0, issue.summary().unwrap_or("n/a".to_owned())),
                subtasks
                    .get(&issue.key)
                    .map(|v| v
                        .iter()
                        .map(|v| self.summary(
                            60.0,
                            format!("{}: {}", v.key, v.summary().unwrap_or("n/a".to_owned()))
                        ))
                        .collect::<Vec<String>>()
                        .join("\n"))
                    .unwrap_or("-".to_owned()),
                flatten!(subtasks, issue, |v: &Issue| v
                    .status()
                    .map(|v| v.name)
                    .unwrap_or("n/a".to_owned())),
                flatten!(subtasks, issue, |v: &Issue| v
                    .assignee()
                    .map(|v| v.display_name)
                    .unwrap_or("Unassigned".to_owned())),
                flatten!(subtasks, issue, |v: &Issue| v
                    .timetracking()
                    .and_then(|v| v.original_estimate)
                    .unwrap_or("n/a".to_owned())),
                flatten!(subtasks, issue, |v: &Issue| v
                    .timetracking()
                    .and_then(|v| v.remaining_estimate)
                    .unwrap_or("n/a".to_owned())),
                flatten!(subtasks, issue, |v: &Issue| v
                    .timetracking()
                    .and_then(|v| v.time_spent)
                    .unwrap_or("n/a".to_owned())),
            ]);
        }

        Ok(self.print_table(table, "No issues were found to match your search"))
    }

    pub fn report(&self, options: &clap::ArgMatches) -> Result<()> {
        let (board_id, sprint_id, planning, update) = (
            options.value_of("board"),
            options.value_of("sprint"),
            options.is_present("planning"),
            options.is_present("update"),
        );

        let board_id = match board_id {
            Some(board_id) => board_id.to_owned(),
            None => {
                let sprint_id = sprint_id.ok_or(Error::Config("sprint".to_owned()))?;
                format!(
                    "{}",
                    self.jira
                        .sprints()
                        .get(sprint_id)?
                        .origin_board_id
                        .ok_or(Error::Config("board".to_owned()))?
                )
            }
        };
        let board = self.jira.boards().get(board_id)?;

        let mut filter = match planning {
            true => vec!["status!=Done".to_owned()],
            false => Vec::new(),
        };

        if let Some(id) = sprint_id {
            filter.push(format!("sprint={}", id));
        }

        let search = SearchOptions::builder()
            .fields(vec![
                "assignee",
                "issuetype",
                "key",
                "parent",
                "timetracking",
            ])
            .jql(&format!("{} ORDER BY assignee", filter.join(" AND ")))
            .build();

        let issues: Vec<Issue> = self.jira.issues().iter(&board, &search)?.collect();
        let (issues, subtasks) = self.subtasks(issues, None, None);

        let mut users = Users::new();
        for issue in issues {
            let estimate = flatten!(subtasks, issue, users, original_estimate_seconds);
            let remaining = flatten!(subtasks, issue, users, remaining_estimate_seconds);

            if update {
                let mut fields = BTreeMap::new();
                fields.insert(
                    "timetracking".to_owned(),
                    TimeTracking {
                        original_estimate: estimate / 60,
                        remaining_estimate: remaining / 60,
                    },
                );
                self.jira.issues().edit(&issue.id, EditIssue { fields })?;
            }

            // Make sure we also update the time spent.
            flatten!(subtasks, issue, users, time_spent_seconds);
        }

        let mut table = Table::new();
        table.set_format(*DEFAULT_TABLE_FORMAT);
        table.set_titles(row![
            "Assignee",
            "Issues",
            "Estimated",
            "Remaining",
            "Time Spent"
        ]);

        for (assignee, details) in users {
            let mut row = row![
                assignee,
                details.assignments(),
                format!("{:.1}d", details.original_estimate_days())
            ];
            if !planning {
                row.insert_cell(
                    3,
                    cell!(format!("{:.1}d", details.remaining_estimate_days())),
                );
                row.insert_cell(4, cell!(format!("{:.1}d", details.time_spent_days())));
            }
            table.add_row(row);
        }

        Ok(self.print_table(table, "No issues were found to match your search"))
    }

    fn subtasks<'a>(
        &self,
        issues: Vec<Issue>,
        assignee: Option<&str>,
        issue_key: Option<&str>,
    ) -> (Vec<Issue>, BTreeMap<String, Vec<Issue>>) {
        let mut tasks: Vec<Issue> = Vec::new();
        let mut subtasks: BTreeMap<String, Vec<Issue>> = BTreeMap::new();

        for issue in issues {
            match issue.issue_type().map(|v| v.subtask).unwrap_or(false) {
                true => {
                    if let Some(parent) = issue.parent().map(|v| v.key) {
                        if let Some(assignee) = assignee {
                            if issue
                                .assignee()
                                .map(|v| v.display_name)
                                .unwrap_or("Unassigned".to_owned())
                                != assignee
                            {
                                continue;
                            }
                        }
                        if let Some(issue_key) = issue_key {
                            if issue.key != issue_key && parent != issue_key {
                                continue;
                            }
                        }
                        match subtasks.get_mut(&parent) {
                            Some(subtasks) => subtasks.push(issue),
                            None => {
                                subtasks.insert(parent, vec![issue]);
                            }
                        };
                    }
                }
                false => tasks.push(issue),
            }
        }

        (tasks, subtasks)
    }

    fn summary(&self, part: f32, input: String) -> String {
        match self.width {
            None => return input,
            Some(width) => {
                let mut output = format!(
                    "{:.width$}",
                    input,
                    width = ((width / 100.0) * part) as usize
                );
                if output.len() + 3 < input.len() {
                    output.push_str("...");
                }
                output
            }
        }
    }

    fn parse_date(&self, date: Option<String>) -> String {
        date.and_then(|dt| {
            DateTime::parse_from_rfc3339(&dt)
                .ok()
                .and_then(|dt| Some(format!("{}", dt.format("%F %R"))))
        })
        .unwrap_or("n/a".to_owned())
    }

    fn print_table(&self, table: Table, msg: &str) {
        if table.is_empty() {
            println!("{}", msg);
        } else {
            println!();
            table.printstd();
            println!();
        }
    }
}

