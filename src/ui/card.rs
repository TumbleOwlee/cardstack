use chrono::NaiveDate;
use ferrowl_ui::COLOR_SCHEME;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use unicode_width::UnicodeWidthStr;

use super::contrasting_text;
use crate::model::{Board, Task};

/// Non-content rows: top+bottom border, title row, footer row.
const CARD_FIXED_ROWS: u16 = 4;

/// UI-R-011 — count how many rows `text` wraps to at `width` columns
/// (greedy word-wrap, matching `Paragraph`'s `Wrap { trim: false }`).
fn wrapped_line_count(text: &str, width: u16) -> u16 {
    if width == 0 {
        return 1;
    }
    let width = width as usize;
    let mut lines: u16 = 0;
    for raw_line in text.split('\n') {
        if raw_line.is_empty() {
            lines += 1;
            continue;
        }
        let mut current = 0usize;
        let mut line_has_content = false;
        for word in raw_line.split(' ') {
            let word_width = word.width();
            let sep = if line_has_content { 1 } else { 0 };
            if line_has_content && current + sep + word_width > width {
                lines += 1;
                current = word_width;
                line_has_content = true;
            } else {
                current += sep + word_width;
                line_has_content = true;
            }
        }
        lines += 1;
    }
    lines.max(1)
}

/// UI-R-011 — a card's total height (including its border), given the
/// content width available for the wrapped description.
pub fn card_height(width: u16, task: &Task) -> u16 {
    let content_width = width.saturating_sub(4); // border (2) + horizontal margin (2)
    let desc_lines = wrapped_line_count(&task.description, content_width);
    CARD_FIXED_ROWS + desc_lines
}

/// BD-R-040, UI-R-012 — a task's card color: its category's color, or the
/// theme's default border color if it has none (BD-R-044).
fn card_color(board: &Board, task: &Task) -> Color {
    task.category
        .as_deref()
        .and_then(|name| board.categories.iter().find(|c| c.name == name))
        .map(|c| Color::Rgb(c.color.0, c.color.1, c.color.2))
        .unwrap_or(COLOR_SCHEME.border)
}

/// UI-R-013 — a due date is overdue if it's in the past and the task isn't `Done`.
fn is_overdue(task: &Task, today: NaiveDate) -> bool {
    task.due_date
        .is_some_and(|d| d < today && !matches!(task.status, crate::model::Status::Done))
}

/// UI-R-011 — render one task as a bordered card: bold title on row one,
/// wrapped description below it, category (bottom-left) and due date
/// (bottom-right) on the footer row.
/// UI-R-023 — the focused card gets a distinct border, keeping its category color.
/// UI-R-055 — fill the card's interior with the theme background before content.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    task: &Task,
    board: &Board,
    today: NaiveDate,
    focused: bool,
) {
    let color = card_color(board, task);
    let mut style = Style::default().fg(color);
    if focused {
        style = style.add_modifier(Modifier::BOLD);
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if focused {
            BorderType::Double
        } else {
            BorderType::Plain
        })
        .border_style(style);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame
        .buffer_mut()
        .set_style(inner, Style::default().bg(COLOR_SCHEME.bg));
    let inner = inner.inner(ratatui::layout::Margin::new(1, 0));

    let [title_a, desc_a, footer_a] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    frame.render_widget(
        Paragraph::new(Span::styled(
            task.title.as_str(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        title_a,
    );
    frame.render_widget(
        Paragraph::new(task.description.as_str())
            .style(Style::default().fg(COLOR_SCHEME.text))
            .wrap(Wrap { trim: false }),
        desc_a,
    );

    let due_text = task.due_date.map(|d| d.format("%Y-%m-%d").to_string());
    let due_width = due_text.as_ref().map(|s| s.len() as u16).unwrap_or(0);
    let cat_text = task.category.as_ref().map(|c| c.to_uppercase());
    let cat_width = cat_text.as_ref().map(|c| c.len() as u16).unwrap_or(0);
    let [cat_a, _, due_a] = Layout::horizontal([
        Constraint::Length(cat_width + 2),
        Constraint::Min(0),
        Constraint::Length(due_width),
    ])
    .areas(footer_a);

    if let Some(category) = &cat_text {
        let badge_style = Style::default()
            .bg(color)
            .fg(contrasting_text(color))
            .add_modifier(Modifier::BOLD);
        frame.render_widget(
            Paragraph::new(format!(" {} ", category)).style(badge_style),
            cat_a,
        );
    }
    if let Some(due) = &due_text {
        let due_style = if is_overdue(task, today) {
            Style::default().fg(COLOR_SCHEME.error)
        } else {
            Style::default().fg(COLOR_SCHEME.text)
        };
        frame.render_widget(Paragraph::new(due.as_str()).style(due_style), due_a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Status;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn task(title: &str) -> Task {
        Task {
            id: 0,
            title: title.to_string(),
            description: String::new(),
            due_date: None,
            category: None,
            labels: Vec::new(),
            status: Status::Open,
        }
    }

    fn render_to_string(task: &Task, board: &Board, today: NaiveDate) -> String {
        let backend = TestBackend::new(30, card_height(30, task));
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render(f, f.area(), task, board, today, false))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect()
    }

    /// UI-R-011
    #[test]
    fn ut_card_shows_title() {
        let board = Board::new("b");
        let t = task("Write tests");
        let out = render_to_string(&t, &board, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert!(out.contains("Write"));
    }

    /// UI-R-013
    #[test]
    fn ut_overdue_due_date_flagged() {
        let mut t = task("Late");
        t.due_date = Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        let today = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert!(is_overdue(&t, today));

        t.status = Status::Done;
        assert!(!is_overdue(&t, today), "Done tasks are never overdue");
    }

    /// UI-R-012, BD-R-044
    #[test]
    fn ut_card_color_falls_back_without_category() {
        let board = Board::new("b");
        let t = task("No category");
        assert_eq!(card_color(&board, &t), COLOR_SCHEME.border);
    }

    /// UI-R-011
    #[test]
    fn ut_contrasting_text_picks_higher_contrast() {
        assert_eq!(contrasting_text(Color::Rgb(255, 255, 255)), Color::Black);
        assert_eq!(contrasting_text(Color::Rgb(0, 0, 0)), Color::White);
    }

    /// UI-R-012
    #[test]
    fn ut_card_color_uses_category_color() {
        let mut board = Board::new("b");
        board.create_category("urgent");
        let mut t = task("Has category");
        t.category = Some("urgent".to_string());
        let expected = board.categories[0].color;
        assert_eq!(
            card_color(&board, &t),
            Color::Rgb(expected.0, expected.1, expected.2)
        );
    }
}
