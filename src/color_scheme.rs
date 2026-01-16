use std::sync::LazyLock;

use ratatui::style::Color;

pub(crate) static COLOR_SCHEME: LazyLock<ColorScheme, fn() -> ColorScheme> =
    LazyLock::new(ColorScheme::load);

pub(crate) struct ColorScheme {
    pub(crate) text_fg: Color,
    pub(crate) text_bg: Color,
    pub(crate) text_alt_bg: Color,
    pub(crate) text_selected_fg: Color,
    pub(crate) text_selected_bg: Color,
    pub(crate) lane_title_fg: Color,
    pub(crate) lane_title_bg: Color,
    pub(crate) lane_active_title_fg: Color,
    pub(crate) lane_active_title_bg: Color,
    pub(crate) status_bar_bg: Color,
    pub(crate) status_bar_fg: Color,
}

impl ColorScheme {
    fn load() -> Self {
        Self {
            text_fg: Color::Reset,
            text_bg: Color::Reset,
            text_alt_bg: Color::Reset, //Color::Rgb(0xf0, 0xf0, 0xf0),
            text_selected_fg: Color::Reset,
            text_selected_bg: Color::Rgb(0xbf, 0xdb, 0xfe),
            lane_title_fg: Color::Reset,
            lane_title_bg: Color::Reset,
            lane_active_title_fg: Color::Reset,
            lane_active_title_bg: Color::Rgb(0xbf, 0xdb, 0xfe),
            status_bar_fg: Color::Reset,
            status_bar_bg: Color::Rgb(0xd0, 0xd0, 0xd0),
        }
    }
}
