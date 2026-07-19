/// UI-R-051 — the generic `:` commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    NewTask,
    NewBoard(String),
    Rename(String),
    Swap(usize, usize),
    Delete,
    Categories,
    Quit,
    Unknown(String),
}

/// UI-R-054 — the command table shown in the command-line help popup
/// (mirrors `docs/specs/tui/api-contract.md`'s `:` command table).
pub const HELP: &[(&str, &str)] = &[
    (":new-task", "Open the task detail dialog blank"),
    (
        ":new-board <name>",
        "Create and switch to a new empty board",
    ),
    (":rename <name>", "Rename the active board"),
    (":swap <i> <j>", "Swap the tab positions of two boards"),
    (":delete", "Delete the active board (with confirmation)"),
    (":categories", "Open the category-management dialog"),
    (":q", "Quit the application"),
];

/// UI-R-051 — parse the text typed after `:` (without the leading `:`).
pub fn parse(input: &str) -> Command {
    let input = input.trim();
    let (name, rest) = match input.split_once(' ') {
        Some((n, r)) => (n, r.trim()),
        None => (input, ""),
    };
    match name {
        "new-task" => Command::NewTask,
        "new-board" => Command::NewBoard(rest.to_string()),
        "rename" => Command::Rename(rest.to_string()),
        "swap" => {
            let mut parts = rest.split_whitespace();
            match (
                parts.next().and_then(|s| s.parse().ok()),
                parts.next().and_then(|s| s.parse().ok()),
                parts.next(),
            ) {
                (Some(i), Some(j), None) => Command::Swap(i, j),
                _ => Command::Unknown(input.to_string()),
            }
        }
        "delete" => Command::Delete,
        "categories" => Command::Categories,
        "q" => Command::Quit,
        _ => Command::Unknown(input.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// UI-R-051
    #[test]
    fn ut_parse_new_task() {
        assert_eq!(parse("new-task"), Command::NewTask);
    }

    /// UI-R-051, UI-R-053
    #[test]
    fn ut_parse_new_board_captures_name() {
        assert_eq!(
            parse("new-board  Project X "),
            Command::NewBoard("Project X".to_string())
        );
    }

    /// UI-R-051, UI-R-057
    #[test]
    fn ut_parse_rename_captures_name() {
        assert_eq!(
            parse("rename  New Name "),
            Command::Rename("New Name".to_string())
        );
    }

    /// UI-R-051, UI-R-058
    #[test]
    fn ut_parse_swap_captures_indices() {
        assert_eq!(parse("swap 0 2"), Command::Swap(0, 2));
        assert_eq!(
            parse("swap 0"),
            Command::Unknown("swap 0".to_string()),
            "malformed swap syntax is unrecognized, not silently ignored"
        );
    }

    /// UI-R-051
    #[test]
    fn ut_parse_categories_and_quit() {
        assert_eq!(parse("categories"), Command::Categories);
        assert_eq!(parse("q"), Command::Quit);
    }

    /// UI-R-051, UI-R-059
    #[test]
    fn ut_parse_delete() {
        assert_eq!(parse("delete"), Command::Delete);
    }

    /// UI-R-051 — an unrecognized command is reported, not silently accepted.
    #[test]
    fn ut_parse_unknown_command() {
        assert_eq!(parse("bogus"), Command::Unknown("bogus".to_string()));
    }
}
