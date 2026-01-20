use std::sync::LazyLock;

use ratatui::style::Color;

pub(crate) static COLOR_SCHEME: LazyLock<ColorScheme, fn() -> ColorScheme> =
    LazyLock::new(ColorScheme::load);

pub(crate) struct ColorScheme {
    pub(crate) text_fg: Color,
    pub(crate) text_bg: Color,
    pub(crate) disabled_fg: Color,
    pub(crate) disabled_bg: Color,
    pub(crate) cursor_fg: Color,
    pub(crate) cursor_bg: Color,
    pub(crate) tag_selected_bg: Color,
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
            disabled_fg: Color::Rgb(0xc0, 0xc0, 0xc0),
            disabled_bg: Color::Reset,
            cursor_fg: Color::Reset,
            cursor_bg: Color::Rgb(0xbf, 0xdb, 0xfe),
            tag_selected_bg: Color::Rgb(0xff, 0xea, 0xa2),
            lane_title_fg: Color::Reset,
            lane_title_bg: Color::Reset,
            lane_active_title_fg: Color::Reset,
            lane_active_title_bg: Color::Rgb(0xbf, 0xdb, 0xfe),
            status_bar_fg: Color::Reset,
            status_bar_bg: Color::Rgb(0xd0, 0xd0, 0xd0),
        }
    }
}
