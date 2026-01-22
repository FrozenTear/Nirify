//! Modal dialogs for the application
//!
//! Implements all modal dialogs with overlay backdrop:
//! - Error dialog
//! - Confirm dialog
//! - First-run wizard
//! - DiffView dialog
//! - Consolidation dialog
//! - Import summary dialog

use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Alignment, Border, Color as IcedColor, Element, Length};

use crate::messages::{ConfirmAction, ConsolidationSuggestion, DialogState, Message, WizardStep};

/// Creates the modal overlay with dialog content
pub fn view<'a>(dialog: &'a DialogState) -> Option<Element<'a, Message>> {
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
        DialogState::FirstRunWizard { step } => Some(wizard_dialog(step)),
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
fn wizard_dialog<'a>(step: &WizardStep) -> Element<'a, Message> {
    let content: Column<'a, Message> = match step {
        WizardStep::Welcome => wizard_welcome(),
        WizardStep::ConfigSetup => wizard_config_setup(),
        WizardStep::ImportResults => wizard_import_results(),
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
        text("This wizard will help you set up the application for first use.")
            .size(13)
            .color([0.7, 0.7, 0.7]),
        row![
            button(text("Skip"))
                .on_press(Message::CloseDialog)
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

fn wizard_config_setup<'a>() -> Column<'a, Message> {
    column![
        text("Config Setup").size(24),
        text("Niri Settings manages your configuration through separate files.")
            .size(14)
            .color([0.8, 0.8, 0.8]),
        text("We'll add one line to your config.kdl to include our settings:")
            .size(13)
            .color([0.7, 0.7, 0.7]),
        container(
            text("include \"~/.config/niri/niri-settings/main.kdl\"")
                .size(12)
                .color([0.5, 0.8, 0.5])
        )
        .padding(12)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.1, 0.15, 0.1))),
            border: Border {
                color: IcedColor::from_rgb(0.3, 0.5, 0.3),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }),
        text("Your original config will be backed up before any changes.")
            .size(12)
            .color([0.6, 0.6, 0.6]),
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
        text("Import Complete").size(24),
        text("Your existing configuration has been imported.")
            .size(14)
            .color([0.8, 0.8, 0.8]),
        text("All settings are now available for editing through the UI.")
            .size(13)
            .color([0.7, 0.7, 0.7]),
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

fn wizard_complete<'a>() -> Column<'a, Message> {
    column![
        text("Setup Complete!").size(28),
        text("Niri Settings is now ready to use.")
            .size(14)
            .color([0.8, 0.8, 0.8]),
        text("You can now configure niri through the sidebar pages.")
            .size(13)
            .color([0.7, 0.7, 0.7]),
        text("Changes are saved automatically after a short delay.")
            .size(12)
            .color([0.6, 0.6, 0.6]),
        button(text("Get Started"))
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

    let mut content = column![
        text("Rule Consolidation Suggestions").size(24),
        text(format!("Found {} opportunities to merge similar rules", suggestion_count))
            .size(14)
            .color([0.8, 0.8, 0.8]),
        text("Select suggestions to apply:")
            .size(13)
            .color([0.7, 0.7, 0.7]),
    ]
    .spacing(12);

    // Add suggestion items (simplified for now)
    for (_idx, suggestion) in suggestions.iter().enumerate() {
        content = content.push(
            container(
                column![
                    text(&suggestion.description).size(13),
                    text(format!("{} rules can be merged", suggestion.rule_count))
                        .size(11)
                        .color([0.6, 0.6, 0.6]),
                ]
                .spacing(4)
            )
            .padding(12)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(IcedColor::from_rgb(0.15, 0.15, 0.15))),
                border: Border {
                    color: IcedColor::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
        );
    }

    content = content.push(
        row![
            button(text("Dismiss"))
                .on_press(Message::CloseDialog)
                .padding([8, 24]),
            button(text("Apply Selected"))
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
                }),
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
