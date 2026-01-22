//! Status bar component showing save status and other info

use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length};

use crate::messages::Message;
use crate::theme::{status_bar_style, AppTheme, NiriColors};

/// Niri connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NiriStatus {
    #[default]
    Unknown,
    Connected,
    Disconnected,
}

/// Creates the status bar at the bottom of the window
pub fn view(
    dirty: bool,
    save_status: Option<String>,
    current_theme: AppTheme,
    niri_status: NiriStatus,
) -> Element<'static, Message> {
    let colors = NiriColors::default();

    // Status indicator
    let status_text = if dirty {
        "● Unsaved changes"
    } else {
        "✓ Changes saved automatically"
    };

    let status_color = if dirty {
        colors.warning
    } else {
        colors.success
    };

    let status = row![text(status_text).size(12).color(status_color),]
        .spacing(8)
        .align_y(Alignment::Center);

    // Niri connection status
    let (niri_icon, niri_text, niri_color) = match niri_status {
        NiriStatus::Connected => ("⬤", "niri", colors.success),
        NiriStatus::Disconnected => ("○", "niri (not running)", colors.error),
        NiriStatus::Unknown => ("◌", "niri (checking...)", colors.text_tertiary),
    };

    let niri_indicator = row![
        text(niri_icon).size(10).color(niri_color),
        text(niri_text).size(12).color(niri_color),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    // Optional save status message (e.g., "Saved 3 files")
    let mut content = row![status, niri_indicator].spacing(16).padding([8, 20]);

    if let Some(ref message) = save_status {
        content = content.push(text("•").size(12).color(colors.text_tertiary));
        content = content.push(text(message.clone()).size(12).color(colors.text_secondary));
    }

    // Theme selector - cycles through available themes
    let theme_button = button(
        row![
            text("◐").size(14).color(colors.text_tertiary),
            text(current_theme.name())
                .size(12)
                .color(colors.text_secondary)
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([4, 10])
    .style(theme_button_style)
    .on_press(Message::ChangeTheme(next_theme(current_theme)));

    // App info and theme selector on the right
    let right_section = row![
        text(format!("niri settings v{}", env!("CARGO_PKG_VERSION")))
            .size(12)
            .color(colors.text_tertiary),
        text("•").size(12).color(colors.text_tertiary),
        theme_button,
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let right_container = container(right_section)
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right);

    let content = row![content, right_container].align_y(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .style(status_bar_style)
        .into()
}

/// Returns the next theme in the cycle
fn next_theme(current: AppTheme) -> AppTheme {
    let themes = AppTheme::all();
    let current_idx = themes.iter().position(|&t| t == current).unwrap_or(0);
    let next_idx = (current_idx + 1) % themes.len();
    themes[next_idx]
}

/// Custom button style for theme selector
fn theme_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    use iced::{Border, Color, Shadow};
    let colors = NiriColors::default();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(colors.bg_surface_hover)),
            text_color: colors.text_primary,
            border: Border {
                color: colors.border_subtle,
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(colors.bg_surface)),
            text_color: colors.text_primary,
            border: Border {
                color: colors.border_strong,
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        _ => button::Style {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            text_color: colors.text_secondary,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}
