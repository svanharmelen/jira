macro_rules! flatten {
    ($subtasks:ident, $issue:ident, $filter:expr) => {
        $subtasks
            .get(&$issue.key)
            .map(|v| v.iter().map($filter).collect::<Vec<String>>().join("\n"))
            .unwrap_or_else(|| $filter(&$issue))
    };
    ($subtasks:ident, $issue:ident, $users:ident, $field:ident) => {
        $subtasks
            .get(&$issue.key)
            .map(|v| {
                v.iter()
                    .map(|v| {
                        let assignee = v
                            .assignee()
                            .map(|v| v.display_name)
                            .unwrap_or("Unassigned".to_owned());
                        v.timetracking()
                            .and_then(|v| $users.$field(assignee, v.$field))
                            .unwrap_or(0)
                    })
                    .sum()
            })
            .unwrap_or_else(|| {
                let assignee = $issue
                    .assignee()
                    .map(|v| v.display_name)
                    .unwrap_or("Unassigned".to_owned());
                $issue
                    .timetracking()
                    .and_then(|v| $users.$field(assignee, v.$field))
                    .unwrap_or(0)
            })
    };
}

