use std::{cell::RefCell, rc::Rc};

use ratatui::{
    buffer::Buffer,
    layout::{HorizontalAlignment, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, StatefulWidget, Widget},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::{color_scheme::COLOR_SCHEME, model::Task};

#[derive(Debug)]
pub(crate) struct LaneState {
    pub(crate) list_state: ListState,
    pub(crate) active: bool,
    pub(crate) tasks: Rc<RefCell<Vec<Task>>>,
}

impl LaneState {
    pub(crate) fn new<'a>(active: bool, tasks: Rc<RefCell<Vec<Task>>>) -> Self {
        Self {
            active,
            list_state: ListState::default(),
            tasks,
        }
    }
}

pub struct LaneItem {
    text: String,
    style: Style,
}

impl LaneItem {
    pub fn new<T: Into<String>>(text: T) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
        }
    }
}

impl Widget for LaneItem {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Line::from(self.text).style(self.style).render(area, buf);
    }
}

pub(crate) struct LaneWidget {
    pub(crate) title: String,
}

impl StatefulWidget for &LaneWidget {
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
        let tasks = state.tasks.borrow();
        let builder = ListBuilder::new(|context| {
            let mut item = LaneItem::new(tasks[context.index].title.clone());
            item.style = Style::default()
                .fg(COLOR_SCHEME.text_fg)
                .bg(COLOR_SCHEME.text_bg);
            if context.index % 2 == 1 {
                item.style = item.style.bg(COLOR_SCHEME.text_alt_bg);
            }
            if context.is_selected && state.active {
                item.style = item
                    .style
                    .fg(COLOR_SCHEME.text_selected_fg)
                    .bg(COLOR_SCHEME.text_selected_bg);
            }
            (item, 1)
        });
        let list = ListView::new(builder, tasks.len());
        let mut block = Block::bordered()
            .title(self.title.clone())
            .title_alignment(HorizontalAlignment::Center);

        let mut block_title_style = Style::default()
            .fg(COLOR_SCHEME.lane_title_fg)
            .bg(COLOR_SCHEME.lane_title_bg);
        if state.active {
            block = block.border_type(BorderType::Double);
            block_title_style = block_title_style
                .add_modifier(Modifier::BOLD)
                .fg(COLOR_SCHEME.lane_active_title_fg)
                .bg(COLOR_SCHEME.lane_active_title_bg);
        }
        block = block.title_style(block_title_style);
        let list_area = block.inner(area);
        block.render(area, buf);
        list.render(list_area, buf, &mut state.list_state);
    }
}
