mod card;
pub mod dialog;

use chrono::Local;
use ferrowl_ui::COLOR_SCHEME;
use ferrowl_ui::widgets::{InputField, InputFieldBuilder, ScrollingTabsBuilder};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, StatefulWidget},
};

use crate::app::App;
use crate::filter::Filter;
use crate::model::{Board, Status};

/// UI-R-002 — top-to-bottom: board tab bar, three-column board area, command line.
/// UI-R-010 — the board area is exactly three columns, Open/InProgress/Done.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame
        .buffer_mut()
        .set_style(area, Style::default().bg(COLOR_SCHEME.bg));

    // UI-R-002, UI-R-061 — the filter-status row exists only when the active
    // board has a filter; otherwise the board area reclaims its height.
    let active_filter = app.boards.get(app.active).and_then(|b| b.filter.as_ref());
    let (tabs_area, board_area, filter_area, cmd_area) = if active_filter.is_some() {
        let [tabs, board, filter, cmd] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);
        (tabs, board, Some(filter), cmd)
    } else {
        let [tabs, board, cmd] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .areas(area);
        (tabs, board, None, cmd)
    };

    let mut tabs_state = app.tabs_state();
    let tabs = ScrollingTabsBuilder::default()
        .build()
        .expect("ScrollingTabsBuilder fields all default");
    StatefulWidget::render(&tabs, tabs_area, frame.buffer_mut(), &mut tabs_state);

    let [open_area, in_progress_area, done_area] = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .areas(board_area);
    let column_areas = [open_area, in_progress_area, done_area];

    if let Some(board) = app.boards.get(app.active) {
        let today = Local::now().date_naive();
        for (status, area) in Status::ORDER.into_iter().zip(column_areas) {
            render_column(
                frame,
                area,
                status,
                board,
                today,
                app.focus.column,
                app.focus.id,
            );
        }
    }

    if let (Some(filter_area), Some(filter)) = (filter_area, active_filter) {
        render_filter_status(frame, filter_area, filter);
    }

    render_cmdline(
        frame,
        cmd_area,
        app.cmdline.as_ref(),
        app.cmdline_error.as_ref(),
    );
    if app.cmdline.is_some() {
        render_command_help(frame, area, cmd_area);
    }

    if let Some(board) = app.boards.get(app.active) {
        if let Some(d) = &app.dialog {
            dialog::render(frame, area, d, board);
        }
        if let Some(c) = &app.confirm_delete {
            dialog::render_confirm(frame, area, c);
        }
        if let Some(c) = &app.category_dialog {
            dialog::render_categories(frame, area, c);
        }
    }
}

/// UI-R-050, UI-R-051 — a one-row command-line prompt; shows the last
/// unrecognized command as a single-line error when not in command mode,
/// or a static ": command" hint when there's no error either.
fn render_cmdline(
    frame: &mut Frame,
    area: Rect,
    cmdline: Option<&ferrowl_ui::state::InputFieldState>,
    error: Option<&String>,
) {
    if let Some(field) = cmdline {
        let [prefix_a, input_a] =
            Layout::horizontal([Constraint::Length(1), Constraint::Min(0)]).areas(area);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(":").style(Style::default().fg(COLOR_SCHEME.text)),
            prefix_a,
        );
        let widget: InputField<String> = InputFieldBuilder::default()
            .border(ferrowl_ui::Border::None)
            .build()
            .expect("InputFieldBuilder: all fields defaulted");
        let mut state = field.clone();
        StatefulWidget::render(&widget, input_a, frame.buffer_mut(), &mut state);
        return;
    }
    let (text, style) = match error {
        Some(err) => (err.clone(), Style::default().fg(COLOR_SCHEME.error)),
        None => (
            ": command".to_string(),
            Style::default().fg(ratatui::style::Color::White),
        ),
    };
    frame.render_widget(ratatui::widgets::Paragraph::new(text).style(style), area);
}

/// UI-R-061 — a one-row filter-status line showing the active filter's
/// condition text, centered, white on blue, between the board and command line.
fn render_filter_status(frame: &mut Frame, area: Rect, filter: &Filter) {
    let style = Style::default()
        .bg(ratatui::style::Color::Blue)
        .fg(ratatui::style::Color::White);
    frame.buffer_mut().set_style(area, style);
    let text = format!("Filter: {}", filter.describe());
    frame.render_widget(
        ratatui::widgets::Paragraph::new(text)
            .style(style)
            .alignment(ratatui::layout::Alignment::Center),
        area,
    );
}

/// UI-R-054 — while command-line mode is active, list the available
/// commands in a popup just above the command line.
/// UI-R-055 — fill the popup with the theme background before drawing.
fn render_command_help(frame: &mut Frame, area: Rect, cmd_area: Rect) {
    let width = crate::command::HELP
        .iter()
        .map(|(syntax, effect)| syntax.len() + effect.len() + 3)
        .max()
        .unwrap_or(0) as u16
        + 2;
    let width = width.min(area.width);
    let height = crate::command::HELP.len() as u16 + 2;
    let height = height.min(cmd_area.y.saturating_sub(area.y));
    let x = area.x;
    let y = cmd_area.y.saturating_sub(height);

    let popup = Rect::new(x, y, width, height);
    frame.render_widget(ratatui::widgets::Clear, popup);
    frame
        .buffer_mut()
        .set_style(popup, Style::default().bg(COLOR_SCHEME.bg));
    let block = Block::default()
        .title("Commands")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_SCHEME.hi));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines: Vec<ratatui::text::Line> = crate::command::HELP
        .iter()
        .map(|(syntax, effect)| {
            ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(
                    format!("{syntax:<20}"),
                    Style::default().fg(COLOR_SCHEME.hi),
                ),
                ratatui::text::Span::styled(*effect, Style::default().fg(COLOR_SCHEME.text)),
            ])
        })
        .collect();
    frame.render_widget(ratatui::widgets::Paragraph::new(lines), inner);
}

/// UI-R-011 — black or white, whichever contrasts more against `bg`.
pub(super) fn contrasting_text(bg: ratatui::style::Color) -> ratatui::style::Color {
    let ratatui::style::Color::Rgb(r, g, b) = bg else {
        return ratatui::style::Color::White;
    };
    let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    if luminance > 128.0 {
        ratatui::style::Color::Black
    } else {
        ratatui::style::Color::White
    }
}

fn status_title(status: Status) -> &'static str {
    match status {
        Status::Open => "Open",
        Status::InProgress => "InProgress",
        Status::Done => "Done",
    }
}

/// UI-R-010 — one column: a titled border, then its tasks stacked top-down in
/// manual order (BD-R-030), scrolled to keep the focused card visible when
/// they don't all fit.
/// UI-R-011 — each card's height is computed from its wrapped description.
/// UI-R-023 — the focused column's border is always distinguished, card or not.
#[allow(clippy::too_many_arguments)]
fn render_column(
    frame: &mut Frame,
    area: Rect,
    status: Status,
    board: &Board,
    today: chrono::NaiveDate,
    focused_column: Status,
    focused_id: Option<u64>,
) {
    let is_focused_column = status == focused_column;
    // UI-R-060 — only cards passing the active filter are drawn; label colors
    // (UI-R-014) still range over every task via `label_colors`.
    let tasks: Vec<&crate::model::Task> = board.visible_tasks_in(status).collect();
    let label_colors = board.label_colors();

    let border_style = if is_focused_column {
        Style::default()
            .fg(COLOR_SCHEME.hi)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(ratatui::style::Color::White)
    };
    let block = Block::default()
        .title(status_title(status))
        .title_alignment(ratatui::layout::Alignment::Center)
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let heights: Vec<u16> = tasks
        .iter()
        .map(|t| card::card_height(inner.width, t))
        .collect();

    let focused_idx = if is_focused_column {
        tasks.iter().position(|t| Some(t.id) == focused_id)
    } else {
        None
    };

    // UI-R-010 — scroll so the focused card is visible: start at the focused
    // card and pull the window start upward while earlier cards still fit.
    let mut start = focused_idx.unwrap_or(0);
    let mut used: u32 = heights.get(start).copied().unwrap_or(0) as u32;
    while start > 0 {
        let candidate = heights[start - 1] as u32;
        if used + candidate > inner.height as u32 {
            break;
        }
        used += candidate;
        start -= 1;
    }

    let mut y = inner.y;
    for (task, height) in tasks.iter().zip(heights.iter()).skip(start) {
        if y + height > inner.y + inner.height {
            break;
        }
        let card_area = Rect::new(inner.x, y, inner.width, *height);
        let focused = is_focused_column && focused_id == Some(task.id);
        card::render(frame, card_area, task, board, &label_colors, today, focused);
        y += height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::focus::Focus;
    use crate::model::Board;
    use ferrowl_ui::state::InputFieldStateBuilder;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

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

    /// UI-R-054
    #[test]
    fn ut_command_help_popup_shown_only_while_cmdline_active() {
        let mut a = app();
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &a)).unwrap();
        let without: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(!without.contains("Commands"));

        a.cmdline = Some(InputFieldStateBuilder::default().build().unwrap());
        terminal.draw(|f| render(f, &a)).unwrap();
        let with: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(with.contains("Commands"));
        assert!(with.contains("new-task"));
    }

    /// UI-R-051
    #[test]
    fn ut_cmdline_error_rendered_in_error_color() {
        let mut a = app();
        a.cmdline_error = Some("Unknown command: bogus".to_string());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &a)).unwrap();
        let last_row = terminal.backend().buffer().area.height - 1;
        let cell = &terminal.backend().buffer()[(0, last_row)];
        assert_eq!(cell.fg, COLOR_SCHEME.error);
    }

    /// UI-R-014
    #[test]
    fn ut_board_render_shows_task_label() {
        let mut a = app();
        let id = a.boards[0].create_task("Fix bug", Status::Open);
        a.boards[0].task_mut(id).unwrap().labels = vec!["urgent".to_string()];
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &a)).unwrap();
        let out: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(out.contains("URGENT"));
    }

    fn rendered_text(a: &App) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, a)).unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect()
    }

    /// UI-R-061
    /// UI-R-002
    #[test]
    fn ut_filter_status_row_shown_only_when_filter_active() {
        let mut a = app();
        a.boards[0].create_task("Fix bug", Status::Open);
        assert!(!rendered_text(&a).contains("Filter:"));

        a.boards[0].filter = Filter::parse("label=bug").unwrap();
        assert!(rendered_text(&a).contains("Filter: label=bug"));
    }

    /// UI-R-061 — the filter row is white on blue and its text is centered.
    #[test]
    fn ut_filter_status_row_styled_and_centered() {
        let mut a = app();
        a.boards[0].create_task("Fix bug", Status::Open);
        a.boards[0].filter = Filter::parse("label=bug").unwrap();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &a)).unwrap();
        let buf = terminal.backend().buffer();
        // Filter row sits directly above the one-row command line.
        let row = buf.area.height - 2;

        // The whole row is white on blue.
        let cell = &buf[(0, row)];
        assert_eq!(cell.bg, ratatui::style::Color::Blue);
        assert_eq!(cell.fg, ratatui::style::Color::White);

        // Centered: the text does not start in column 0.
        let line: String = (0..buf.area.width)
            .map(|x| buf[(x, row)].symbol())
            .collect();
        assert!(line.trim() == "Filter: label=bug");
        assert!(line.starts_with(' '), "centered text is left-padded");
    }

    /// UI-R-060
    #[test]
    fn ut_filtered_out_card_not_rendered() {
        let mut a = app();
        let id = a.boards[0].create_task("Fix bug", Status::Open);
        a.boards[0].task_mut(id).unwrap().labels = vec!["bug".to_string()];
        a.boards[0].create_task("Write docs", Status::Open);

        a.boards[0].filter = Filter::parse("label=bug").unwrap();
        let out = rendered_text(&a);
        assert!(out.contains("Fix bug"));
        assert!(!out.contains("Write docs"));
    }
}
