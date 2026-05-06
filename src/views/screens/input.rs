//! Input screen — device summary cards + keybindings table
//!
//! Two-section layout: device cards at top, keybindings table at bottom.
//! Device details and keybinding editing are done through modal overlays.

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use crate::app::UiState;
use crate::config::Settings;
use crate::messages::{EditableDevice, KeybindingsMessage, Message};
use crate::theme::{fonts, neon};
use crate::views;

/// Input screen with device cards + keybindings table
pub fn view<'a>(settings: &'a Settings, ui: &'a UiState) -> Element<'a, Message> {
    let content = column![
        // ── Device Management Section ──────────────────────────────────
        super::hero_header(
            "DEVICE MANAGEMENT",
            "Hardware Profile",
            "Configure your input devices — keyboard layout, pointer acceleration, touchpad gestures, and more.",
            neon::SECONDARY,
        ),
        Space::new().height(20),
        // Device cards: row 1 (primary devices)
        row![
            device_card(EditableDevice::Keyboard, neon::PRIMARY, vec![
                ("Layout", settings.keyboard.xkb_layout.clone()),
                ("Delay", format!("{}ms", settings.keyboard.repeat_delay)),
            ]),
            device_card(EditableDevice::Mouse, neon::SECONDARY, vec![
                ("Accel", format!("{}", settings.mouse.accel_profile)),
                ("Scroll", format!("{:.1}x", settings.mouse.scroll_factor)),
            ]),
            device_card(EditableDevice::Touchpad, neon::TERTIARY, vec![
                ("Tap", (if settings.touchpad.tap { "On" } else { "Off" }).to_string()),
                ("Scroll", (if settings.touchpad.natural_scroll { "Natural" } else { "Standard" }).to_string()),
            ]),
            device_card(EditableDevice::Gestures, neon::PRIMARY, vec![
                ("Hot Corners", (if settings.gestures.hot_corners.enabled { "On" } else { "Off" }).to_string()),
                ("Edge Scroll", (if settings.gestures.dnd_edge_view_scroll.enabled { "On" } else { "Off" }).to_string()),
            ]),
        ].spacing(12).align_y(Alignment::Start),
        Space::new().height(12),
        row![
            device_card(EditableDevice::Trackpoint, neon::SECONDARY, vec![
                ("Accel", format!("{}", settings.trackpoint.accel_profile)),
                ("Scroll", format!("{}", settings.trackpoint.scroll_method)),
            ]),
            device_card(EditableDevice::Trackball, neon::TERTIARY, vec![
                ("Accel", format!("{}", settings.trackball.accel_profile)),
                ("Scroll", format!("{}", settings.trackball.scroll_method)),
            ]),
            device_card(EditableDevice::Tablet, neon::PRIMARY, vec![
                ("Output", if settings.tablet.map_to_output.is_empty() { "Auto".to_string() } else { settings.tablet.map_to_output.clone() }),
                ("Left Hand", (if settings.tablet.left_handed { "On" } else { "Off" }).to_string()),
            ]),
            device_card(EditableDevice::Touch, neon::SECONDARY, vec![
                ("Output", if settings.touch.map_to_output.is_empty() { "Auto".to_string() } else { settings.touch.map_to_output.clone() }),
                ("Enabled", (if settings.touch.off { "Off" } else { "On" }).to_string()),
            ]),
        ].spacing(12).align_y(Alignment::Start),
        Space::new().height(32),
        // ── Keybindings Section ────────────────────────────────────────
        row![
            column![
                super::hero_header(
                    "COMMAND LAYER",
                    "System Bindings",
                    "Keyboard shortcuts for compositor actions, window management, and custom commands.",
                    neon::PRIMARY,
                ),
            ]
            .width(Length::Fill),
            button(
                row![
                    text("+").size(16),
                    text("New Binding").size(14).font(fonts::UI_FONT_MEDIUM),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .on_press(Message::Keybindings(KeybindingsMessage::AddKeybinding))
            .padding([10, 20])
            .style(|_: &iced::Theme, status| {
                let bg = match status {
                    iced::widget::button::Status::Hovered => neon::PRIMARY,
                    _ => iced::Color { a: 0.8, ..neon::PRIMARY },
                };
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: neon::SURFACE_LOW,
                    border: iced::Border { radius: 12.0.into(), ..Default::default() },
                    ..Default::default()
                }
            }),
        ]
        .align_y(Alignment::End),
        Space::new().height(16),
        // Search bar
        container(
            row![
                text("⌕").size(16).color(neon::OUTLINE_VARIANT),
                text_input("Search bindings by action, key, or command...", &ui.keybindings_search)
                    .on_input(Message::SetKeybindingsSearch)
                    .padding([8, 4])
                    .size(14)
                    .width(Length::Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([8, 16])
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(neon::SURFACE_CONTAINER_HIGH)),
            border: iced::Border {
                radius: 12.0.into(),
                color: iced::Color { a: 0.15, ..neon::OUTLINE_VARIANT },
                width: 1.0,
            },
            ..Default::default()
        }),
        Space::new().height(12),
        // Table header
        container(
            row![
                text("COMMAND / ACTION").size(10).font(fonts::UI_FONT_SEMIBOLD).color(neon::OUTLINE_VARIANT)
                    .width(Length::FillPortion(4)),
                text("MODIFIER").size(10).font(fonts::UI_FONT_SEMIBOLD).color(neon::OUTLINE_VARIANT)
                    .width(Length::FillPortion(2)),
                text("KEY").size(10).font(fonts::UI_FONT_SEMIBOLD).color(neon::OUTLINE_VARIANT)
                    .width(Length::FillPortion(2)),
            ]
            .spacing(8)
            .padding([8, 16]),
        )
        .width(Length::Fill),
        // Table rows
        keybindings_table(settings, ui),
    ]
    .spacing(0)
    .padding(32)
    .width(Length::Fill);

    scrollable(content).height(Length::Fill).into()
}

// ── Device Card ────────────────────────────────────────────────────────────

fn device_card<'a>(
    device: EditableDevice,
    accent: iced::Color,
    summary: Vec<(&'static str, String)>,
) -> Element<'a, Message> {
    let icon = device.icon();
    let name = device.name();

    let mut summary_items = column![].spacing(4);
    for (label, value) in summary {
        summary_items = summary_items.push(
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text(value).size(11).font(fonts::MONO_FONT).color(accent),
            ]
            .align_y(Alignment::Center),
        );
    }

    let card = column![
        // Header: icon + name
        row![
            container(text(icon).size(18).color(accent),)
                .width(36)
                .height(36)
                .center(Length::Shrink)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.12, ..accent })),
                    border: iced::Border {
                        radius: 10.0.into(),
                        color: iced::Color { a: 0.2, ..accent },
                        width: 1.0
                    },
                    ..Default::default()
                }),
            Space::new().width(10),
            text(name).size(14).font(fonts::UI_FONT_SEMIBOLD),
        ]
        .align_y(Alignment::Center),
        Space::new().height(10),
        summary_items,
        Space::new().height(10),
        button(
            text("CONFIGURE")
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(accent),
        )
        .on_press(Message::OpenDeviceEditor(device))
        .padding([6, 12])
        .width(Length::Fill)
        .style(move |_: &iced::Theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => iced::Color { a: 0.15, ..accent },
                _ => iced::Color { a: 0.08, ..accent },
            };
            iced::widget::button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: accent,
                border: iced::Border {
                    radius: 8.0.into(),
                    color: iced::Color { a: 0.2, ..accent },
                    width: 1.0,
                },
                ..Default::default()
            }
        }),
    ]
    .spacing(0);

    container(card)
        .padding(16)
        .width(Length::FillPortion(1))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(neon::SURFACE_CONTAINER)),
            border: iced::Border {
                color: iced::Color { a: 0.12, ..accent },
                width: 1.0,
                radius: 16.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.08, ..accent },
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 20.0,
            },
            ..Default::default()
        })
        .into()
}

// ── Keybindings Table ──────────────────────────────────────────────────────

fn keybindings_table<'a>(settings: &'a Settings, ui: &'a UiState) -> Element<'a, Message> {
    let search = &ui.keybindings_search;

    let filtered: Vec<(usize, &crate::config::models::Keybinding)> = settings
        .keybindings
        .bindings
        .iter()
        .enumerate()
        .filter(|(_, kb)| {
            if search.is_empty() {
                return true;
            }
            let s = search.to_lowercase();
            kb.key_combo.to_lowercase().contains(&s)
                || kb.display_name().to_lowercase().contains(&s)
                || kb.action.description().to_lowercase().contains(&s)
        })
        .collect();

    let total = settings.keybindings.bindings.len();
    let shown = filtered.len();

    let mut rows = column![].spacing(2);
    for (idx, kb) in &filtered {
        rows = rows.push(keybinding_row(*idx, kb));
    }

    column![
        container(rows).width(Length::Fill),
        Space::new().height(8),
        // Footer
        container(
            text(format!("Showing {} of {} total bindings", shown, total))
                .size(11)
                .color(neon::ON_SURFACE_VARIANT),
        )
        .width(Length::Fill)
        .center_x(Length::Fill)
        .padding([8, 0]),
    ]
    .spacing(0)
    .into()
}

fn keybinding_row<'a>(
    idx: usize,
    kb: &'a crate::config::models::Keybinding,
) -> Element<'a, Message> {
    // Parse key combo into modifiers + key
    let parts: Vec<&str> = kb.key_combo.split('+').collect();
    let key = parts.last().copied().unwrap_or("");
    let modifiers: Vec<&str> = if parts.len() > 1 {
        parts[..parts.len() - 1].to_vec()
    } else {
        vec![]
    };

    // Action display
    let action_name = kb.display_name();
    let action_subtitle = kb.action.description();

    // Modifier pills
    let mod_pills: Vec<Element<'a, Message>> = modifiers
        .iter()
        .map(|m| {
            let label = match *m {
                "Mod" => "Super",
                other => other,
            };
            modifier_pill(label)
        })
        .collect();

    let row_content = row![
        // Command / Action column
        column![
            text(action_name.to_string())
                .size(13)
                .font(fonts::UI_FONT_MEDIUM),
            text(action_subtitle.to_string())
                .size(11)
                .color(neon::ON_SURFACE_VARIANT),
        ]
        .spacing(2)
        .width(Length::FillPortion(4)),
        // Modifier column
        row(mod_pills).spacing(4).width(Length::FillPortion(2)),
        // Key column
        row![key_pill(key)].width(Length::FillPortion(2)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding([10, 16]);

    button(row_content)
        .on_press(Message::OpenKeybindingEditor(idx))
        .width(Length::Fill)
        .style(|_: &iced::Theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => iced::Color {
                    a: 0.06,
                    ..neon::ON_SURFACE
                },
                _ => iced::Color::TRANSPARENT,
            };
            iced::widget::button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: neon::ON_SURFACE,
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

// ── Device Editor Modal ────────────────────────────────────────────────────

/// Wraps an existing device view in a modal overlay
pub fn device_editor_modal<'a>(
    device: EditableDevice,
    settings: &'a Settings,
    ui: &'a UiState,
) -> Element<'a, Message> {
    let accent = match device {
        EditableDevice::Keyboard => neon::PRIMARY,
        EditableDevice::Mouse => neon::SECONDARY,
        EditableDevice::Touchpad => neon::TERTIARY,
        EditableDevice::Gestures => neon::PRIMARY,
        _ => neon::SECONDARY,
    };

    // Get the device view content from existing views
    let device_content: Element<'a, Message> = match device {
        EditableDevice::Keyboard => views::keyboard::view(&settings.keyboard),
        EditableDevice::Mouse => views::mouse::view(&settings.mouse),
        EditableDevice::Touchpad => views::touchpad::view(&settings.touchpad),
        EditableDevice::Trackpoint => views::trackpoint::view(&settings.trackpoint),
        EditableDevice::Trackball => views::trackball::view(&settings.trackball),
        EditableDevice::Tablet => {
            views::tablet::view(&settings.tablet, &ui.tablet_calibration_cache)
        }
        EditableDevice::Touch => views::touch::view(&settings.touch, &ui.touch_calibration_cache),
        EditableDevice::Gestures => views::gestures::view(&settings.gestures),
    };

    // Wider modal for touchpad/gestures (more settings)
    let modal_width = match device {
        EditableDevice::Touchpad | EditableDevice::Gestures => 900.0,
        _ => 750.0,
    };

    let editor = column![
        // Header
        row![
            container(text(device.icon()).size(24).color(accent))
                .width(48)
                .height(48)
                .center(Length::Shrink)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.15, ..accent })),
                    border: iced::Border {
                        radius: 14.0.into(),
                        color: iced::Color { a: 0.25, ..accent },
                        width: 1.0
                    },
                    ..Default::default()
                }),
            Space::new().width(16),
            column![
                text("DEVICE CONFIGURATION")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(accent),
                text(device.name()).size(22).font(fonts::UI_FONT_SEMIBOLD),
            ]
            .spacing(4)
            .width(Length::Fill),
            button(text("✕").size(16).color(neon::ON_SURFACE_VARIANT))
                .on_press(Message::CloseDeviceEditor)
                .padding([8, 12])
                .style(|_: &iced::Theme, status| {
                    let bg = match status {
                        iced::widget::button::Status::Hovered => iced::Color {
                            a: 0.15,
                            ..neon::ON_SURFACE
                        },
                        _ => iced::Color {
                            a: 0.08,
                            ..neon::ON_SURFACE
                        },
                    };
                    iced::widget::button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: neon::ON_SURFACE,
                        border: iced::Border {
                            radius: 999.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
        ]
        .spacing(0)
        .align_y(Alignment::Center),
        Space::new().height(16),
        // Device content (from existing view)
        device_content,
        // Footer
        Space::new().height(16),
        container(Space::new().width(Length::Fill).height(1))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.15,
                    ..neon::OUTLINE_VARIANT
                })),
                ..Default::default()
            }),
        container(
            row![
                row![
                    text("●").size(10).color(neon::SECONDARY),
                    text("Live Configuration Sync Active")
                        .size(12)
                        .color(neon::ON_SURFACE_VARIANT),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
                .width(Length::Fill),
                button(text("Close").size(13).font(fonts::UI_FONT_MEDIUM))
                    .on_press(Message::CloseDeviceEditor)
                    .padding([10, 20])
                    .style(|_: &iced::Theme, status| {
                        let bg = match status {
                            iced::widget::button::Status::Hovered => neon::PRIMARY,
                            _ => iced::Color {
                                a: 0.85,
                                ..neon::PRIMARY
                            },
                        };
                        iced::widget::button::Style {
                            background: Some(iced::Background::Color(bg)),
                            text_color: neon::SURFACE_LOW,
                            border: iced::Border {
                                radius: 12.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
            ]
            .align_y(Alignment::Center)
        )
        .padding([16, 0]),
    ];

    let modal_content = scrollable(editor.spacing(0).width(Length::Fill)).height(Length::Fill);

    let dialog = container(modal_content)
        .padding(32)
        .width(Length::Fixed(modal_width))
        .max_height(700.0)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(neon::SURFACE_CONTAINER_HIGH)),
            border: iced::Border {
                color: iced::Color { a: 0.3, ..accent },
                width: 2.0,
                radius: 20.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 40.0,
            },
            ..Default::default()
        });

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.7,
            })),
            ..Default::default()
        })
        .into()
}

// ── Key/Modifier Pills ────────────────────────────────────────────────────

fn modifier_pill<'a>(label: &str) -> Element<'a, Message> {
    container(
        text(label.to_string())
            .size(10)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(neon::SECONDARY),
    )
    .padding([4, 10])
    .style(|_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color {
            a: 0.12,
            ..neon::SECONDARY
        })),
        border: iced::Border {
            color: iced::Color {
                a: 0.3,
                ..neon::SECONDARY
            },
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn key_pill<'a>(label: &str) -> Element<'a, Message> {
    container(
        text(label.to_string())
            .size(11)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(neon::PRIMARY),
    )
    .padding([4, 12])
    .style(|_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color {
            a: 0.10,
            ..neon::PRIMARY
        })),
        border: iced::Border {
            color: iced::Color {
                a: 0.3,
                ..neon::PRIMARY
            },
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}
