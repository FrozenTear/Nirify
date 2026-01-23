//! Recent windows settings view
//!
//! Shows settings for the window switcher overlay (Alt-Tab) with editing capabilities.

use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, toggler};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::config::models::{RecentWindowsSettings, RecentWindowsScope};
use crate::messages::{Message, RecentWindowsMessage};
use crate::theme::fonts;

/// Creates the recent windows settings view
pub fn view(settings: &RecentWindowsSettings) -> Element<'static, Message> {
    // Clone values for closures
    let off = settings.off;
    let debounce_ms = settings.debounce_ms;
    let open_delay_ms = settings.open_delay_ms;
    let active_color = settings.highlight.active_color.to_hex();
    let urgent_color = settings.highlight.urgent_color.to_hex();
    let padding = settings.highlight.padding;
    let corner_radius = settings.highlight.corner_radius;
    let max_height = settings.previews.max_height;
    let max_scale = settings.previews.max_scale;
    let binds = settings.binds.clone();

    let mut content = column![
        page_title("Recent Windows Switcher"),
        info_text(
            "Configure the window switcher overlay for switching between recently used windows (Alt-Tab)."
        ),
        spacer(16.0),

        // Status
        subsection_header("Status"),
        toggle_setting("Disable Switcher", off, RecentWindowsMessage::SetOff),
        info_text("When enabled, the recent windows switcher will be completely disabled."),
        spacer(16.0),

        // Timing
        subsection_header("Timing"),
        slider_setting("Debounce Delay", debounce_ms, 0, 2000, "ms", RecentWindowsMessage::SetDebounceMs),
        info_text("Delay before a window is added to the recent list after focusing."),
        spacer(8.0),
        slider_setting("Open Delay", open_delay_ms, 0, 2000, "ms", RecentWindowsMessage::SetOpenDelayMs),
        info_text("Delay before the switcher UI appears after pressing the shortcut."),
        spacer(16.0),

        // Highlight Style
        subsection_header("Highlight Style"),
        color_setting("Active Color", &active_color, RecentWindowsMessage::SetActiveColor),
        color_setting("Urgent Color", &urgent_color, RecentWindowsMessage::SetUrgentColor),
        slider_setting("Padding", padding, 0, 50, "px", RecentWindowsMessage::SetHighlightPadding),
        slider_setting("Corner Radius", corner_radius, 0, 50, "px", RecentWindowsMessage::SetHighlightCornerRadius),
        spacer(16.0),

        // Preview Settings
        subsection_header("Preview Settings"),
        slider_setting("Max Height", max_height, 50, 500, "px", RecentWindowsMessage::SetPreviewMaxHeight),
        scale_setting("Max Scale", max_scale, RecentWindowsMessage::SetPreviewMaxScale),
        spacer(16.0),

        // Keybindings section header
        subsection_header("Custom Keybindings"),
        info_text("Add custom keybindings for the window switcher. Leave empty to use defaults (Alt+Tab)."),
        spacer(8.0),
    ]
    .spacing(4);

    // Keybindings list
    if binds.is_empty() {
        content = content.push(
            text("No custom keybindings configured - using defaults")
                .size(14)
                .color([0.75, 0.75, 0.75])
        );
    } else {
        for (idx, bind) in binds.iter().enumerate() {
            let key_combo = bind.key_combo.clone();
            let is_next = bind.is_next;
            let filter_app = bind.filter_app_id;
            let scope = bind.scope;
            let cooldown = bind.cooldown_ms;

            content = content.push(
                container(
                    column![
                        // Header row with delete button
                        row![
                            text(format!("Keybinding #{}", idx + 1))
                                .size(13)
                                .color([0.75, 0.75, 0.75]),
                            button(text("×").size(14))
                                .on_press(Message::RecentWindows(RecentWindowsMessage::RemoveBind(idx)))
                                .padding([2, 8])
                                .style(delete_button_style),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),

                        // Key combo input
                        row![
                            text("Key Combo").size(13).width(Length::Fixed(120.0)),
                            text_input("e.g., Alt+Tab", &key_combo)
                                .on_input(move |s| Message::RecentWindows(RecentWindowsMessage::SetBindKeyCombo(idx, s)))
                                .padding(6)
                                .font(fonts::MONO_FONT)
                                .width(Length::Fixed(180.0)),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),

                        // Direction toggle
                        row![
                            text("Direction").size(13).width(Length::Fixed(120.0)),
                            pick_list(
                                vec!["Next Window", "Previous Window"],
                                Some(if is_next { "Next Window" } else { "Previous Window" }),
                                move |s| Message::RecentWindows(RecentWindowsMessage::SetBindIsNext(idx, s == "Next Window")),
                            )
                            .width(Length::Fixed(180.0)),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),

                        // Filter to app toggle
                        row![
                            text("Filter to App").size(13).width(Length::Fixed(120.0)),
                            toggler(filter_app)
                                .on_toggle(move |v| Message::RecentWindows(RecentWindowsMessage::SetBindFilterAppId(idx, v)))
                                .size(18),
                            text("Only show windows from current app").size(12).color([0.5, 0.5, 0.5]),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),

                        // Scope picker
                        row![
                            text("Scope").size(13).width(Length::Fixed(120.0)),
                            pick_list(
                                vec![ScopeOption::All, ScopeOption::Output, ScopeOption::Workspace],
                                Some(scope_to_option(&scope)),
                                move |opt| Message::RecentWindows(RecentWindowsMessage::SetBindScope(idx, option_to_scope(opt))),
                            )
                            .width(Length::Fixed(180.0)),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),

                        // Cooldown input
                        row![
                            text("Cooldown").size(13).width(Length::Fixed(120.0)),
                            text_input("ms (optional)", &cooldown.map(|c| c.to_string()).unwrap_or_default())
                                .on_input(move |s| {
                                    let cooldown = if s.is_empty() { None } else { s.parse().ok() };
                                    Message::RecentWindows(RecentWindowsMessage::SetBindCooldown(idx, cooldown))
                                })
                                .padding(6)
                                .width(Length::Fixed(100.0)),
                            text("ms").size(13).color([0.5, 0.5, 0.5]),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(8)
                )
                .padding(12)
                .style(card_style)
            );
            content = content.push(spacer(4.0));
        }
    }

    // Add keybinding button
    content = content.push(spacer(8.0));
    content = content.push(
        button(
            row![
                text("+").size(16),
                text("Add Keybinding").size(14),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        )
        .on_press(Message::RecentWindows(RecentWindowsMessage::AddBind))
        .padding([12, 20])
        .style(add_button_style)
    );

    content = content.push(spacer(16.0));

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
        .into()
}

// Scope option for pick_list display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeOption {
    All,
    Output,
    Workspace,
}

impl std::fmt::Display for ScopeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeOption::All => write!(f, "All Windows"),
            ScopeOption::Output => write!(f, "Current Output"),
            ScopeOption::Workspace => write!(f, "Current Workspace"),
        }
    }
}

fn scope_to_option(scope: &Option<RecentWindowsScope>) -> ScopeOption {
    match scope {
        None | Some(RecentWindowsScope::All) => ScopeOption::All,
        Some(RecentWindowsScope::Output) => ScopeOption::Output,
        Some(RecentWindowsScope::Workspace) => ScopeOption::Workspace,
    }
}

fn option_to_scope(opt: ScopeOption) -> Option<RecentWindowsScope> {
    match opt {
        ScopeOption::All => None,
        ScopeOption::Output => Some(RecentWindowsScope::Output),
        ScopeOption::Workspace => Some(RecentWindowsScope::Workspace),
    }
}

/// Create a toggle setting row
fn toggle_setting<F>(label: &str, value: bool, msg_fn: F) -> Element<'static, Message>
where
    F: Fn(bool) -> RecentWindowsMessage + 'static,
{
    row![
        text(label.to_string()).size(14).width(Length::Fixed(200.0)),
        toggler(value)
            .on_toggle(move |v| Message::RecentWindows(msg_fn(v)))
            .size(20),
    ]
    .spacing(16)
    .align_y(Alignment::Center)
    .into()
}

/// Create a slider setting row for i32 values
fn slider_setting<F>(label: &str, value: i32, min: i32, max: i32, unit: &str, msg_fn: F) -> Element<'static, Message>
where
    F: Fn(i32) -> RecentWindowsMessage + 'static,
{
    use iced::widget::slider;

    let unit_owned = unit.to_string();

    row![
        text(label.to_string()).size(14).width(Length::Fixed(200.0)),
        slider(min..=max, value, move |v| Message::RecentWindows(msg_fn(v)))
            .width(Length::Fixed(150.0)),
        text(format!("{} {}", value, unit_owned)).size(14).width(Length::Fixed(80.0)),
    ]
    .spacing(16)
    .align_y(Alignment::Center)
    .into()
}

/// Create a scale setting row (0.0-1.0 as percentage)
fn scale_setting<F>(label: &str, value: f64, msg_fn: F) -> Element<'static, Message>
where
    F: Fn(f64) -> RecentWindowsMessage + 'static,
{
    use iced::widget::slider;

    // Convert to percentage for slider (10-100%)
    let pct = (value * 100.0) as i32;

    row![
        text(label.to_string()).size(14).width(Length::Fixed(200.0)),
        slider(10..=100, pct, move |v| Message::RecentWindows(msg_fn(v as f64 / 100.0)))
            .width(Length::Fixed(150.0)),
        text(format!("{}%", pct)).size(14).width(Length::Fixed(80.0)),
    ]
    .spacing(16)
    .align_y(Alignment::Center)
    .into()
}

/// Create a color setting row with hex input
fn color_setting<F>(label: &str, hex: &str, msg_fn: F) -> Element<'static, Message>
where
    F: Fn(String) -> RecentWindowsMessage + 'static,
{
    let hex_owned = hex.to_string();

    // Parse color for preview swatch
    let parsed_color = crate::types::Color::from_hex(&hex_owned)
        .map(|c| iced::Color::from_rgba(
            c.r as f32 / 255.0,
            c.g as f32 / 255.0,
            c.b as f32 / 255.0,
            c.a as f32 / 255.0,
        ))
        .unwrap_or(iced::Color::from_rgb(0.5, 0.5, 0.5));

    row![
        text(label.to_string()).size(14).width(Length::Fixed(200.0)),
        container(text("").size(14))
            .width(24)
            .height(24)
            .style(move |_theme| {
                container::Style {
                    background: Some(iced::Background::Color(parsed_color)),
                    border: iced::Border {
                        color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            }),
        text_input("#RRGGBB", &hex_owned)
            .on_input(move |s| Message::RecentWindows(msg_fn(s)))
            .padding(6)
            .font(fonts::MONO_FONT)
            .width(Length::Fixed(100.0)),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

/// Style for delete buttons
fn delete_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => iced::Color::from_rgba(0.6, 0.2, 0.2, 0.5),
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color::from_rgb(0.8, 0.4, 0.4),
        ..Default::default()
    }
}

/// Style for add buttons
fn add_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => iced::Color::from_rgba(0.3, 0.5, 0.7, 0.5),
        button::Status::Pressed => iced::Color::from_rgba(0.4, 0.6, 0.8, 0.5),
        _ => iced::Color::from_rgba(0.2, 0.4, 0.6, 0.4),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color::WHITE,
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Style for card containers
fn card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.15, 0.15, 0.15, 0.4))),
        border: iced::Border {
            color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}
