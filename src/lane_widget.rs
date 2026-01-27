use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, StatefulWidget, Widget},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::{color_scheme::COLOR_SCHEME, model::Task};

struct LaneItem {
    title: String,
    tags: String,
    style: Style,
}

impl Widget for LaneItem {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let areas = Layout::horizontal([
            Constraint::Min(3),
            Constraint::Length(self.tags.len() as u16),
        ])
        .split(area);
        if (areas[0].width as usize) < self.title.len() {
            self.title.truncate(areas[0].width as usize - 1);
            self.title.push('>');
        }
        Line::styled(self.title, self.style).render(areas[0], buf);
        Line::styled(self.tags, self.style.bold())
            .right_aligned()
            .render(areas[1], buf);
    }
}

pub(crate) struct LaneState {
    pub(crate) list_state: ListState,
    pub(crate) selected: bool,
}

impl LaneState {
    pub(crate) fn new() -> Self {
        Self {
            selected: false,
            list_state: ListState::default(),
        }
    }
}

pub(crate) struct LaneWidget<'a> {
    pub(crate) title: &'a str,
    pub(crate) tasks: &'a [Task],
    pub(crate) inactive: bool,
}

impl<'a> StatefulWidget for &LaneWidget<'a> {
    type State = LaneState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        if state.list_state.selected.is_none() {
            state.list_state.next();
        }
        let builder = ListBuilder::new(|context| {
            let task = &self.tasks[context.index];
            let mut style = Style::default()
                .fg(COLOR_SCHEME.text_fg)
                .bg(COLOR_SCHEME.text_bg);
            if context.is_selected && state.selected && !self.inactive {
                style = style.fg(COLOR_SCHEME.cursor_fg).bg(COLOR_SCHEME.cursor_bg);
            }
            let item = LaneItem {
                title: task.title.clone(),
                tags: format!("[{}]", task.tags.join(", ")),
                style,
            };
            (item, 1)
        });
        let list = ListView::new(builder, self.tasks.len());
        let mut block = Block::bordered()
            .title(String::from(self.title))
            .title_alignment(ratatui::layout::Alignment::Center);

        let block_border_style = Style::default()
            .fg(COLOR_SCHEME.text_fg)
            .bg(COLOR_SCHEME.text_bg);
        let mut block_title_style = Style::default()
            .fg(COLOR_SCHEME.lane_title_fg)
            .bg(COLOR_SCHEME.lane_title_bg);
        if state.selected && !self.inactive {
            block = block.border_type(BorderType::Double);
            block_title_style = block_title_style
                .add_modifier(Modifier::BOLD)
                .fg(COLOR_SCHEME.lane_active_title_fg)
                .bg(COLOR_SCHEME.lane_active_title_bg);
        }
        block = block
            .title_style(block_title_style)
            .border_style(block_border_style);
        let list_area = block.inner(area);
        block.render(area, buf);
        list.render(list_area, buf, &mut state.list_state);
    }
}
