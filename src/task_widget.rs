use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Margin, Rect},
    style::{Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Widget},
};
use tui_textarea::{CursorMove, TextArea};

use crate::{
    model::{Task, TaskState},
    util::{centered_rect, is_newline},
};

struct LabeledEdit<'a, 'b> {
    label: &'a str,
    edit: &'b TextArea<'a>,
}

impl<'a> LabeledEdit<'a, '_> {
    fn new(label: &'a str, edit: &'a TextArea) -> Self {
        Self { label, edit }
    }
}

impl Widget for &LabeledEdit<'_, '_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal(vec![
            Constraint::Length(self.label.len() as u16),
            Constraint::Fill(1),
        ])
        .spacing(2)
        .split(area);
        Line::from(self.label.to_string())
            .style(Style::default().bold())
            .render(layout[0], buf);
        self.edit.render(layout[1], buf);
    }
}

#[derive(Debug)]
pub(crate) struct TaskView<'a> {
    pub(crate) task_id: Option<u64>,
    task_state: TaskState,
    pub(crate) text_areas: Vec<TextArea<'a>>,
    pub(crate) active_text_area: usize,
}

impl<'a> TaskView<'a> {
    const TITLE: usize = 0;
    const DESCRIPTION: usize = 1;
    const TAGS: usize = 2;

    fn on_focus_change(&mut self) {
        for (i, tv) in self.text_areas.iter_mut().enumerate() {
            if i == self.active_text_area {
                tv.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
            } else {
                tv.set_cursor_style(Style::default());
            }
        }
    }
    pub(crate) fn next_field(&mut self) {
        self.active_text_area = (self.active_text_area + 1) % self.text_areas.len();
        self.on_focus_change();
    }
    pub(crate) fn prev_field(&mut self) {
        self.active_text_area =
            (self.active_text_area + self.text_areas.len() - 1) % self.text_areas.len();
        self.on_focus_change();
    }
    pub(crate) fn process_event(&mut self, event: KeyEvent) {
        if self.active_text_area == Self::TITLE && is_newline(event) {
            self.next_field();
        }
        self.text_areas[self.active_text_area].input(event);
    }
}

impl Into<Task> for TaskView<'_> {
    fn into(self) -> Task {
        let description = self.text_areas[TaskView::DESCRIPTION].lines().join("\n");
        let description = description.trim().to_string();
        let tags = self.text_areas[TaskView::TAGS]
            .lines()
            .join("\n")
            .split(",")
            .map(|v| v.trim().to_string())
            .collect();
        Task {
            id: self.task_id,
            state: self.task_state,
            title: self.text_areas[TaskView::TITLE].lines().join("\n"),
            description: if description.len() == 0 {
                None
            } else {
                Some(description.to_string())
            },
            tags,
        }
    }
}

impl From<Task> for TaskView<'_> {
    fn from(task: Task) -> Self {
        let mut title_area = TextArea::new(vec![task.title]);
        let mut description_area = TextArea::new(
            task.description
                .map_or_else(Vec::new, |d| d.split("\n").map(String::from).collect()),
        );
        let mut tags_area = TextArea::new(vec![task.tags.join(", ")]);

        title_area.set_cursor_line_style(Style::default());
        title_area.move_cursor(CursorMove::End);
        description_area.set_cursor_line_style(Style::default());
        description_area.move_cursor(CursorMove::Bottom);
        description_area.move_cursor(CursorMove::End);
        tags_area.set_cursor_line_style(Style::default());
        tags_area.move_cursor(CursorMove::End);
        description_area.set_block(
            Block::bordered()
                .title("Description")
                .title_style(Style::default().bold()),
        );

        let mut r = Self {
            task_id: task.id,
            task_state: task.state,
            text_areas: vec![title_area, description_area, tags_area],
            active_text_area: 0,
        };
        r.on_focus_change();
        r
    }
}

impl Widget for &TaskView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let window_area = centered_rect(area, 80, 80);
        let block = Block::bordered()
            .title(
                self.task_id
                    .map_or_else(|| "New task".to_string(), |id| format!("Task #{id}")),
            )
            .border_type(BorderType::Double);
        let task_area = block.inner(window_area).inner(Margin::new(2, 0));
        let layout = Layout::vertical(vec![
            Constraint::Max(1),
            Constraint::Fill(1),
            Constraint::Max(1),
        ])
        .spacing(1)
        .split(task_area);

        Clear.render(window_area, buf);
        block.render(window_area, buf);
        LabeledEdit::new("Title:", &self.text_areas[TaskView::TITLE]).render(layout[0], buf);
        self.text_areas[TaskView::DESCRIPTION].render(layout[1], buf);
        LabeledEdit::new("Tags:", &self.text_areas[TaskView::TAGS]).render(layout[2], buf);
    }
}
