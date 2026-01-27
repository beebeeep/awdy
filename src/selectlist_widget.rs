use ratatui::{
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, StatefulWidget, Widget},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::color_scheme::COLOR_SCHEME;

pub(crate) struct SelectListState {
    pub(crate) list_state: ListState,
    pub(crate) items: Vec<(String, bool, &'static str)>, //3rd element is symbol shown if item is not selected
}

pub(crate) struct SelectList {
    pub(crate) title: String,
    pub(crate) inactive: bool,
}

impl StatefulWidget for &SelectList {
    type State = SelectListState;

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
            let item = &state.items[context.index];
            let mut style = Style::default()
                .fg(COLOR_SCHEME.text_fg)
                .bg(COLOR_SCHEME.text_bg);
            if item.1 {
                style = style.bold();
            }
            let mut item = Text::from(format!(
                "[{}] {}",
                if item.1 { "x" } else { item.2 },
                item.0
            ))
            .style(style);
            if !self.inactive && context.is_selected {
                item.style = item
                    .style
                    .fg(COLOR_SCHEME.cursor_fg)
                    .bg(COLOR_SCHEME.cursor_bg);
            }
            (item, 1)
        });
        let list = ListView::new(builder, state.items.len());
        let mut block = Block::bordered()
            .title(self.title.clone())
            .title_alignment(ratatui::layout::Alignment::Center)
            // .borders(Borders::RIGHT | Borders::TOP)
            .border_type(BorderType::Thick);

        let mut block_title_style = Style::default()
            .bold()
            .fg(COLOR_SCHEME.lane_title_fg)
            .bg(COLOR_SCHEME.lane_title_bg);
        let block_border_style = Style::default()
            .fg(COLOR_SCHEME.text_fg)
            .bg(COLOR_SCHEME.text_bg);
        if !self.inactive {
            block_title_style = block_title_style
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
