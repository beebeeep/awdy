use ratatui::{
    layout::{Constraint, Flex, Layout},
    style::{Style, Stylize},
    widgets::{Block, Clear, Paragraph, Widget},
};

pub(crate) struct ErrorWidget {
    pub(crate) title: String,
    pub(crate) message: String,
}

impl Widget for ErrorWidget {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let lines: Vec<_> = self.message.split("\n").collect();
        let w = lines.iter().map(|l| l.len()).max().unwrap();
        let h = lines.len();
        let t = Paragraph::new(lines.join("\n")).block(
            Block::bordered()
                .title(self.title)
                .style(Style::new().black().on_blue()),
        );
        let hor = Layout::horizontal([Constraint::Max(2 + w as u16)]).flex(Flex::Center);
        let ver = Layout::vertical([Constraint::Max(2 + h as u16)]).flex(Flex::Center);
        let [area] = ver.areas(area);
        let [area] = hor.areas(area);
        Clear.render(area, buf);
        t.render(area, buf);
    }
}
