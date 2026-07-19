use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// BD-R-011 — a task's status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Open,
    InProgress,
    Done,
}

// Not yet called from `main` — wired in as the app/command layer lands
// (Stages 2, 4, 5, 6).
#[allow(dead_code)]
impl Status {
    /// BD-R-020 — fixed column order.
    pub const ORDER: [Status; 3] = [Status::Open, Status::InProgress, Status::Done];

    pub fn index(self) -> usize {
        Self::ORDER.iter().position(|s| *s == self).unwrap()
    }

    /// BD-R-020 — one column right, or `None` past `Done`.
    pub fn right(self) -> Option<Status> {
        Self::ORDER.get(self.index() + 1).copied()
    }

    /// BD-R-020 — one column left, or `None` before `Open`.
    pub fn left(self) -> Option<Status> {
        self.index().checked_sub(1).map(|i| Self::ORDER[i])
    }
}

/// BD-R-040 — a user-defined category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    /// RGB.
    pub color: (u8, u8, u8),
}

/// BD-R-010 — a single task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub due_date: Option<NaiveDate>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    pub status: Status,
}

/// BD-R-001 — a board: a name, its categories, and its tasks in manual order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub name: String,
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(default)]
    pub tasks: Vec<Task>,
    #[serde(default)]
    next_id: u64,
}

/// BD-R-041 — palette categories are auto-assigned from, cycling once exhausted.
const CATEGORY_PALETTE: &[(u8, u8, u8)] = &[
    (243, 139, 168), // red
    (166, 227, 161), // green
    (137, 180, 250), // blue
    (250, 179, 135), // peach
    (203, 166, 247), // mauve
    (249, 226, 175), // yellow
    (148, 226, 213), // teal
    (245, 194, 231), // pink
];

// Mutating/creating methods aren't called yet — wired in as the command layer
// lands (Stages 4, 5, 6). `tasks_in` is used by rendering (Stage 3).
#[allow(dead_code)]
impl Board {
    pub fn new(name: impl Into<String>) -> Self {
        Board {
            name: name.into(),
            categories: Vec::new(),
            tasks: Vec::new(),
            next_id: 0,
        }
    }

    /// BD-R-012, BD-R-030 — create a task, defaulting its status, appended to
    /// the bottom of its column.
    pub fn create_task(&mut self, title: impl Into<String>, status: Status) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.push(Task {
            id,
            title: title.into(),
            description: String::new(),
            due_date: None,
            category: None,
            labels: Vec::new(),
            status,
        });
        id
    }

    /// BD-R-013.
    pub fn delete_task(&mut self, id: u64) {
        self.tasks.retain(|t| t.id != id);
    }

    fn task_index(&self, id: u64) -> Option<usize> {
        self.tasks.iter().position(|t| t.id == id)
    }

    /// BD-R-014 — look up a task by id for display/editing.
    pub fn task(&self, id: u64) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// BD-R-014 — look up a task by id for in-place editing.
    pub fn task_mut(&mut self, id: u64) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    /// BD-R-020, BD-R-021 — move a task's status, appending it to the bottom
    /// of the target column. No-op if `new_status` has no effect (unknown id).
    pub fn move_status(&mut self, id: u64, new_status: Status) {
        if let Some(idx) = self.task_index(id) {
            let mut task = self.tasks.remove(idx);
            task.status = new_status;
            self.tasks.push(task);
        }
    }

    /// BD-R-031 — swap a task with its same-status neighbor. `forward` moves
    /// it down/later; no-op at either edge of its status group.
    pub fn reorder(&mut self, id: u64, forward: bool) {
        let Some(idx) = self.task_index(id) else {
            return;
        };
        let status = self.tasks[idx].status;
        let same_status = |i: usize, tasks: &[Task]| tasks[i].status == status;

        let neighbor = if forward {
            (idx + 1..self.tasks.len()).find(|&i| same_status(i, &self.tasks))
        } else {
            (0..idx).rev().find(|&i| same_status(i, &self.tasks))
        };
        if let Some(n) = neighbor {
            self.tasks.swap(idx, n);
        }
    }

    /// BD-R-041 — create a category with the next unused palette color.
    pub fn create_category(&mut self, name: impl Into<String>) {
        let color = CATEGORY_PALETTE[self.categories.len() % CATEGORY_PALETTE.len()];
        self.categories.push(Category {
            name: name.into(),
            color,
        });
    }

    /// BD-R-043 — delete a category, clearing the reference on every task
    /// that had it (never deletes a task).
    pub fn delete_category(&mut self, name: &str) {
        self.categories.retain(|c| c.name != name);
        for task in &mut self.tasks {
            if task.category.as_deref() == Some(name) {
                task.category = None;
            }
        }
    }

    /// BD-R-042 — rename a category; every task referencing it follows the
    /// rename. No-op if `new` collides with a different existing category
    /// (BD-R-040: names are unique within a board).
    pub fn rename_category(&mut self, old: &str, new: impl Into<String>) {
        let new = new.into();
        if new == old || self.categories.iter().any(|c| c.name == new) {
            return;
        }
        let Some(cat) = self.categories.iter_mut().find(|c| c.name == old) else {
            return;
        };
        cat.name = new.clone();
        for task in &mut self.tasks {
            if task.category.as_deref() == Some(old) {
                task.category = Some(new.clone());
            }
        }
    }

    /// BD-R-041, BD-R-042 — advance a category to the next palette color.
    pub fn cycle_category_color(&mut self, name: &str) {
        if let Some(cat) = self.categories.iter_mut().find(|c| c.name == name) {
            let idx = CATEGORY_PALETTE
                .iter()
                .position(|&c| c == cat.color)
                .unwrap_or(0);
            cat.color = CATEGORY_PALETTE[(idx + 1) % CATEGORY_PALETTE.len()];
        }
    }

    /// BD-R-010 — tasks with the given status, in manual order (BD-R-030).
    pub fn tasks_in(&self, status: Status) -> impl Iterator<Item = &Task> {
        self.tasks.iter().filter(move |t| t.status == status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BD-R-012
    /// BD-R-030
    #[test]
    fn ut_create_task_defaults_status_and_appends() {
        let mut b = Board::new("b");
        let id = b.create_task("first", Status::InProgress);
        assert_eq!(b.tasks[0].id, id);
        assert_eq!(b.tasks[0].status, Status::InProgress);
        b.create_task("second", Status::InProgress);
        assert_eq!(b.tasks[1].title, "second");
    }

    /// BD-R-013
    #[test]
    fn ut_delete_task_removes_it() {
        let mut b = Board::new("b");
        let id = b.create_task("t", Status::Open);
        b.delete_task(id);
        assert!(b.tasks.is_empty());
    }

    /// BD-R-020
    #[test]
    fn ut_status_left_right_edges_are_none() {
        assert_eq!(Status::Open.left(), None);
        assert_eq!(Status::Open.right(), Some(Status::InProgress));
        assert_eq!(Status::Done.right(), None);
        assert_eq!(Status::Done.left(), Some(Status::InProgress));
    }

    /// BD-R-021
    #[test]
    fn ut_move_status_appends_to_bottom_of_target() {
        let mut b = Board::new("b");
        b.create_task("a", Status::Open);
        let id = b.create_task("b", Status::Open);
        b.create_task("c", Status::InProgress);
        b.move_status(id, Status::InProgress);
        let in_progress: Vec<_> = b.tasks_in(Status::InProgress).map(|t| &t.title).collect();
        assert_eq!(in_progress, vec!["c", "b"]);
    }

    /// BD-R-031
    #[test]
    fn ut_reorder_swaps_same_status_neighbor_only() {
        let mut b = Board::new("b");
        let a = b.create_task("a", Status::Open);
        b.create_task("x", Status::Done);
        let c = b.create_task("c", Status::Open);
        // a, x, c — reorder c forward is a no-op (no later Open task);
        // reorder c backward swaps with a (x is a different status, skipped).
        b.reorder(c, true);
        assert_eq!(
            b.tasks_in(Status::Open).map(|t| t.id).collect::<Vec<_>>(),
            vec![a, c]
        );
        b.reorder(c, false);
        assert_eq!(
            b.tasks_in(Status::Open).map(|t| t.id).collect::<Vec<_>>(),
            vec![c, a]
        );
    }

    /// BD-R-041
    #[test]
    fn ut_create_category_assigns_palette_color() {
        let mut b = Board::new("b");
        b.create_category("red-ish");
        b.create_category("green-ish");
        assert_ne!(b.categories[0].color, b.categories[1].color);
    }

    /// BD-R-043
    #[test]
    fn ut_delete_category_clears_task_reference_not_task() {
        let mut b = Board::new("b");
        b.create_category("cat");
        let id = b.create_task("t", Status::Open);
        b.tasks[0].category = Some("cat".to_string());
        b.delete_category("cat");
        assert!(b.categories.is_empty());
        assert!(b.tasks.iter().any(|t| t.id == id));
        assert_eq!(b.tasks[0].category, None);
    }

    /// BD-R-042
    #[test]
    fn ut_rename_category_updates_task_references() {
        let mut b = Board::new("b");
        b.create_category("cat");
        b.create_task("t", Status::Open);
        b.tasks[0].category = Some("cat".to_string());
        b.rename_category("cat", "renamed");
        assert_eq!(b.categories[0].name, "renamed");
        assert_eq!(b.tasks[0].category, Some("renamed".to_string()));
    }

    /// BD-R-042, BD-R-040
    #[test]
    fn ut_rename_category_noop_on_name_collision() {
        let mut b = Board::new("b");
        b.create_category("a");
        b.create_category("b");
        b.rename_category("a", "b");
        assert_eq!(b.categories[0].name, "a");
    }

    /// BD-R-042
    #[test]
    fn ut_cycle_category_color_advances_through_palette() {
        let mut b = Board::new("b");
        b.create_category("cat");
        let first = b.categories[0].color;
        b.cycle_category_color("cat");
        assert_ne!(b.categories[0].color, first);
    }
}
