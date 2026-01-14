use ratatui::{
    buffer::Buffer,
    layout::{HorizontalAlignment, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, StatefulWidget, Widget},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::model;

#[derive(Debug)]
pub(crate) struct LaneState {
    list_state: ListState,
    tasks: Vec<String>,
}

impl LaneState {
    pub(crate) fn from_tasks<'a>(tasks: impl IntoIterator<Item = &'a model::Task>) -> Self {
        Self {
            list_state: ListState::default(),
            tasks: tasks.into_iter().map(|t| t.title.clone()).collect(),
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

pub struct App {
    state: ListState,
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
        let builder = ListBuilder::new(|context| {
            let mut item = LaneItem::new(state.tasks[context.index].clone());
            if context.index % 2 == 1 {
                item.style = Style::default().bg(Color::Reset);
            }
            if context.is_selected {
                item.style = Style::default().bg(Color::LightBlue);
            }
            (item, 1)
        });
        let list = ListView::new(builder, state.tasks.len());
        let block = Block::bordered()
            .title(self.title.clone())
            .title_alignment(HorizontalAlignment::Center)
            .style(Style::default().add_modifier(Modifier::BOLD));
        let list_area = block.inner(area);
        block.render(area, buf);
        list.render(list_area, buf, &mut state.list_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;

    #[test]
    fn t() {
        let tasks = vec![model::Task {
            state: model::TaskState::Todo,
            title: "chlos".to_string(),
            description: None,
            tags: Vec::new(),
        }];
        let l = LaneState::from_tasks(&tasks);
    }
}
