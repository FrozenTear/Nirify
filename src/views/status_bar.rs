//! Status bar component showing save status and other info

use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length, Theme};

use crate::messages::Message;
use crate::theme::{status_bar_style, AppTheme, muted_text_container, secondary_text_container};

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
    // Status indicator - uses theme's warning/success colors
    let status_text = if dirty {
        "● Unsaved changes"
    } else {
        "✓ Changes saved automatically"
    };

    let status = container(text(status_text).size(12))
        .style(move |theme: &Theme| {
            let color = if dirty {
                theme.palette().warning
            } else {
                theme.palette().success
            };
            container::Style {
                text_color: Some(color),
                ..Default::default()
            }
        });

    let status_row = row![status]
        .spacing(8)
        .align_y(Alignment::Center);

    // Niri connection status - uses theme's success/danger colors
    let (niri_icon, niri_text) = match niri_status {
        NiriStatus::Connected => ("⬤", "niri"),
        NiriStatus::Disconnected => ("○", "niri (not running)"),
        NiriStatus::Unknown => ("◌", "niri (checking...)"),
    };

    let niri_indicator = container(
        row![
            text(niri_icon).size(10),
            text(niri_text).size(12),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
    )
    .style(move |theme: &Theme| {
        let color = match niri_status {
            NiriStatus::Connected => theme.palette().success,
            NiriStatus::Disconnected => theme.palette().danger,
            NiriStatus::Unknown => {
                let txt = theme.palette().text;
                iced::Color { a: 0.5, ..txt }
            }
        };
        container::Style {
            text_color: Some(color),
            ..Default::default()
        }
    });

    // Optional save status message (e.g., "Saved 3 files")
    let mut content = row![status_row, niri_indicator].spacing(16).padding([8, 20]);

    if let Some(ref message) = save_status {
        content = content.push(container(text("•").size(12)).style(muted_text_container));
        content = content.push(container(text(message.clone()).size(12)).style(secondary_text_container));
    }

    // Theme selector - cycles through available themes
    let theme_button = button(
        row![
            container(text("◐").size(14)).style(muted_text_container),
            container(text(current_theme.name()).size(12)).style(secondary_text_container),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([4, 10])
    .style(theme_button_style)
    .on_press(Message::ChangeTheme(next_theme(current_theme)));

    // App info and theme selector on the right
    let right_section = row![
        container(text(format!("niri settings v{}", env!("CARGO_PKG_VERSION"))).size(12)).style(muted_text_container),
        container(text("•").size(12)).style(muted_text_container),
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
/// Uses theme colors for consistent appearance across themes.
fn theme_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    use iced::{Border, Color, Shadow};
    let palette = theme.palette();
    let bg_base = palette.background;

    // Derive surface colors from background
    let bg_surface_hover = Color { r: bg_base.r + 0.08, g: bg_base.g + 0.08, b: bg_base.b + 0.08, a: 1.0 };
    let bg_surface = Color { r: bg_base.r + 0.05, g: bg_base.g + 0.05, b: bg_base.b + 0.05, a: 1.0 };
    let border_subtle = Color { r: bg_base.r + 0.12, g: bg_base.g + 0.12, b: bg_base.b + 0.12, a: 1.0 };
    let border_strong = Color { r: bg_base.r + 0.18, g: bg_base.g + 0.18, b: bg_base.b + 0.18, a: 1.0 };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(bg_surface_hover)),
            text_color: palette.text,
            border: Border {
                color: border_subtle,
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(bg_surface)),
            text_color: palette.text,
            border: Border {
                color: border_strong,
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        _ => button::Style {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            text_color: Color { a: 0.7, ..palette.text },
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
