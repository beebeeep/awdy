use ratatui::{
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, BorderType, StatefulWidget, Widget},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::color_scheme::COLOR_SCHEME;

pub(crate) struct SelectListState {
    pub(crate) list_state: ListState,
    pub(crate) items: Vec<(String, bool)>,
    pub(crate) active: bool,
}

pub(crate) struct SelectList {
    pub(crate) title: String,
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
                style = style.add_modifier(Modifier::REVERSED);
            }
            let mut item = Text::from(item.0.clone()).style(style);
            if context.is_selected && state.active {
                item.style = item
                    .style
                    .fg(COLOR_SCHEME.text_selected_fg)
                    .bg(COLOR_SCHEME.text_selected_bg);
            }
            (item, 1)
        });
        let list = ListView::new(builder, state.items.len());
        let mut block = Block::bordered()
            .title(self.title.clone())
            .title_alignment(ratatui::layout::Alignment::Left);

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
