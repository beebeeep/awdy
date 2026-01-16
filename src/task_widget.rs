use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Clear, Widget},
};
use tui_textarea::TextArea;

use crate::{model::Task, util::centered_rect};

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
        Line::from(self.label.to_string()).render(layout[0], buf);
        self.edit.render(layout[1], buf);
    }
}

#[derive(Debug)]
pub(crate) struct TaskView<'a> {
    pub(crate) task_id: Option<u64>,
    pub(crate) text_areas: Vec<TextArea<'a>>,
    pub(crate) active_text_area: usize,
}

impl<'a> TaskView<'a> {
    const TITLE: usize = 0;
    const DESCRIPTION: usize = 1;
    const TAGS: usize = 2;
    pub(crate) fn from_task(task: Task) -> Self {
        let title_area = TextArea::new(vec![task.title]);
        let mut description_area = TextArea::new(
            task.description
                .map_or_else(Vec::new, |d| d.split("\n").map(String::from).collect()),
        );
        description_area.set_block(Block::bordered().title("Description"));
        description_area.set_cursor_line_style(Style::default());
        let mut tags_area = TextArea::new(vec![task.tags.join(", ")]);
        tags_area.set_cursor_line_style(Style::default());
        Self {
            task_id: task.id,
            text_areas: vec![title_area, description_area, tags_area],
            active_text_area: 0,
        }
    }
}

impl Widget for &TaskView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let window_area = centered_rect(area, 80, 80);
        let block = Block::bordered().title(
            self.task_id
                .map_or_else(|| "New task".to_string(), |id| format!("Task #{id}")),
        );
        let task_area = block.inner(window_area).inner(Margin::new(2, 0));
        let layout = Layout::vertical(vec![
            Constraint::Max(1),
            Constraint::Fill(1),
            Constraint::Max(1),
        ])
        .split(task_area);

        Clear.render(window_area, buf);
        block.render(window_area, buf);
        LabeledEdit::new("Title:", &self.text_areas[TaskView::TITLE]).render(layout[0], buf);
        self.text_areas[1].render(layout[TaskView::DESCRIPTION], buf);
        LabeledEdit::new("Tags:", &self.text_areas[TaskView::TAGS]).render(layout[2], buf);
    }
}
