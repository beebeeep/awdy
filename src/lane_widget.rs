use std::{cell::RefCell, rc::Rc};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, BorderType, StatefulWidget, Widget},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::{color_scheme::COLOR_SCHEME, model::TaskMeta};

pub(crate) struct LaneState {
    pub(crate) list_state: ListState,
    pub(crate) selected: bool,
    pub(crate) tasks: Rc<RefCell<Vec<TaskMeta>>>,
}

impl LaneState {
    pub(crate) fn new(active: bool, tasks: Rc<RefCell<Vec<TaskMeta>>>) -> Self {
        Self {
            selected: active,
            list_state: ListState::default(),
            tasks,
        }
    }
}

pub(crate) struct LaneWidget {
    pub(crate) title: String,
    pub(crate) inactive: bool,
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
            let mut item = Text::from(tasks[context.index].title.clone());
            item.style = Style::default()
                .fg(COLOR_SCHEME.text_fg)
                .bg(COLOR_SCHEME.text_bg);
            if context.is_selected && state.selected && !self.inactive {
                item.style = item
                    .style
                    .fg(COLOR_SCHEME.cursor_fg)
                    .bg(COLOR_SCHEME.cursor_bg);
            }
            (item, 1)
        });
        let list = ListView::new(builder, tasks.len());
        let mut block = Block::bordered()
            .title(self.title.clone())
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
