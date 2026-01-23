//! Modal dialogs for the application
//!
//! Implements all modal dialogs with overlay backdrop:
//! - Error dialog
//! - Confirm dialog
//! - First-run wizard
//! - DiffView dialog
//! - Consolidation dialog
//! - Import summary dialog

use iced::widget::{button, checkbox, column, container, row, scrollable, text, Column};
use iced::{Alignment, Border, Color as IcedColor, Element, Length};

use crate::messages::{ConfirmAction, ConsolidationSuggestion, DialogState, Message, WizardStep};

/// Creates the modal overlay with dialog content
pub fn view<'a>(dialog: &'a DialogState, wizard_suggestions: &'a [ConsolidationSuggestion]) -> Option<Element<'a, Message>> {
    match dialog {
        DialogState::None => None,
        DialogState::Error { title, message, details } => {
            Some(error_dialog(title, message, details.as_deref()))
        }
        DialogState::Confirm {
            title,
            message,
            confirm_label,
            on_confirm,
        } => Some(confirm_dialog(title, message, confirm_label, on_confirm)),
        DialogState::FirstRunWizard { step } => Some(wizard_dialog(step, wizard_suggestions)),
        DialogState::ImportSummary {
            imported_count,
            defaulted_count,
            warnings,
        } => Some(import_summary_dialog(*imported_count, *defaulted_count, warnings)),
        DialogState::Consolidation { suggestions } => Some(consolidation_dialog(suggestions)),
        DialogState::DiffView { title, before, after } => Some(diff_view_dialog(title, before, after)),
    }
}

/// Error dialog
fn error_dialog<'a>(title: &'a str, message: &'a str, details: Option<&'a str>) -> Element<'a, Message> {
    let mut content = column![
        text(title).size(24),
        text(message).size(14).color([0.9, 0.9, 0.9]),
    ]
    .spacing(12);

    if let Some(details_str) = details {
        content = content.push(
            scrollable(
                container(text(details_str).size(12))
                    .padding(8)
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(IcedColor::from_rgb(0.15, 0.15, 0.15))),
                        border: Border {
                            color: IcedColor::from_rgb(0.3, 0.3, 0.3),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
            )
            .height(Length::Fixed(150.0))
        );
    }

    content = content.push(
        row![
            button(text("Close"))
                .on_press(Message::CloseDialog)
                .padding([8, 24])
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(IcedColor::from_rgb(0.3, 0.6, 0.9))),
                    text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
    );

    dialog_container(content)
}

/// Confirm dialog
fn confirm_dialog<'a>(
    title: &'a str,
    message: &'a str,
    confirm_label: &'a str,
    _on_confirm: &ConfirmAction,
) -> Element<'a, Message> {
    let content = column![
        text(title).size(24),
        text(message).size(14).color([0.9, 0.9, 0.9]),
        row![
            button(text("Cancel"))
                .on_press(Message::CloseDialog)
                .padding([8, 24])
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(IcedColor::from_rgb(0.3, 0.3, 0.3))),
                    text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            button(text(confirm_label))
                .on_press(Message::DialogConfirm)
                .padding([8, 24])
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(IcedColor::from_rgb(0.9, 0.3, 0.3))),
                    text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(12)
        .align_y(Alignment::Center)
    ]
    .spacing(16);

    dialog_container(content)
}

/// First-run wizard dialog
fn wizard_dialog<'a>(step: &WizardStep, wizard_suggestions: &'a [ConsolidationSuggestion]) -> Element<'a, Message> {
    let content: Column<'a, Message> = match step {
        WizardStep::Welcome => wizard_welcome(),
        WizardStep::ConfigSetup => wizard_config_setup(),
        WizardStep::ImportResults => wizard_import_results(),
        WizardStep::Consolidation => wizard_consolidation(wizard_suggestions),
        WizardStep::Complete => wizard_complete(),
    };

    dialog_container(content)
}

fn wizard_welcome<'a>() -> Column<'a, Message> {
    column![
        text("Welcome to Niri Settings").size(28),
        text("A graphical settings manager for the niri Wayland compositor")
            .size(14)
            .color([0.8, 0.8, 0.8]),
        container(
            column![
                text("Features:").size(13).color([0.7, 0.8, 0.9]),
                text("  - Visual configuration for all niri settings").size(12).color([0.7, 0.7, 0.7]),
                text("  - Window & layer rules with regex pattern matching").size(12).color([0.7, 0.7, 0.7]),
                text("  - Smart rule consolidation to merge similar rules").size(12).color([0.7, 0.7, 0.7]),
                text("  - Live preview - changes apply instantly").size(12).color([0.7, 0.7, 0.7]),
                text("  - Import your existing config automatically").size(12).color([0.7, 0.7, 0.7]),
            ]
            .spacing(4)
        )
        .padding([12, 16])
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.12, 0.14, 0.18))),
            border: Border {
                color: IcedColor::from_rgb(0.25, 0.3, 0.4),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }),
        text("This wizard will help you set up the application.")
            .size(13)
            .color([0.6, 0.6, 0.6]),
        row![
            button(text("Skip"))
                .on_press(Message::CloseDialog)
                .padding([8, 24]),
            button(text("Get Started"))
                .on_press(Message::WizardNext)
                .padding([8, 24])
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(IcedColor::from_rgb(0.3, 0.6, 0.9))),
                    text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(12)
    ]
    .spacing(16)
}

fn wizard_config_setup<'a>() -> Column<'a, Message> {
    column![
        text("Config Setup").size(24),
        text("Niri Settings uses a non-destructive approach to manage your config.")
            .size(14)
            .color([0.8, 0.8, 0.8]),
        container(
            column![
                text("How it works:").size(13).color([0.7, 0.8, 0.9]),
                text("1. We create separate .kdl files in ~/.config/niri/nirify/")
                    .size(12).color([0.7, 0.7, 0.7]),
                text("2. One include line is added to your config.kdl:")
                    .size(12).color([0.7, 0.7, 0.7]),
                container(
                    text("include \"~/.config/niri/nirify/main.kdl\"")
                        .size(11)
                        .color([0.5, 0.8, 0.5])
                )
                .padding([4, 12]),
                text("3. Your original config.kdl stays mostly untouched")
                    .size(12).color([0.7, 0.7, 0.7]),
            ]
            .spacing(6)
        )
        .padding([12, 16])
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.12, 0.14, 0.12))),
            border: Border {
                color: IcedColor::from_rgb(0.25, 0.35, 0.25),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }),
        text("A backup of your config will be created before any changes.")
            .size(12)
            .color([0.6, 0.7, 0.6]),
        row![
            button(text("Back"))
                .on_press(Message::WizardBack)
                .padding([8, 24]),
            button(text("Set Up Config"))
                .on_press(Message::WizardSetupConfig)
                .padding([8, 24])
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(IcedColor::from_rgb(0.3, 0.6, 0.9))),
                    text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(12)
    ]
    .spacing(16)
}

fn wizard_import_results<'a>() -> Column<'a, Message> {
    column![
        text("Configuration Ready").size(24),
        text("Your settings are now managed by Niri Settings.")
            .size(14)
            .color([0.8, 0.8, 0.8]),
        container(
            column![
                text("What was set up:").size(13).color([0.7, 0.8, 0.9]),
                text("  - Appearance, animations, and cursor settings").size(12).color([0.7, 0.7, 0.7]),
                text("  - Input devices (keyboard, mouse, touchpad, etc.)").size(12).color([0.7, 0.7, 0.7]),
                text("  - Window rules and layer rules").size(12).color([0.7, 0.7, 0.7]),
                text("  - Keybindings and gestures").size(12).color([0.7, 0.7, 0.7]),
                text("  - Workspaces, outputs, and layout settings").size(12).color([0.7, 0.7, 0.7]),
            ]
            .spacing(4)
        )
        .padding([12, 16])
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.12, 0.14, 0.18))),
            border: Border {
                color: IcedColor::from_rgb(0.25, 0.3, 0.4),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }),
        text("If you had existing window/layer rules, check the Tools page for consolidation suggestions.")
            .size(12)
            .color([0.7, 0.7, 0.6]),
        row![
            button(text("Back"))
                .on_press(Message::WizardBack)
                .padding([8, 24]),
            button(text("Next"))
                .on_press(Message::WizardNext)
                .padding([8, 24])
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(IcedColor::from_rgb(0.3, 0.6, 0.9))),
                    text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(12)
    ]
    .spacing(16)
}

fn wizard_consolidation<'a>(suggestions: &'a [ConsolidationSuggestion]) -> Column<'a, Message> {
    let selected_count = suggestions.iter().filter(|s| s.selected).count();
    let total_count = suggestions.len();

    let mut content = column![
        text("Optimize Your Rules").size(24),
        text(format!(
            "Found {} rules that could be merged to reduce duplication.",
            total_count
        ))
            .size(14)
            .color([0.8, 0.8, 0.8]),
        text("Select which suggestions to apply:")
            .size(13)
            .color([0.7, 0.7, 0.7]),
    ]
    .spacing(12);

    // Scrollable list of suggestions
    let mut suggestion_list = Column::new().spacing(8);

    for (index, suggestion) in suggestions.iter().enumerate() {
        let rule_type = if suggestion.is_window_rule { "window" } else { "layer" };
        let patterns_preview = if suggestion.patterns.len() <= 3 {
            suggestion.patterns.join(", ")
        } else {
            format!(
                "{}, ... ({} more)",
                suggestion.patterns[..2].join(", "),
                suggestion.patterns.len() - 2
            )
        };

        let bg_color = if suggestion.selected {
            IcedColor::from_rgb(0.15, 0.2, 0.15)
        } else {
            IcedColor::from_rgb(0.12, 0.12, 0.14)
        };

        suggestion_list = suggestion_list.push(
            container(
                row![
                    checkbox(suggestion.selected)
                        .on_toggle(move |_| Message::WizardConsolidationToggle(index)),
                    column![
                        text(&suggestion.description).size(12),
                        text(format!("{} rules: {}", rule_type, patterns_preview))
                            .size(11)
                            .color([0.6, 0.6, 0.6]),
                        text(format!("→ {}", suggestion.merged_pattern))
                            .size(11)
                            .color([0.5, 0.7, 0.9]),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                ]
                .spacing(12)
                .align_y(Alignment::Center)
            )
            .padding(10)
            .width(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(bg_color)),
                border: Border {
                    color: IcedColor::from_rgb(0.25, 0.25, 0.28),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
        );
    }

    content = content.push(
        scrollable(suggestion_list)
            .height(Length::Fixed(200.0))
    );

    // Buttons
    content = content.push(
        row![
            button(text("Skip"))
                .on_press(Message::WizardConsolidationSkip)
                .padding([8, 24]),
            button(text(format!("Apply {} Selected", selected_count)))
                .on_press(Message::WizardConsolidationApply)
                .padding([8, 24])
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(IcedColor::from_rgb(0.3, 0.6, 0.9))),
                    text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(12)
    );

    content
}

fn wizard_complete<'a>() -> Column<'a, Message> {
    column![
        text("You're All Set!").size(28),
        text("Niri Settings is ready to use.")
            .size(14)
            .color([0.8, 0.8, 0.8]),
        container(
            column![
                text("Tips:").size(13).color([0.7, 0.8, 0.9]),
                text("  - Changes apply instantly - no need to save manually").size(12).color([0.7, 0.7, 0.7]),
                text("  - Use Window Rules to customize per-app behavior").size(12).color([0.7, 0.7, 0.7]),
                text("  - Check Tools > Analyze Rules to consolidate similar rules").size(12).color([0.7, 0.7, 0.7]),
                text("  - Backups are created automatically in ~/.config/niri/nirify/backups/").size(12).color([0.7, 0.7, 0.7]),
            ]
            .spacing(4)
        )
        .padding([12, 16])
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.12, 0.15, 0.12))),
            border: Border {
                color: IcedColor::from_rgb(0.25, 0.35, 0.25),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }),
        button(text("Start Configuring"))
            .on_press(Message::CloseDialog)
            .padding([10, 32])
            .style(|_theme, _status| button::Style {
                background: Some(iced::Background::Color(IcedColor::from_rgb(0.3, 0.7, 0.3))),
                text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .spacing(20)
}

/// Import summary dialog
fn import_summary_dialog<'a>(
    imported_count: usize,
    defaulted_count: usize,
    warnings: &'a [String],
) -> Element<'a, Message> {
    let mut content = column![
        text("Import Summary").size(24),
        text(format!("Imported {} settings sections", imported_count))
            .size(14)
            .color([0.8, 0.8, 0.8]),
        text(format!("{} sections used default values", defaulted_count))
            .size(13)
            .color([0.7, 0.7, 0.7]),
    ]
    .spacing(12);

    if !warnings.is_empty() {
        let warnings_text = warnings.join("\n");

        content = content.push(text("Warnings:").size(14).color([0.9, 0.6, 0.3]));
        content = content.push(
            scrollable(
                container(text(warnings_text).size(12))
                    .padding(8)
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(IcedColor::from_rgb(0.2, 0.15, 0.1))),
                        border: Border {
                            color: IcedColor::from_rgb(0.5, 0.3, 0.2),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
            )
            .height(Length::Fixed(150.0))
        );
    }

    content = content.push(
        button(text("Close"))
            .on_press(Message::CloseDialog)
            .padding([8, 24])
    );

    dialog_container(content)
}

/// Consolidation dialog
fn consolidation_dialog<'a>(suggestions: &'a [ConsolidationSuggestion]) -> Element<'a, Message> {
    let suggestion_count = suggestions.len();
    let selected_count = suggestions.iter().filter(|s| s.selected).count();

    let mut content = column![
        text("Rule Consolidation Suggestions").size(24),
        text(format!(
            "Found {} opportunities to merge similar rules ({} selected)",
            suggestion_count, selected_count
        ))
            .size(14)
            .color([0.8, 0.8, 0.8]),
        text("Select suggestions to apply:")
            .size(13)
            .color([0.7, 0.7, 0.7]),
    ]
    .spacing(12);

    // Add suggestion items with checkboxes
    for (index, suggestion) in suggestions.iter().enumerate() {
        let rule_type = if suggestion.is_window_rule { "window" } else { "layer" };
        let patterns_preview = if suggestion.patterns.len() <= 3 {
            suggestion.patterns.join(", ")
        } else {
            format!(
                "{}, ... ({} more)",
                suggestion.patterns[..2].join(", "),
                suggestion.patterns.len() - 2
            )
        };

        let bg_color = if suggestion.selected {
            IcedColor::from_rgb(0.2, 0.25, 0.2)
        } else {
            IcedColor::from_rgb(0.15, 0.15, 0.15)
        };

        content = content.push(
            container(
                row![
                    checkbox(suggestion.selected)
                        .on_toggle(move |_| Message::ConsolidationToggle(index)),
                    column![
                        text(&suggestion.description).size(13),
                        text(format!("Type: {} rules", rule_type))
                            .size(11)
                            .color([0.6, 0.7, 0.6]),
                        text(format!("Patterns: {}", patterns_preview))
                            .size(11)
                            .color([0.6, 0.6, 0.6]),
                        text(format!("Merged: {}", suggestion.merged_pattern))
                            .size(11)
                            .color([0.5, 0.7, 0.9]),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                ]
                .spacing(12)
                .align_y(Alignment::Center)
            )
            .padding(12)
            .width(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(bg_color)),
                border: Border {
                    color: IcedColor::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
        );
    }

    // Buttons row
    let has_selection = selected_count > 0;
    let apply_btn = if has_selection {
        button(text(format!("Apply {} Selected", selected_count)))
            .on_press(Message::ConsolidationApply)
            .padding([8, 24])
            .style(|_theme, _status| button::Style {
                background: Some(iced::Background::Color(IcedColor::from_rgb(0.3, 0.6, 0.9))),
                text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
    } else {
        button(text("Apply Selected"))
            .padding([8, 24])
            .style(|_theme, _status| button::Style {
                background: Some(iced::Background::Color(IcedColor::from_rgb(0.3, 0.3, 0.3))),
                text_color: IcedColor::from_rgb(0.5, 0.5, 0.5),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
    };

    content = content.push(
        row![
            button(text("Dismiss"))
                .on_press(Message::CloseDialog)
                .padding([8, 24]),
            apply_btn,
        ]
        .spacing(12)
    );

    dialog_container(content)
}

/// Diff view dialog showing before/after config changes
fn diff_view_dialog<'a>(title: &'a str, before: &'a str, after: &'a str) -> Element<'a, Message> {
    let content = column![
        text(title).size(24),
        text("Compare configuration changes before applying")
            .size(13)
            .color([0.7, 0.7, 0.7]),
        row![
            // Before panel
            column![
                text("Before").size(14).color([0.9, 0.5, 0.5]),
                scrollable(
                    container(
                        text(before)
                            .size(12)
                            .font(iced::Font::MONOSPACE)
                    )
                    .padding(12)
                    .width(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(IcedColor::from_rgb(0.12, 0.10, 0.10))),
                        border: Border {
                            color: IcedColor::from_rgb(0.4, 0.25, 0.25),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                )
                .height(Length::Fixed(300.0))
            ]
            .spacing(8)
            .width(Length::Fill),
            // After panel
            column![
                text("After").size(14).color([0.5, 0.9, 0.5]),
                scrollable(
                    container(
                        text(after)
                            .size(12)
                            .font(iced::Font::MONOSPACE)
                    )
                    .padding(12)
                    .width(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(IcedColor::from_rgb(0.10, 0.12, 0.10))),
                        border: Border {
                            color: IcedColor::from_rgb(0.25, 0.4, 0.25),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                )
                .height(Length::Fixed(300.0))
            ]
            .spacing(8)
            .width(Length::Fill),
        ]
        .spacing(16),
        row![
            button(text("Close"))
                .on_press(Message::CloseDialog)
                .padding([8, 24])
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(IcedColor::from_rgb(0.3, 0.3, 0.3))),
                    text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            button(text("Apply Changes"))
                .on_press(Message::DialogConfirm)
                .padding([8, 24])
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(IcedColor::from_rgb(0.3, 0.6, 0.9))),
                    text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(12)
    ]
    .spacing(16);

    // Use wider dialog for diff view
    let dialog = container(content)
        .padding(32)
        .width(Length::Fixed(900.0))
        .max_height(600.0)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.18, 0.18, 0.20))),
            border: Border {
                color: IcedColor::from_rgb(0.4, 0.4, 0.4),
                width: 2.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        });

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(IcedColor {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.7,
            })),
            ..Default::default()
        })
        .into()
}

/// Wraps content in a dialog container with backdrop
fn dialog_container<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    // Create dialog box
    let dialog = container(content)
        .padding(32)
        .width(Length::Fixed(600.0))
        .max_height(700.0)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.18, 0.18, 0.20))),
            border: Border {
                color: IcedColor::from_rgb(0.4, 0.4, 0.4),
                width: 2.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        });

    // Stack dialog on top of backdrop (using a column for now - iced doesn't have true z-stacking)
    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(IcedColor {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.7,
            })),
            ..Default::default()
        })
        .into()
}
