use crossterm::event::{KeyCode, KeyModifiers};
use ferrowl_ui::state::{
    InputFieldState, InputFieldStateBuilder, SelectionState, SelectionStateBuilder,
};
use ferrowl_ui::traits::{HandleEvents, SetFocus, ToLabel};
use ferrowl_ui::widgets::GetValue;

use crate::model::{Board, Status, Task};

/// A category picker entry: `None` renders as "(none)".
#[derive(Debug, Clone, PartialEq)]
pub struct CategoryChoice(pub Option<String>);

impl ToLabel for CategoryChoice {
    fn to_label(&self) -> String {
        self.0.clone().unwrap_or_else(|| "(none)".to_string())
    }
}

/// UI-R-041 — which field of the task detail dialog has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogField {
    Title,
    DueDate,
    Category,
    Labels,
    Description,
    Save,
}

impl DialogField {
    const ORDER: [DialogField; 6] = [
        DialogField::Title,
        DialogField::DueDate,
        DialogField::Category,
        DialogField::Labels,
        DialogField::Description,
        DialogField::Save,
    ];

    fn position(self) -> usize {
        Self::ORDER.iter().position(|f| *f == self).unwrap()
    }

    fn next(self) -> Self {
        Self::ORDER[(self.position() + 1) % Self::ORDER.len()]
    }

    fn prev(self) -> Self {
        let len = Self::ORDER.len();
        Self::ORDER[(self.position() + len - 1) % len]
    }
}

/// UI-R-040, UI-R-052 — whether the dialog edits an existing task or (via
/// `:new-task`) creates one.
#[derive(Debug, Clone, Copy)]
pub enum DialogTarget {
    Edit(u64),
    Create(Status),
}

/// Outcome of a key handled by the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAction {
    None,
    /// UI-R-044 — close without applying.
    Cancel,
    /// UI-R-040 — apply edits and close.
    Confirm,
}

/// UI-R-041 — the task detail dialog: title, due date, category, labels, and
/// a multi-line description, plus a save control.
pub struct TaskDialog {
    pub target: DialogTarget,
    pub field: DialogField,
    pub title: InputFieldState,
    pub due_date: InputFieldState,
    pub category: SelectionState<CategoryChoice>,
    pub labels: InputFieldState,
    pub description: InputFieldState,
}

fn category_values(board: &Board) -> Vec<CategoryChoice> {
    let mut values = vec![CategoryChoice(None)];
    values.extend(
        board
            .categories
            .iter()
            .map(|c| CategoryChoice(Some(c.name.clone()))),
    );
    values
}

fn labels_to_text(labels: &[String]) -> String {
    labels.join(", ")
}

fn text_to_labels(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

impl TaskDialog {
    fn build(
        target: DialogTarget,
        title: &str,
        due_date: &str,
        category: Option<&str>,
        labels: &str,
        description: &str,
        board: &Board,
    ) -> Self {
        let values = category_values(board);
        let selected = values
            .iter()
            .position(|c| c.0.as_deref() == category)
            .unwrap_or(0);
        let mut category_state = SelectionStateBuilder::default()
            .values(values)
            .build()
            .expect("SelectionStateBuilder: values is the only required field");
        for _ in 0..selected {
            category_state.move_down();
        }
        category_state.set_focused(false);

        let mut dialog = TaskDialog {
            target,
            field: DialogField::Title,
            title: field_state(title),
            due_date: field_state(due_date),
            category: category_state,
            labels: field_state(labels),
            description: field_state(description),
        };
        dialog.title.set_focused(true);
        dialog
    }

    /// UI-R-040 — open pre-filled with an existing task's fields.
    pub fn for_task(task: &Task, board: &Board) -> Self {
        let due = task.due_date.map(|d| d.to_string()).unwrap_or_default();
        Self::build(
            DialogTarget::Edit(task.id),
            &task.title,
            &due,
            task.category.as_deref(),
            &labels_to_text(&task.labels),
            &task.description,
            board,
        )
    }

    /// UI-R-052 — open blank, defaulting status to `status`.
    pub fn blank(status: Status, board: &Board) -> Self {
        Self::build(DialogTarget::Create(status), "", "", None, "", "", board)
    }

    fn set_field_focus(&mut self, field: DialogField, focused: bool) {
        match field {
            DialogField::Title => self.title.set_focused(focused),
            DialogField::DueDate => self.due_date.set_focused(focused),
            DialogField::Category => self.category.set_focused(focused),
            DialogField::Labels => self.labels.set_focused(focused),
            DialogField::Description => self.description.set_focused(focused),
            DialogField::Save => {}
        }
    }

    fn focus_next(&mut self) {
        self.set_field_focus(self.field, false);
        self.field = self.field.next();
        self.set_field_focus(self.field, true);
    }

    fn focus_prev(&mut self) {
        self.set_field_focus(self.field, false);
        self.field = self.field.prev();
        self.set_field_focus(self.field, true);
    }

    /// UI-R-041, UI-R-044 — route a key to the focused field, or to
    /// Tab-cycling / cancel / confirm.
    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> DialogAction {
        match code {
            KeyCode::Esc => return DialogAction::Cancel,
            KeyCode::Tab => {
                self.focus_next();
                return DialogAction::None;
            }
            KeyCode::BackTab => {
                self.focus_prev();
                return DialogAction::None;
            }
            KeyCode::Enter if self.field == DialogField::Save => {
                if self.title.input().trim().is_empty() {
                    return DialogAction::None;
                }
                return DialogAction::Confirm;
            }
            _ => {}
        }
        match self.field {
            DialogField::Title => {
                self.title.handle_events(modifiers, code);
            }
            DialogField::DueDate => {
                self.due_date.handle_events(modifiers, code);
            }
            DialogField::Category => {
                self.category.handle_events(modifiers, code);
            }
            DialogField::Labels => {
                self.labels.handle_events(modifiers, code);
            }
            DialogField::Description => {
                self.description.handle_events(modifiers, code);
            }
            DialogField::Save => {}
        }
        DialogAction::None
    }

    /// BD-R-014 — write the dialog's fields onto `task`.
    pub fn apply(&self, task: &mut Task) {
        task.title = self.title.input().trim().to_string();
        task.due_date = self.due_date.input().trim().parse().ok();
        task.category = self.category.get_value().0;
        task.labels = text_to_labels(self.labels.input());
        task.description = self.description.input().clone();
    }
}

/// UI-R-042, UI-R-059 — what a confirm dialog, once confirmed, deletes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmTarget {
    Task(u64),
    Board,
}

/// UI-R-042, UI-R-059 — yes/no confirmation before deleting a task or board.
pub struct ConfirmDialog {
    pub target: ConfirmTarget,
    pub title: String,
    pub yes_focused: bool,
}

/// Outcome of a key handled by a confirm dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    None,
    /// UI-R-044 — close without applying.
    Cancel,
    /// The `Yes` control was activated.
    Confirm,
}

impl ConfirmDialog {
    pub fn for_task(task_id: u64, title: String) -> Self {
        ConfirmDialog {
            target: ConfirmTarget::Task(task_id),
            title,
            yes_focused: false,
        }
    }

    pub fn for_board(title: String) -> Self {
        ConfirmDialog {
            target: ConfirmTarget::Board,
            title,
            yes_focused: false,
        }
    }

    /// UI-R-042, UI-R-044 — Tab toggles Yes/No, Enter activates the focused
    /// control, Esc always cancels.
    pub fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> ConfirmAction {
        match code {
            KeyCode::Esc => ConfirmAction::Cancel,
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Left | KeyCode::Right => {
                self.yes_focused = !self.yes_focused;
                ConfirmAction::None
            }
            KeyCode::Enter => {
                if self.yes_focused {
                    ConfirmAction::Confirm
                } else {
                    ConfirmAction::Cancel
                }
            }
            _ => ConfirmAction::None,
        }
    }
}

/// UI-R-043 — one item of the category list: name plus its color, used to
/// colorize the selected item's highlight.
#[derive(Debug, Clone, Default)]
pub struct CategoryItem {
    pub name: String,
    pub color: (u8, u8, u8),
}

impl ToLabel for CategoryItem {
    /// UI-R-043 — always shown uppercase.
    fn to_label(&self) -> String {
        format!(" {} ", self.name.to_uppercase())
    }
}

fn category_items(board: &Board) -> Vec<CategoryItem> {
    board
        .categories
        .iter()
        .map(|c| CategoryItem {
            name: c.name.clone(),
            color: c.color,
        })
        .collect()
}

/// UI-R-043 — which control has focus in the category dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryFocus {
    List,
    AddInput,
}

/// UI-R-043 — what confirming the add/rename input does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    Add,
    Rename(usize),
}

/// UI-R-043 — the category-management dialog: a scrollable selection list of
/// the board's categories, plus an always-visible add/rename input below it.
pub struct CategoryDialog {
    pub list: SelectionState<CategoryItem>,
    pub input: InputFieldState,
    pub focus: CategoryFocus,
    mode: InputMode,
}

/// Outcome of a key handled by the category dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryDialogAction {
    None,
    /// A category was added, renamed, recolored, or deleted — autosave.
    Mutated,
    /// UI-R-044 — close without applying any pending input text.
    Close,
}

impl CategoryDialog {
    pub fn new(board: &Board) -> Self {
        let mut list = SelectionStateBuilder::default()
            .values(category_items(board))
            .build()
            .expect("SelectionStateBuilder: values is the only required field");
        list.set_focused(true);
        CategoryDialog {
            list,
            input: field_state(""),
            focus: CategoryFocus::List,
            mode: InputMode::Add,
        }
    }

    /// UI-R-043 — label for the add/rename input, reflecting its current mode.
    pub fn input_label(&self) -> &'static str {
        match self.mode {
            InputMode::Add => "New category",
            InputMode::Rename(_) => "Rename to",
        }
    }

    fn reset_input(&mut self) {
        self.input = field_state("");
        self.mode = InputMode::Add;
    }

    fn switch_focus(&mut self, focus: CategoryFocus) {
        self.focus = focus;
        self.list.set_focused(focus == CategoryFocus::List);
        self.input.set_focused(focus == CategoryFocus::AddInput);
    }

    /// Rebuild the list from `board`, keeping the selection as close as
    /// possible to `target` (clamped to the new length).
    fn rebuild_list(&mut self, board: &Board, target: usize) {
        let mut list = SelectionStateBuilder::default()
            .values(category_items(board))
            .build()
            .expect("SelectionStateBuilder: values is the only required field");
        list.set_focused(self.list.focused());
        let len = list.values().len();
        let target = if len == 0 { 0 } else { target.min(len - 1) };
        for _ in 0..target {
            list.move_down();
        }
        self.list = list;
    }

    /// UI-R-043, UI-R-044 — Esc always fully closes the dialog, discarding
    /// any in-progress add/rename text (there is no separate nested dialog).
    pub fn handle_key(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
        board: &mut Board,
    ) -> CategoryDialogAction {
        if code == KeyCode::Esc {
            return CategoryDialogAction::Close;
        }
        match self.focus {
            CategoryFocus::List => {
                let idx = self.list.selection();
                match code {
                    KeyCode::Tab | KeyCode::BackTab => {
                        self.reset_input();
                        self.switch_focus(CategoryFocus::AddInput);
                        CategoryDialogAction::None
                    }
                    KeyCode::Enter => {
                        if let Some(name) = board.categories.get(idx).map(|c| c.name.clone()) {
                            self.mode = InputMode::Rename(idx);
                            self.input = field_state(&name);
                            self.switch_focus(CategoryFocus::AddInput);
                        }
                        CategoryDialogAction::None
                    }
                    KeyCode::Char('c') => {
                        let Some(name) = board.categories.get(idx).map(|c| c.name.clone()) else {
                            return CategoryDialogAction::None;
                        };
                        board.cycle_category_color(&name);
                        self.rebuild_list(board, idx);
                        CategoryDialogAction::Mutated
                    }
                    KeyCode::Char('d') | KeyCode::Char('x') => {
                        let Some(name) = board.categories.get(idx).map(|c| c.name.clone()) else {
                            return CategoryDialogAction::None;
                        };
                        board.delete_category(&name);
                        self.rebuild_list(board, idx);
                        CategoryDialogAction::Mutated
                    }
                    _ => {
                        self.list.handle_events(modifiers, code);
                        CategoryDialogAction::None
                    }
                }
            }
            CategoryFocus::AddInput => match code {
                KeyCode::Tab | KeyCode::BackTab => {
                    self.reset_input();
                    self.switch_focus(CategoryFocus::List);
                    CategoryDialogAction::None
                }
                KeyCode::Enter => {
                    let name = self.input.input().trim().to_string();
                    let mode = self.mode;
                    let target = self.list.selection();
                    self.reset_input();
                    self.switch_focus(CategoryFocus::List);
                    if name.is_empty() {
                        return CategoryDialogAction::None;
                    }
                    match mode {
                        InputMode::Add => board.create_category(name),
                        InputMode::Rename(idx) => {
                            if let Some(old) = board.categories.get(idx).map(|c| c.name.clone()) {
                                board.rename_category(&old, name);
                            }
                        }
                    }
                    self.rebuild_list(board, target);
                    CategoryDialogAction::Mutated
                }
                _ => {
                    self.input.handle_events(modifiers, code);
                    CategoryDialogAction::None
                }
            },
        }
    }
}

fn field_state(initial: &str) -> InputFieldState {
    InputFieldStateBuilder::default()
        .input(initial.to_string())
        .cursor(initial.chars().count())
        .build()
        .expect("InputFieldStateBuilder: all fields defaulted")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn board_with_category() -> Board {
        let mut b = Board::new("b");
        b.create_category("bug");
        b
    }

    /// UI-R-040
    #[test]
    fn ut_for_task_prefills_fields() {
        let board = board_with_category();
        let mut task = Task {
            id: 1,
            title: "T".to_string(),
            description: "desc".to_string(),
            due_date: Some(NaiveDate::from_ymd_opt(2026, 1, 2).unwrap()),
            category: Some("bug".to_string()),
            labels: vec!["a".to_string(), "b".to_string()],
            status: Status::Open,
        };
        let dialog = TaskDialog::for_task(&task, &board);
        assert_eq!(dialog.title.input(), "T");
        assert_eq!(dialog.due_date.input(), "2026-01-02");
        assert_eq!(dialog.labels.input(), "a, b");
        assert_eq!(
            dialog.category.get_value(),
            CategoryChoice(Some("bug".to_string()))
        );

        dialog.apply(&mut task);
        assert_eq!(task.title, "T");
    }

    /// UI-R-041
    #[test]
    fn ut_tab_cycles_through_all_fields_and_wraps() {
        let board = Board::new("b");
        let mut dialog = TaskDialog::blank(Status::Open, &board);
        assert_eq!(dialog.field, DialogField::Title);
        for expected in [
            DialogField::DueDate,
            DialogField::Category,
            DialogField::Labels,
            DialogField::Description,
            DialogField::Save,
            DialogField::Title,
        ] {
            dialog.handle_key(KeyCode::Tab, KeyModifiers::NONE);
            assert_eq!(dialog.field, expected);
        }
    }

    /// UI-R-044
    #[test]
    fn ut_esc_returns_cancel() {
        let board = Board::new("b");
        let mut dialog = TaskDialog::blank(Status::Open, &board);
        assert_eq!(
            dialog.handle_key(KeyCode::Esc, KeyModifiers::NONE),
            DialogAction::Cancel
        );
    }

    /// UI-R-040, BD-R-010
    #[test]
    fn ut_confirm_blocked_when_title_empty() {
        let board = Board::new("b");
        let mut dialog = TaskDialog::blank(Status::Open, &board);
        for _ in 0..5 {
            dialog.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        }
        assert_eq!(dialog.field, DialogField::Save);
        assert_eq!(
            dialog.handle_key(KeyCode::Enter, KeyModifiers::NONE),
            DialogAction::None
        );
    }

    /// UI-R-040, BD-R-014
    #[test]
    fn ut_confirm_with_title_applies_and_returns_confirm() {
        let board = Board::new("b");
        let mut dialog = TaskDialog::blank(Status::Open, &board);
        for c in "Hi".chars() {
            dialog.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        for _ in 0..5 {
            dialog.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        }
        assert_eq!(
            dialog.handle_key(KeyCode::Enter, KeyModifiers::NONE),
            DialogAction::Confirm
        );
        let mut task = Task {
            id: 1,
            title: String::new(),
            description: String::new(),
            due_date: None,
            category: None,
            labels: Vec::new(),
            status: Status::Open,
        };
        dialog.apply(&mut task);
        assert_eq!(task.title, "Hi");
    }

    /// tui edge-case (see docs/specs/tui/edge-cases.md) — malformed due-date
    /// text is silently treated as no due date, not rejected.
    #[test]
    fn ut_invalid_due_date_text_is_ignored_not_rejected() {
        let board = Board::new("b");
        let mut task = Task {
            id: 1,
            title: "T".to_string(),
            description: String::new(),
            due_date: None,
            category: None,
            labels: Vec::new(),
            status: Status::Open,
        };
        let mut dialog = TaskDialog::for_task(&task, &board);
        dialog.due_date.set_input("not-a-date".to_string());
        dialog.apply(&mut task);
        assert_eq!(task.due_date, None);
    }

    /// UI-R-052, BD-R-012
    #[test]
    fn ut_blank_dialog_creates_into_given_status() {
        let board = Board::new("b");
        let dialog = TaskDialog::blank(Status::InProgress, &board);
        assert!(matches!(
            dialog.target,
            DialogTarget::Create(Status::InProgress)
        ));
        assert_eq!(dialog.title.input(), "");
    }

    /// UI-R-042
    #[test]
    fn ut_confirm_dialog_tab_toggles_yes_no() {
        let mut confirm = ConfirmDialog::for_task(1, "T".to_string());
        assert!(!confirm.yes_focused);
        confirm.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        assert!(confirm.yes_focused);
    }

    /// UI-R-042 — `No` is focused by default, so a stray `Enter` cancels
    /// rather than deletes.
    #[test]
    fn ut_confirm_dialog_enter_on_yes_confirms_on_no_cancels() {
        let mut confirm = ConfirmDialog::for_task(1, "T".to_string());
        assert_eq!(
            confirm.handle_key(KeyCode::Enter, KeyModifiers::NONE),
            ConfirmAction::Cancel
        );
        confirm.yes_focused = true;
        assert_eq!(
            confirm.handle_key(KeyCode::Enter, KeyModifiers::NONE),
            ConfirmAction::Confirm
        );
    }

    /// UI-R-042, UI-R-044
    #[test]
    fn ut_confirm_dialog_esc_always_cancels() {
        let mut confirm = ConfirmDialog::for_task(1, "T".to_string());
        assert_eq!(
            confirm.handle_key(KeyCode::Esc, KeyModifiers::NONE),
            ConfirmAction::Cancel
        );
    }

    /// UI-R-043
    #[test]
    fn ut_category_dialog_add_input_creates_category() {
        let mut board = Board::new("b");
        let mut dialog = CategoryDialog::new(&board);
        dialog.handle_key(KeyCode::Tab, KeyModifiers::NONE, &mut board);
        assert_eq!(dialog.focus, CategoryFocus::AddInput);
        for c in "urgent".chars() {
            dialog.handle_key(KeyCode::Char(c), KeyModifiers::NONE, &mut board);
        }
        let action = dialog.handle_key(KeyCode::Enter, KeyModifiers::NONE, &mut board);
        assert_eq!(action, CategoryDialogAction::Mutated);
        assert_eq!(board.categories[0].name, "urgent");
        assert_eq!(dialog.focus, CategoryFocus::List);
    }

    /// UI-R-043 — Enter on a selected table row switches to rename mode.
    #[test]
    fn ut_category_dialog_enter_on_row_renames_selected() {
        let mut board = Board::new("b");
        board.create_category("old");
        let mut dialog = CategoryDialog::new(&board);
        dialog.handle_key(KeyCode::Enter, KeyModifiers::NONE, &mut board);
        assert_eq!(dialog.focus, CategoryFocus::AddInput);
        assert_eq!(dialog.input.input(), "old");
        dialog.handle_key(KeyCode::Char('x'), KeyModifiers::NONE, &mut board);
        dialog.handle_key(KeyCode::Enter, KeyModifiers::NONE, &mut board);
        assert_eq!(board.categories[0].name, "oldx");
    }

    /// UI-R-043, BD-R-043
    #[test]
    fn ut_category_dialog_delete_removes_category() {
        let mut board = Board::new("b");
        board.create_category("cat");
        let mut dialog = CategoryDialog::new(&board);
        let action = dialog.handle_key(KeyCode::Char('d'), KeyModifiers::NONE, &mut board);
        assert_eq!(action, CategoryDialogAction::Mutated);
        assert!(board.categories.is_empty());
    }

    /// UI-R-043, UI-R-044
    #[test]
    fn ut_category_dialog_esc_closes_and_discards_pending_edit() {
        let mut board = Board::new("b");
        let mut dialog = CategoryDialog::new(&board);
        dialog.handle_key(KeyCode::Tab, KeyModifiers::NONE, &mut board);
        dialog.handle_key(KeyCode::Char('x'), KeyModifiers::NONE, &mut board);
        let action = dialog.handle_key(KeyCode::Esc, KeyModifiers::NONE, &mut board);
        assert_eq!(action, CategoryDialogAction::Close);
        assert!(board.categories.is_empty());
    }

    /// tui edge-case (see docs/specs/tui/edge-cases.md) — Tab away from the
    /// add/rename input discards uncommitted text.
    #[test]
    fn ut_category_dialog_tab_away_discards_input_text() {
        let mut board = Board::new("b");
        let mut dialog = CategoryDialog::new(&board);
        dialog.handle_key(KeyCode::Tab, KeyModifiers::NONE, &mut board);
        dialog.handle_key(KeyCode::Char('x'), KeyModifiers::NONE, &mut board);
        dialog.handle_key(KeyCode::Tab, KeyModifiers::NONE, &mut board);
        assert_eq!(dialog.focus, CategoryFocus::List);
        assert_eq!(dialog.input.input(), "");
    }
}
