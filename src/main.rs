use jira::Client;

use anyhow::Result;
use clap::{crate_authors, crate_version, App, AppSettings, Arg, ArgGroup};

fn main() -> Result<()> {
    let global_args = vec![
        Arg::with_name("organization")
            .help("Organization")
            .short("o")
            .long("organization")
            .env("JIRA_ORGANIZATION")
            .empty_values(false)
            .hide_env_values(true)
            .display_order(1)
            .required(true),
        Arg::with_name("user")
            .help("User")
            .short("u")
            .long("user")
            .env("JIRA_USER")
            .empty_values(false)
            .hide_env_values(true)
            .display_order(2)
            .required(true),
        Arg::with_name("token")
            .help("Token")
            .short("t")
            .long("token")
            .env("JIRA_TOKEN")
            .empty_values(false)
            .hide_env_values(true)
            .display_order(3)
            .required(true),
    ];

    let app = App::new("Jira Sprint Helper")
        .about("A small tool to help prepare, start and complete sprints in Jira")
        .author(crate_authors!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .global_setting(AppSettings::ColorNever)
        .subcommand(
            App::new("boards")
                .about("List all boards you have access to")
                .args(&global_args)
                .display_order(1),
        )
        .subcommand(
            App::new("sprints")
                .about("List and filter sprints from a given board")
                .args(&global_args)
                .args(&[
                    Arg::with_name("board")
                        .help("Board ID from which to fetch sprints")
                        .short("b")
                        .long("board-id")
                        .required(true)
                        .takes_value(true)
                        .validator(|v| match v.parse::<u64>() {
                            Ok(_) => Ok(()),
                            Err(_) => Err("board ID is not a number".to_owned()),
                        }),
                    Arg::with_name("all")
                        .help("Also show closed sprints")
                        .short("A")
                        .long("all")
                        .group("filter")
                        .display_order(1),
                    Arg::with_name("active")
                        .short("a")
                        .long("active")
                        .help("Only show active sprints")
                        .group("filter")
                        .display_order(2),
                    Arg::with_name("future")
                        .help("Only show future sprints")
                        .short("f")
                        .long("future")
                        .group("filter")
                        .display_order(3),
                ])
                .display_order(2),
        )
        .subcommand(
            App::new("issues")
                .about("List, filter and search issues from a given board")
                .args(&global_args)
                .args(&[
                    Arg::with_name("board")
                        .help("Board ID from which to fetch issues")
                        .short("b")
                        .long("board-id")
                        .group("select")
                        .takes_value(true)
                        .display_order(4)
                        .validator(|v| match v.parse::<u64>() {
                            Ok(_) => Ok(()),
                            Err(_) => Err("board ID is not a number".to_owned()),
                        }),
                    Arg::with_name("sprint")
                        .help("Sprint ID from which to fetch issues")
                        .short("s")
                        .long("sprint-id")
                        .group("select")
                        .takes_value(true)
                        .display_order(5)
                        .validator(|v| match v.parse::<u64>() {
                            Ok(_) => Ok(()),
                            Err(_) => Err("sprint ID is not a number".to_owned()),
                        }),
                    Arg::with_name("assignee")
                        .help("Only show issues for a given assignee")
                        .short("a")
                        .long("assignee")
                        .group("filter")
                        .takes_value(true)
                        .display_order(6),
                    Arg::with_name("issue")
                        .help("Show details from a specific issue")
                        .short("i")
                        .long("issue")
                        .group("filter")
                        .takes_value(true)
                        .display_order(7),
                    Arg::with_name("all")
                        .help("Also show issues that are done")
                        .short("A")
                        .long("all")
                        .display_order(1),
                    Arg::with_name("no-subtasks")
                        .help("Only show stories, tasks and bugs")
                        .short("S")
                        .long("no-subtasks")
                        .display_order(2),
                ])
                .group(ArgGroup::with_name("select").required(true))
                .display_order(3),
        )
        .subcommand(
            App::new("report")
                .about("Show and update original estimates and time logged")
                .args(&global_args)
                .args(&[
                    Arg::with_name("board")
                        .help("Board ID from which to fetch issues")
                        .short("b")
                        .long("board-id")
                        .group("select")
                        .takes_value(true)
                        .display_order(4)
                        .validator(|v| match v.parse::<u64>() {
                            Ok(_) => Ok(()),
                            Err(_) => Err("board ID is not a number".to_owned()),
                        }),
                    Arg::with_name("sprint")
                        .help("Sprint ID from which to fetch issues")
                        .short("s")
                        .long("sprint-id")
                        .group("select")
                        .takes_value(true)
                        .display_order(5)
                        .validator(|v| match v.parse::<u64>() {
                            Ok(_) => Ok(()),
                            Err(_) => Err("sprint ID is not a number".to_owned()),
                        }),
                    Arg::with_name("planning")
                        .help("Ignore issues that are done")
                        .short("p")
                        .long("planning")
                        .display_order(1),
                    Arg::with_name("update")
                        .help("Update estimates and time logged")
                        .short("U")
                        .long("update")
                        .display_order(1),
                ])
                .display_order(4),
        )
        .get_matches();

    match app.subcommand() {
        ("boards", Some(options)) => Ok(Client::new(options)?.boards()?),
        ("sprints", Some(options)) => Ok(Client::new(options)?.sprints(options)?),
        ("issues", Some(options)) => Ok(Client::new(options)?.issues(options)?),
        ("report", Some(options)) => Ok(Client::new(options)?.report(options)?),
        _ => unreachable!(),
    }
}

