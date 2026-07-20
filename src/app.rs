use std::io::Stdout;
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ferrowl_ui::AlternateScreen;
use ferrowl_ui::state::{InputFieldState, InputFieldStateBuilder, ScrollingTabsState};
use ferrowl_ui::traits::{HandleEvents, SetFocus};

use crate::command::{self, Command};
use crate::dialog::{
    CategoryDialog, CategoryDialogAction, ConfirmAction, ConfirmDialog, DialogAction, DialogTarget,
    TaskDialog,
};
use crate::filter::Filter;
use crate::focus::Focus;
use crate::model::Board;
use crate::storage;
use crate::ui;

/// Top-level application state: every open board, which one is active, and
/// what's focused on the active board.
pub struct App {
    pub dir: PathBuf,
    pub boards: Vec<Board>,
    pub active: usize,
    pub focus: Focus,
    pub dialog: Option<TaskDialog>,
    pub confirm_delete: Option<ConfirmDialog>,
    pub category_dialog: Option<CategoryDialog>,
    pub cmdline: Option<InputFieldState>,
    /// UI-R-051 — single-line error from the last unrecognized command.
    pub cmdline_error: Option<String>,
    /// UI-R-004 — `Ctrl+T` was pressed; the next key is a board index.
    pub pending_tab_switch: bool,
    pub should_quit: bool,
}

impl App {
    /// BD-R-003, ST-R-020 — load every board file, or bootstrap a `Default`
    /// board if none exist yet.
    pub fn load(dir: PathBuf) -> std::io::Result<Self> {
        let mut boards = storage::load_all(&dir)?;
        if boards.is_empty() {
            let board = Board::new("Default");
            storage::save(&dir, &board)?;
            boards.push(board);
        }
        // ST-R-022 — restore the persisted tab order; boards not listed
        // (freshly discovered files) are appended at the end.
        let config_dir = dir
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| dir.clone());
        let order = storage::load_order(&config_dir);
        boards.sort_by_key(|b| {
            order
                .iter()
                .position(|n| n == &b.name)
                .unwrap_or(usize::MAX)
        });
        let _ = storage::save_order(
            &config_dir,
            &boards.iter().map(|b| b.name.clone()).collect::<Vec<_>>(),
        );
        let mut focus = Focus::new();
        focus.resync(&boards[0]);
        Ok(App {
            dir,
            boards,
            active: 0,
            focus,
            dialog: None,
            confirm_delete: None,
            category_dialog: None,
            cmdline: None,
            cmdline_error: None,
            pending_tab_switch: false,
            should_quit: false,
        })
    }

    /// UI-R-004 — tab titles are prefixed with their board's index.
    pub fn tabs_state(&self) -> ScrollingTabsState<String> {
        ScrollingTabsState {
            titles: self
                .boards
                .iter()
                .enumerate()
                .map(|(i, b)| format!(" [{i}] {} ", b.name))
                .collect(),
            selected: self.active,
        }
    }

    /// UI-R-004 — switch directly to the board at `idx`, if it exists.
    fn switch_to_board_index(&mut self, idx: usize) {
        if idx < self.boards.len() {
            self.active = idx;
            self.focus = Focus::new();
            self.focus.resync(&self.boards[self.active]);
        }
    }

    fn active_board(&self) -> &Board {
        &self.boards[self.active]
    }

    /// ST-R-010 — every mutation autosaves immediately.
    fn autosave(&self) {
        let _ = storage::save(&self.dir, self.active_board());
    }

    /// UI-R-003 — switch the active board tab.
    pub fn next_board(&mut self) {
        if !self.boards.is_empty() {
            self.active = (self.active + 1) % self.boards.len();
            let board = &self.boards[self.active];
            self.focus.resync(board);
        }
    }

    pub fn prev_board(&mut self) {
        if !self.boards.is_empty() {
            self.active = (self.active + self.boards.len() - 1) % self.boards.len();
            let board = &self.boards[self.active];
            self.focus.resync(board);
        }
    }

    /// UI-R-021
    fn move_column_focus(&mut self, forward: bool) {
        let board = &self.boards[self.active];
        self.focus.move_column(board, forward);
    }

    /// UI-R-022
    fn move_card_focus(&mut self, forward: bool) {
        let board = &self.boards[self.active];
        self.focus.move_card(board, forward);
    }

    /// BD-R-020, BD-R-021, UI-R-030 — move the focused task's status,
    /// following it into the target column.
    fn move_status_focus(&mut self, forward: bool) {
        let Some(id) = self.focus.id else { return };
        let target = if forward {
            self.focus.column.right()
        } else {
            self.focus.column.left()
        };
        let Some(target) = target else { return };
        self.boards[self.active].move_status(id, target);
        self.focus.column = target;
        self.autosave();
    }

    /// BD-R-031, UI-R-031
    fn reorder_focus(&mut self, forward: bool) {
        let Some(id) = self.focus.id else { return };
        self.boards[self.active].reorder(id, forward);
        self.autosave();
    }

    /// UI-R-040 — open the task detail dialog on the focused card.
    fn open_dialog(&mut self) {
        let Some(id) = self.focus.id else { return };
        let board = &self.boards[self.active];
        if let Some(task) = board.task(id) {
            self.dialog = Some(TaskDialog::for_task(task, board));
        }
    }

    /// UI-R-052 — open the task detail dialog blank, for `:new-task`.
    fn open_blank_dialog(&mut self) {
        let status = self.focus.column;
        let board = &self.boards[self.active];
        self.dialog = Some(TaskDialog::blank(status, board));
    }

    /// UI-R-040, UI-R-052, BD-R-014 — apply a confirmed dialog's fields,
    /// creating the task first if this was a create-flow dialog.
    fn confirm_dialog(&mut self) {
        let Some(dialog) = self.dialog.take() else {
            return;
        };
        match dialog.target {
            DialogTarget::Edit(id) => {
                if let Some(task) = self.boards[self.active].task_mut(id) {
                    dialog.apply(task);
                }
            }
            DialogTarget::Create(status) => {
                let board = &mut self.boards[self.active];
                let id = board.create_task(dialog.title.input().trim(), status);
                if let Some(task) = board.task_mut(id) {
                    dialog.apply(task);
                }
                self.focus.column = status;
                self.focus.id = Some(id);
            }
        }
        self.autosave();
    }

    fn handle_dialog_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let Some(dialog) = &mut self.dialog else {
            return;
        };
        match dialog.handle_key(code, modifiers) {
            DialogAction::None => {}
            DialogAction::Cancel => self.dialog = None,
            DialogAction::Confirm => self.confirm_dialog(),
        }
    }

    /// UI-R-042 — open the delete-confirmation dialog on the focused card.
    fn open_delete_confirm(&mut self) {
        let Some(id) = self.focus.id else { return };
        if let Some(task) = self.boards[self.active].task(id) {
            self.confirm_delete = Some(ConfirmDialog::for_task(id, task.title.clone()));
        }
    }

    /// BD-R-013 — the only path that deletes a task.
    fn handle_confirm_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let Some(confirm) = &mut self.confirm_delete else {
            return;
        };
        match confirm.handle_key(code, modifiers) {
            ConfirmAction::None => {}
            ConfirmAction::Cancel => self.confirm_delete = None,
            ConfirmAction::Confirm => {
                let target = confirm.target;
                self.confirm_delete = None;
                match target {
                    crate::dialog::ConfirmTarget::Task(id) => {
                        self.boards[self.active].delete_task(id);
                        self.focus.resync(&self.boards[self.active]);
                        self.autosave();
                    }
                    crate::dialog::ConfirmTarget::Board => self.delete_active_board(),
                }
            }
        }
    }

    fn handle_category_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let Some(dialog) = &mut self.category_dialog else {
            return;
        };
        let board = &mut self.boards[self.active];
        match dialog.handle_key(code, modifiers, board) {
            CategoryDialogAction::None => {}
            CategoryDialogAction::Mutated => self.autosave(),
            CategoryDialogAction::Close => self.category_dialog = None,
        }
    }

    /// UI-R-050 — enter command-line mode.
    fn open_command_line(&mut self) {
        let mut field = InputFieldStateBuilder::default()
            .build()
            .expect("InputFieldStateBuilder: all fields defaulted");
        field.set_focused(true);
        self.cmdline = Some(field);
        self.cmdline_error = None;
    }

    fn handle_cmdline_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let Some(field) = &mut self.cmdline else {
            return;
        };
        match code {
            KeyCode::Esc => self.cmdline = None,
            KeyCode::Enter => {
                let text = field.input().clone();
                self.cmdline = None;
                self.run_command(&text);
            }
            _ => {
                field.handle_events(modifiers, code);
            }
        }
    }

    /// UI-R-051 — dispatch a submitted `:` command.
    fn run_command(&mut self, text: &str) {
        match command::parse(text) {
            Command::NewTask => self.open_blank_dialog(),
            Command::NewBoard(name) => self.new_board(name),
            Command::Rename(name) => self.rename_active_board(name),
            Command::Swap(i, j) => self.swap_boards(i, j),
            Command::Delete => {
                let name = self.boards[self.active].name.clone();
                self.confirm_delete = Some(ConfirmDialog::for_board(name));
            }
            Command::Categories => {
                self.category_dialog = Some(CategoryDialog::new(&self.boards[self.active]))
            }
            Command::Filter(query) => self.set_filter(&query),
            Command::Quit => self.should_quit = true,
            Command::Unknown(cmd) => {
                self.cmdline_error = Some(format!("Unknown command: {cmd}"));
            }
        }
    }

    /// UI-R-060 — apply, replace, or clear the active board's filter from a
    /// `:filter` condition. An invalid condition surfaces a command-line error
    /// (`UI-R-051`) and leaves any existing filter unchanged.
    fn set_filter(&mut self, query: &str) {
        match Filter::parse(query) {
            Ok(filter) => {
                self.boards[self.active].filter = filter;
                // The filter can hide the focused card; drop to a visible one.
                self.focus.resync(&self.boards[self.active]);
            }
            Err(()) => {
                self.cmdline_error = Some(format!("Invalid filter: {query}"));
            }
        }
    }

    /// UI-R-060 — clear the active board's filter (bound to `Esc` in board
    /// view). Returns whether a filter was actually cleared.
    fn clear_filter(&mut self) -> bool {
        if self.boards[self.active].filter.take().is_some() {
            self.focus.resync(&self.boards[self.active]);
            true
        } else {
            false
        }
    }

    fn config_dir(&self) -> PathBuf {
        self.dir
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.dir.clone())
    }

    /// ST-R-022 — persist the current tab order immediately.
    fn save_order(&self) {
        let names: Vec<String> = self.boards.iter().map(|b| b.name.clone()).collect();
        let _ = storage::save_order(&self.config_dir(), &names);
    }

    /// UI-R-053, ST-R-011 — create and switch to a new empty board.
    fn new_board(&mut self, name: String) {
        let name = name.trim();
        if name.is_empty() {
            return;
        }
        let board = Board::new(name);
        let _ = storage::save(&self.dir, &board);
        self.boards.push(board);
        self.active = self.boards.len() - 1;
        self.focus = Focus::new();
        self.focus.resync(&self.boards[self.active]);
        self.save_order();
    }

    /// UI-R-058, ST-R-022 — swap the tab positions of the boards at `i`/`j`.
    fn swap_boards(&mut self, i: usize, j: usize) {
        if i >= self.boards.len() || j >= self.boards.len() || i == j {
            return;
        }
        self.boards.swap(i, j);
        if self.active == i {
            self.active = j;
        } else if self.active == j {
            self.active = i;
        }
        self.save_order();
    }

    /// UI-R-059, ST-R-014 — delete the active board; bootstrap a fresh
    /// `Default` board if it was the only one open.
    fn delete_active_board(&mut self) {
        let name = self.boards[self.active].name.clone();
        let _ = storage::delete(&self.dir, &name);
        self.boards.remove(self.active);
        if self.boards.is_empty() {
            let board = Board::new("Default");
            let _ = storage::save(&self.dir, &board);
            self.boards.push(board);
        }
        self.active = self.active.min(self.boards.len() - 1);
        self.focus = Focus::new();
        self.focus.resync(&self.boards[self.active]);
        self.save_order();
    }

    /// UI-R-057, ST-R-012 — rename the active board.
    fn rename_active_board(&mut self, name: String) {
        let name = name.trim();
        if name.is_empty() {
            return;
        }
        let old_name = self.boards[self.active].name.clone();
        self.boards[self.active].name = name.to_string();
        let _ = storage::rename(&self.dir, &old_name, &self.boards[self.active]);
        self.save_order();
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        if self.cmdline.is_some() {
            self.handle_cmdline_key(code, modifiers);
            return;
        }
        if self.dialog.is_some() {
            self.handle_dialog_key(code, modifiers);
            return;
        }
        if self.confirm_delete.is_some() {
            self.handle_confirm_key(code, modifiers);
            return;
        }
        if self.category_dialog.is_some() {
            self.handle_category_key(code, modifiers);
            return;
        }
        if self.pending_tab_switch {
            self.pending_tab_switch = false;
            if let KeyCode::Char(c) = code
                && let Some(digit) = c.to_digit(10)
            {
                self.switch_to_board_index(digit as usize);
            }
            return;
        }
        self.cmdline_error = None;
        let shift = modifiers.contains(KeyModifiers::SHIFT);
        match code {
            KeyCode::Enter => self.open_dialog(),
            KeyCode::Char('d') => self.open_delete_confirm(),
            KeyCode::Char(':') => self.open_command_line(),
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char(']') => self.next_board(),
            KeyCode::Char('[') => self.prev_board(),
            KeyCode::Char('t') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.pending_tab_switch = true;
            }

            KeyCode::Char('H') => self.move_status_focus(false),
            KeyCode::Char('L') => self.move_status_focus(true),
            KeyCode::Char('J') => self.reorder_focus(true),
            KeyCode::Char('K') => self.reorder_focus(false),

            KeyCode::Left if shift => self.move_status_focus(false),
            KeyCode::Right if shift => self.move_status_focus(true),
            KeyCode::Down if shift => self.reorder_focus(true),
            KeyCode::Up if shift => self.reorder_focus(false),

            KeyCode::Char('h') | KeyCode::Left => self.move_column_focus(false),
            KeyCode::Char('l') | KeyCode::Right => self.move_column_focus(true),
            KeyCode::Char('j') | KeyCode::Down => self.move_card_focus(true),
            KeyCode::Char('k') | KeyCode::Up => self.move_card_focus(false),
            // UI-R-060 — Esc clears the active board's filter, if one is set.
            KeyCode::Esc => {
                self.clear_filter();
            }
            _ => {}
        }
    }

    /// UI-R-001 — the main event/redraw loop.
    pub fn run(&mut self, screen: &mut AlternateScreen<Stdout>) -> std::io::Result<()> {
        while !self.should_quit {
            screen.draw(|frame| ui::render(frame, self))?;
            if event::poll(std::time::Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.handle_key(key.code, key.modifiers);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App {
            dir: std::env::temp_dir(),
            boards: vec![Board::new("b")],
            active: 0,
            focus: Focus::new(),
            dialog: None,
            confirm_delete: None,
            category_dialog: None,
            cmdline: None,
            cmdline_error: None,
            pending_tab_switch: false,
            should_quit: false,
        }
    }

    /// UI-R-051
    #[test]
    fn ut_unknown_command_sets_cmdline_error_not_stderr() {
        let mut a = app();
        a.run_command("bogus");
        assert_eq!(a.cmdline_error.as_deref(), Some("Unknown command: bogus"));
    }

    /// UI-R-051 — a board-view key clears a stale command error.
    #[test]
    fn ut_board_key_clears_cmdline_error() {
        let mut a = app();
        a.cmdline_error = Some("Unknown command: x".to_string());
        a.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(a.cmdline_error, None);
    }

    /// UI-R-050 — opening the command line clears a stale error.
    #[test]
    fn ut_open_command_line_clears_cmdline_error() {
        let mut a = app();
        a.cmdline_error = Some("Unknown command: x".to_string());
        a.handle_key(KeyCode::Char(':'), KeyModifiers::NONE);
        assert_eq!(a.cmdline_error, None);
        assert!(a.cmdline.is_some());
    }

    use crate::model::Status;

    fn visible_titles(a: &App) -> Vec<String> {
        a.active_board()
            .visible_tasks_in(Status::Open)
            .map(|t| t.title.clone())
            .collect()
    }

    /// UI-R-060 — `:filter` narrows the visible cards to matching tasks.
    #[test]
    fn ut_filter_command_narrows_visible_cards() {
        let mut a = app();
        let b = &mut a.boards[0];
        b.create_task("bug task", Status::Open);
        b.tasks[0].labels = vec!["bug".to_string()];
        b.create_task("plain task", Status::Open);

        a.run_command("filter label=bug");
        assert_eq!(visible_titles(&a), vec!["bug task"]);
    }

    /// UI-R-060 — bare `:filter` and `:filter clear` both clear the filter.
    #[test]
    fn ut_filter_command_clears() {
        let mut a = app();
        a.boards[0].create_task("t", Status::Open);
        a.boards[0].tasks[0].labels = vec!["bug".to_string()];

        a.run_command("filter label=nomatch");
        assert!(visible_titles(&a).is_empty());
        a.run_command("filter");
        assert_eq!(visible_titles(&a), vec!["t"]);

        a.run_command("filter label=nomatch");
        assert!(visible_titles(&a).is_empty());
        a.run_command("filter clear");
        assert_eq!(visible_titles(&a), vec!["t"]);
    }

    /// UI-R-060 — `Esc` in board view clears an active filter.
    #[test]
    fn ut_esc_clears_filter() {
        let mut a = app();
        a.boards[0].create_task("t", Status::Open);
        a.boards[0].tasks[0].labels = vec!["bug".to_string()];
        a.run_command("filter label=nomatch");
        assert!(visible_titles(&a).is_empty());

        a.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(visible_titles(&a), vec!["t"]);
    }

    /// UI-R-060 — an invalid condition errors and leaves the filter unchanged.
    #[test]
    fn ut_invalid_filter_errors_and_keeps_existing() {
        let mut a = app();
        a.boards[0].create_task("bug task", Status::Open);
        a.boards[0].tasks[0].labels = vec!["bug".to_string()];
        a.boards[0].create_task("plain task", Status::Open);

        a.run_command("filter label=bug");
        a.run_command("filter label=a&b|c");
        assert!(a.cmdline_error.is_some());
        // The prior filter still applies.
        assert_eq!(visible_titles(&a), vec!["bug task"]);
    }
}
