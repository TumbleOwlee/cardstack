use crate::model::{Board, Status};

/// UI-R-020 — which column and which card (if any) is currently focused.
#[derive(Debug, Clone, Copy)]
pub struct Focus {
    pub column: Status,
    pub id: Option<u64>,
}

impl Focus {
    pub fn new() -> Self {
        Focus {
            column: Status::Open,
            id: None,
        }
    }

    /// UI-R-020 — ensure `id` names a task actually in `column`; if not (empty
    /// column, deleted task, freshly switched board), default to the topmost.
    pub fn resync(&mut self, board: &Board) {
        let still_present = self
            .id
            .is_some_and(|id| board.tasks_in(self.column).any(|t| t.id == id));
        if !still_present {
            self.id = board.tasks_in(self.column).map(|t| t.id).next();
        }
    }

    /// UI-R-021 — move column focus; no-op past the leftmost/rightmost column.
    pub fn move_column(&mut self, board: &Board, forward: bool) {
        let next = if forward {
            self.column.right()
        } else {
            self.column.left()
        };
        if let Some(next) = next {
            self.column = next;
            self.resync(board);
        }
    }

    /// UI-R-022 — move card focus within the current column; no-op at either edge.
    pub fn move_card(&mut self, board: &Board, forward: bool) {
        let ids: Vec<u64> = board.tasks_in(self.column).map(|t| t.id).collect();
        let Some(pos) = self.id.and_then(|id| ids.iter().position(|&i| i == id)) else {
            return;
        };
        let new_pos = if forward {
            pos + 1
        } else {
            match pos.checked_sub(1) {
                Some(p) => p,
                None => return,
            }
        };
        if let Some(&id) = ids.get(new_pos) {
            self.id = Some(id);
        }
    }
}

impl Default for Focus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board_with_tasks(n: usize) -> Board {
        let mut b = Board::new("b");
        for i in 0..n {
            b.create_task(format!("t{i}"), Status::Open);
        }
        b
    }

    /// UI-R-020
    #[test]
    fn ut_resync_defaults_to_topmost_when_id_invalid() {
        let board = board_with_tasks(2);
        let mut focus = Focus::new();
        focus.id = Some(999);
        focus.resync(&board);
        assert_eq!(focus.id, Some(board.tasks[0].id));
    }

    /// UI-R-020
    #[test]
    fn ut_resync_leaves_none_on_empty_column() {
        let board = Board::new("b");
        let mut focus = Focus::new();
        focus.resync(&board);
        assert_eq!(focus.id, None);
    }

    /// UI-R-021
    #[test]
    fn ut_move_column_noop_at_edges() {
        let board = Board::new("b");
        let mut focus = Focus::new();
        focus.move_column(&board, false); // already Open, no left
        assert_eq!(focus.column, Status::Open);
        focus.column = Status::Done;
        focus.move_column(&board, true); // already Done, no right
        assert_eq!(focus.column, Status::Done);
    }

    /// UI-R-021
    #[test]
    fn ut_move_column_resyncs_focused_card() {
        let mut board = Board::new("b");
        board.create_task("open task", Status::Open);
        board.create_task("ip task", Status::InProgress);
        let mut focus = Focus::new();
        focus.resync(&board);
        focus.move_column(&board, true);
        assert_eq!(focus.column, Status::InProgress);
        assert_eq!(focus.id, Some(board.tasks[1].id));
    }

    /// UI-R-022
    #[test]
    fn ut_move_card_noop_at_edges() {
        let board = board_with_tasks(2);
        let mut focus = Focus::new();
        focus.resync(&board);
        let first = focus.id;
        focus.move_card(&board, false); // already topmost
        assert_eq!(focus.id, first);
        focus.move_card(&board, true);
        let second = focus.id;
        assert_ne!(first, second);
        focus.move_card(&board, true); // already bottom
        assert_eq!(focus.id, second);
    }
}
