use ferrowl_ui::COLOR_SCHEME;
use ferrowl_ui::state::InputFieldStateBuilder;
use ferrowl_ui::style::SelectionStyleBuilder;
use ferrowl_ui::traits::SetFocus;
use ferrowl_ui::widgets::{InputField, InputFieldBuilder, SelectionBuilder};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, StatefulWidget},
};

use super::contrasting_text;
use crate::dialog::{CategoryDialog, CategoryFocus, ConfirmDialog, DialogField, TaskDialog};
use crate::model::Board;

/// UI-R-041 — render the task detail dialog as a centered overlay: title,
/// due date, category, labels, description, and a save line.
pub fn render(frame: &mut Frame, area: Rect, dialog: &TaskDialog, _board: &Board) {
    let popup = centered(area, 60, 20);
    frame.render_widget(Clear, popup);
    fill_bg(frame, popup);
    let block = Block::default()
        .title("Task")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_SCHEME.hi));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let inner = inner.inner(Margin::new(2, 1));

    let [title_a, due_cat_a, labels_a, desc_a, save_a] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(6),
        Constraint::Length(1),
    ])
    .areas(inner);
    let [due_a, cat_a] =
        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(due_cat_a);

    render_field(
        frame,
        title_a,
        "Title",
        dialog.field == DialogField::Title,
        {
            let mut s = dialog.title.clone();
            s.set_focused(dialog.field == DialogField::Title);
            s
        },
    );
    render_field(
        frame,
        due_a,
        "Due date (YYYY-MM-DD)",
        dialog.field == DialogField::DueDate,
        {
            let mut s = dialog.due_date.clone();
            s.set_focused(dialog.field == DialogField::DueDate);
            s
        },
    );
    render_field(
        frame,
        labels_a,
        "Labels (comma-separated)",
        dialog.field == DialogField::Labels,
        {
            let mut s = dialog.labels.clone();
            s.set_focused(dialog.field == DialogField::Labels);
            s
        },
    );
    render_multiline(
        frame,
        desc_a,
        "Description",
        dialog.field == DialogField::Description,
        {
            let mut s = dialog.description.clone();
            s.set_focused(dialog.field == DialogField::Description);
            s
        },
    );

    let cat_focused = dialog.field == DialogField::Category;
    let mut category = dialog.category.clone();
    category.set_focused(cat_focused);
    let border_style = field_border_style(cat_focused);
    let selection = SelectionBuilder::default()
        .title(Some("Category".into()))
        .border(ferrowl_ui::Border::Full(ratatui::layout::Margin::new(1, 0)))
        .build()
        .expect("SelectionBuilder: all fields defaulted");
    let mut selection = selection;
    let mut style = selection.style().clone();
    style.border = border_style;
    selection.set_style(style);
    StatefulWidget::render(&selection, cat_a, frame.buffer_mut(), &mut category);

    let save_style = if dialog.field == DialogField::Save {
        Style::default()
            .fg(COLOR_SCHEME.hi)
            .add_modifier(ratatui::style::Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_SCHEME.text)
    };
    frame.render_widget(
        ratatui::widgets::Paragraph::new("[ Save ]")
            .style(save_style)
            .alignment(Alignment::Right),
        save_a,
    );
}

/// UI-R-056 — unfocused borders/titles render in white.
fn field_border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(COLOR_SCHEME.hi)
    } else {
        Style::default().fg(Color::White)
    }
}

fn render_field(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    focused: bool,
    mut state: ferrowl_ui::state::InputFieldState,
) {
    let mut field: InputField<String> = InputFieldBuilder::default()
        .title(Some(title.into()))
        .border(ferrowl_ui::Border::Full(ratatui::layout::Margin::new(1, 0)))
        .build()
        .expect("InputFieldBuilder: all fields defaulted");
    let mut style = field.style().clone();
    style.border = field_border_style(focused);
    style.focused = field_border_style(true);
    field.set_style(style);
    StatefulWidget::render(&field, area, frame.buffer_mut(), &mut state);
}

fn render_multiline(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    focused: bool,
    mut state: ferrowl_ui::state::InputFieldState,
) {
    let mut field: InputField<String> = InputFieldBuilder::default()
        .title(Some(title.into()))
        .border(ferrowl_ui::Border::Full(ratatui::layout::Margin::new(1, 0)))
        .multiline(true)
        .build()
        .expect("InputFieldBuilder: all fields defaulted");
    let mut style = field.style().clone();
    style.border = field_border_style(focused);
    style.focused = field_border_style(true);
    field.set_style(style);
    StatefulWidget::render(&field, area, frame.buffer_mut(), &mut state);
}

/// UI-R-042, UI-R-059 — render the yes/no delete-confirmation dialog.
pub fn render_confirm(frame: &mut Frame, area: Rect, confirm: &ConfirmDialog) {
    let popup = centered(area, 46, 8);
    frame.render_widget(Clear, popup);
    fill_bg(frame, popup);
    let dialog_title = match confirm.target {
        crate::dialog::ConfirmTarget::Task(_) => "Delete task?",
        crate::dialog::ConfirmTarget::Board => "Delete board?",
    };
    let block = Block::default()
        .title(dialog_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_SCHEME.hi));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let inner = inner.inner(Margin::new(2, 1));

    let [name_a, buttons_a] =
        Layout::vertical([Constraint::Length(3), Constraint::Length(1)]).areas(inner);

    let mut name_state = InputFieldStateBuilder::default()
        .input(confirm.title.clone())
        .cursor(confirm.title.chars().count())
        .build()
        .expect("InputFieldStateBuilder: all fields defaulted");
    name_state.set_focused(false);
    render_field(frame, name_a, "Board/Task name", false, name_state);

    let yes_style = button_style(confirm.yes_focused);
    let no_style = button_style(!confirm.yes_focused);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("[ Yes ]", yes_style),
            Span::raw("  "),
            Span::styled("[ No ]", no_style),
        ]))
        .alignment(Alignment::Right),
        buttons_a,
    );
}

fn button_style(focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(COLOR_SCHEME.hi)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_SCHEME.text)
    }
}

/// UI-R-043 — render the category-management dialog: a scrollable selection
/// list of the board's categories, plus an always-visible add/rename input
/// below it.
pub fn render_categories(frame: &mut Frame, area: Rect, dialog: &CategoryDialog) {
    let popup = centered(area, 64, 22);
    frame.render_widget(Clear, popup);
    fill_bg(frame, popup);
    let block = Block::default()
        .title("Modify Category")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_SCHEME.hi));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let inner = inner.inner(Margin::new(2, 1));

    let [list_a, input_a, help_a] = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .areas(inner);

    let selected_color = dialog
        .list
        .values()
        .get(dialog.list.selection())
        .map(|item| Color::Rgb(item.color.0, item.color.1, item.color.2));
    let mut selection_style_builder = SelectionStyleBuilder::default();
    if let Some(color) = selected_color {
        let fg = contrasting_text(color);
        selection_style_builder.focused(
            Style::default()
                .fg(fg)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        );
        selection_style_builder.general(Style::default().fg(fg).bg(color));
    }
    let selection_style = selection_style_builder
        .build()
        .expect("SelectionStyleBuilder: all fields defaulted");

    // UI-R-043 — the Selection widget shrink-wraps its border to its item
    // count, so the border is drawn manually to always fill `list_a`.
    let list_border_style = if dialog.focus == CategoryFocus::List {
        Style::default().fg(COLOR_SCHEME.hi)
    } else {
        Style::default().fg(Color::White)
    };
    let list_block = Block::default()
        .title("Category")
        .borders(Borders::ALL)
        .border_style(list_border_style);
    let list_inner = list_block.inner(list_a);
    frame.render_widget(list_block, list_a);

    let list_widget = SelectionBuilder::default()
        .border(ferrowl_ui::Border::None)
        .style(selection_style)
        .build()
        .expect("SelectionBuilder: all fields defaulted");
    let mut list_state = dialog.list.clone();
    StatefulWidget::render(
        &list_widget,
        list_inner,
        frame.buffer_mut(),
        &mut list_state,
    );

    render_field(
        frame,
        input_a,
        dialog.input_label(),
        dialog.focus == CategoryFocus::AddInput,
        {
            let mut s = dialog.input.clone();
            s.set_focused(dialog.focus == CategoryFocus::AddInput);
            s
        },
    );

    frame.render_widget(
        Paragraph::new("Enter: rename   c: recolor   d: delete   Esc: close")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center),
        help_a,
    );
}

/// UI-R-055 — fill an area with the theme background before drawing widgets in it.
fn fill_bg(frame: &mut Frame, area: Rect) {
    frame
        .buffer_mut()
        .set_style(area, Style::default().bg(COLOR_SCHEME.bg));
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
