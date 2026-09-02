//! Modal dialogs for the application
//!
//! Implements all modal dialogs with overlay backdrop:
//! - Error dialog
//! - Confirm dialog
//! - First-run wizard
//! - DiffView dialog
//! - Consolidation dialog
//! - Import summary dialog

use iced::widget::{
    button, checkbox, column, container, mouse_area, row, scrollable, stack, text, Column, Space,
};
use iced::{Alignment, Border, Color as IcedColor, Element, Length};

use crate::app::ui_state::WizardImportSummary;
use crate::messages::{ConfirmAction, ConsolidationSuggestion, DialogState, Message, WizardStep};
use crate::version::{get_unsupported_features, NiriVersion};

/// Theme-aware muted text color for dialog body copy. Pulls the theme's
/// base text color partway toward the base background so it reads as a
/// secondary tone in BOTH light and dark themes (the old hard-coded light
/// greys were invisible white-on-white on the light palettes).
pub(crate) fn muted_text_style(theme: &iced::Theme) -> text::Style {
    let p = theme.extended_palette();
    let fg = p.background.base.text;
    let bg = p.background.base.color;
    let t = 0.30_f32;
    text::Style {
        color: Some(IcedColor::from_rgb(
            fg.r + (bg.r - fg.r) * t,
            fg.g + (bg.g - fg.g) * t,
            fg.b + (bg.b - fg.b) * t,
        )),
    }
}

/// Theme-aware warning/attention text color for dialog body copy.
fn warning_text_style(theme: &iced::Theme) -> text::Style {
    let p = theme.extended_palette();
    text::Style {
        color: Some(p.danger.strong.color),
    }
}

/// Theme-aware accent color for info-box headings ("Features:", "How it works:")
/// and highlighted values, readable in both light and dark themes.
fn accent_text_style(theme: &iced::Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().primary.strong.color),
    }
}

/// Theme-aware positive/confirmation color (e.g. the include-line snippet).
fn success_text_style(theme: &iced::Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().success.strong.color),
    }
}

/// Theme-aware container style for the callout/info boxes inside dialogs.
/// Uses the theme's weak background so it reads as a subtle panel in BOTH
/// light and dark themes, and pins the text color so any uncolored inner
/// copy stays legible (the old hard-coded dark boxes turned into dark
/// islands with dark inherited text on the light palettes).
fn info_box_style(theme: &iced::Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(p.background.weak.color.into()),
        text_color: Some(p.background.weak.text),
        border: Border {
            color: p.background.strong.color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Theme-aware warning callout box (version notes etc.).
fn warning_box_style(theme: &iced::Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(p.danger.weak.color.into()),
        text_color: Some(p.danger.weak.text),
        border: Border {
            color: p.danger.strong.color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Theme-aware style for a selectable list item (consolidation suggestions).
/// Selected rows use the primary-weak tint, others the neutral weak panel;
/// both pin their text color so inner copy is legible in every theme.
fn selection_item_style(theme: &iced::Theme, selected: bool) -> container::Style {
    let p = theme.extended_palette();
    let pair = if selected {
        p.primary.weak
    } else {
        p.background.weak
    };
    container::Style {
        background: Some(pair.color.into()),
        text_color: Some(pair.text),
        border: Border {
            color: p.background.strong.color,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Creates the modal overlay with dialog content
pub fn view<'a>(
    dialog: &'a DialogState,
    wizard_suggestions: &'a [ConsolidationSuggestion],
    niri_version: Option<NiriVersion>,
    pending_revert: Option<&crate::app::ui_state::PendingRevert>,
    wizard_import: Option<&'a WizardImportSummary>,
) -> Option<Element<'a, Message>> {
    match dialog {
        DialogState::None => None,
        DialogState::Error {
            title,
            message,
            details,
        } => Some(error_dialog(title, message, details.as_deref())),
        DialogState::Confirm {
            title,
            message,
            confirm_label,
            on_confirm,
        } => Some(confirm_dialog(title, message, confirm_label, on_confirm)),
        DialogState::FirstRunWizard { step } => Some(wizard_dialog(
            step,
            wizard_suggestions,
            niri_version,
            wizard_import,
        )),
        DialogState::ImportSummary {
            imported_count,
            defaulted_count,
            warnings,
        } => Some(import_summary_dialog(
            *imported_count,
            *defaulted_count,
            warnings,
        )),
        DialogState::Consolidation { suggestions } => Some(consolidation_dialog(suggestions)),
        DialogState::DiffView {
            title,
            before,
            after,
        } => Some(diff_view_dialog(title, before, after)),
        DialogState::RevertCountdown { description } => {
            let seconds_left = pending_revert.map(|p| p.seconds_left).unwrap_or(0);
            Some(revert_countdown_dialog(description, seconds_left))
        }
    }
}

/// Error dialog
fn error_dialog<'a>(
    title: &'a str,
    message: &'a str,
    details: Option<&'a str>,
) -> Element<'a, Message> {
    let mut content = column![
        text(title).size(24),
        text(message).size(14).style(muted_text_style),
    ]
    .spacing(12);

    if let Some(details_str) = details {
        content = content.push(
            scrollable(
                container(text(details_str).size(12))
                    .padding(8)
                    .style(info_box_style),
            )
            .height(Length::Fixed(150.0)),
        );
    }

    content = content.push(
        row![button(text("Close"))
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
            }),]
        .spacing(8)
        .align_y(Alignment::Center),
    );

    dialog_container(content, true, 600.0)
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
        text(message).size(14).style(muted_text_style),
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

    dialog_container(content, true, 600.0)
}

/// First-run wizard dialog
fn wizard_dialog<'a>(
    step: &WizardStep,
    wizard_suggestions: &'a [ConsolidationSuggestion],
    niri_version: Option<NiriVersion>,
    wizard_import: Option<&'a WizardImportSummary>,
) -> Element<'a, Message> {
    let content: Column<'a, Message> = match step {
        WizardStep::Welcome => wizard_welcome(niri_version),
        WizardStep::ConfigSetup => wizard_config_setup(),
        WizardStep::ImportResults => wizard_import_results(wizard_import),
        WizardStep::Consolidation => wizard_consolidation(wizard_suggestions),
        WizardStep::Complete => wizard_complete(),
        WizardStep::SkipWarning => wizard_skip_warning(),
    };

    // The wizard must not be dismissible via backdrop click before setup.
    dialog_container(content, false, 600.0)
}

fn wizard_welcome<'a>(niri_version: Option<NiriVersion>) -> Column<'a, Message> {
    let mut content = column![
        text("Welcome to Niri Settings").size(28),
        text("A graphical settings manager for the niri Wayland compositor")
            .size(14)
            .style(muted_text_style),
        container(
            column![
                text("Features:").size(13).style(accent_text_style),
                text("  - Visual configuration for all niri settings")
                    .size(12)
                    .style(muted_text_style),
                text("  - Window & layer rules with regex pattern matching")
                    .size(12)
                    .style(muted_text_style),
                text("  - Smart rule consolidation to merge similar rules")
                    .size(12)
                    .style(muted_text_style),
                text("  - Live preview - changes apply instantly")
                    .size(12)
                    .style(muted_text_style),
                text("  - Import your existing config automatically")
                    .size(12)
                    .style(muted_text_style),
            ]
            .spacing(4)
        )
        .padding([12, 16])
        .style(info_box_style),
    ]
    .spacing(16);

    // Show version warning if some features are not supported
    if let Some(version) = niri_version {
        let unsupported = get_unsupported_features(version);
        if !unsupported.is_empty() {
            let feature_list: String = unsupported
                .iter()
                .map(|f| format!("  - {} (requires {}+)", f.display_name(), f.min_version()))
                .collect::<Vec<_>>()
                .join("\n");

            content = content.push(
                container(
                    column![
                        text(format!("Note: niri {} detected", version))
                            .size(12)
                            .style(warning_text_style),
                        text("Some features are not available in your version:").size(11),
                        text(feature_list).size(11),
                    ]
                    .spacing(4),
                )
                .padding([10, 14])
                .style(warning_box_style),
            );
        }
    }

    content = content.push(
        text("This wizard will help you set up the application.")
            .size(13)
            .style(muted_text_style),
    );

    content = content.push(
        row![
            button(text("Skip"))
                .on_press(Message::WizardBack)
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
        .spacing(12),
    );

    content
}

fn wizard_config_setup<'a>() -> Column<'a, Message> {
    column![
        text("Config Setup").size(24),
        text("Nirify will import your current settings, then take over those sections.")
            .size(14)
            .style(muted_text_style),
        container(
            column![
                text("How it works:").size(13).style(accent_text_style),
                text("1. Your existing config.kdl (and includes) is imported into Nirify")
                    .size(12)
                    .style(muted_text_style),
                text("2. A timestamped backup is written to .nirify-backups/")
                    .size(12)
                    .style(muted_text_style),
                text("3. Managed sections (layout, input, outputs, binds, …) move into nirify/")
                    .size(12)
                    .style(muted_text_style),
                text("4. One include line is added last in config.kdl:")
                    .size(12)
                    .style(muted_text_style),
                container(
                    text("include \"nirify/main.kdl\"")
                        .size(11)
                        .style(success_text_style)
                )
                .padding([4, 12]),
                text("5. Unmanaged / custom nodes in config.kdl are preserved")
                    .size(12)
                    .style(muted_text_style),
            ]
            .spacing(6)
        )
        .padding([12, 16])
        .style(info_box_style),
        text("A backup of your config is always created before config.kdl is rewritten.")
            .size(12)
            .style(muted_text_style),
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

fn wizard_import_results<'a>(import: Option<&'a WizardImportSummary>) -> Column<'a, Message> {
    let heading = import
        .map(|i| i.summary.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Configuration ready");

    let mut imported_list = Column::new().spacing(4);
    imported_list = imported_list.push(
        text("What was imported from your config:")
            .size(13)
            .style(accent_text_style),
    );
    match import {
        Some(i) if !i.imported_sections.is_empty() => {
            for section in &i.imported_sections {
                imported_list = imported_list.push(
                    text(format!("  - {}", section))
                        .size(12)
                        .style(muted_text_style),
                );
            }
            if i.includes_processed > 0 {
                imported_list = imported_list.push(
                    text(format!(
                        "  ({} include file(s) processed)",
                        i.includes_processed
                    ))
                    .size(11)
                    .style(muted_text_style),
                );
            }
        }
        Some(i) => {
            imported_list = imported_list.push(
                text("No existing settings found in config.kdl; using defaults.")
                    .size(12)
                    .style(muted_text_style),
            );
            if i.defaulted_count > 0 {
                imported_list = imported_list.push(
                    text(format!(
                        "{} section(s) used built-in defaults.",
                        i.defaulted_count
                    ))
                    .size(11)
                    .style(muted_text_style),
                );
            }
        }
        None => {
            imported_list = imported_list.push(
                text("Import results were not recorded. Your nirify/ files were written from the in-memory settings at setup time.")
                    .size(12)
                    .style(muted_text_style),
            );
        }
    }

    let mut content = column![
        text("Configuration Ready").size(24),
        text(heading).size(14).style(muted_text_style),
        container(imported_list)
            .padding([12, 16])
            .style(info_box_style),
    ]
    .spacing(16);

    if let Some(i) = import {
        if !i.warnings.is_empty() {
            let warnings_text = i.warnings.join("\n");
            content = content.push(text("Warnings:").size(13).style(warning_text_style));
            content = content.push(
                scrollable(
                    container(text(warnings_text).size(12))
                        .padding(8)
                        .style(warning_box_style),
                )
                .height(Length::Fixed(100.0)),
            );
        }
    }

    content = content.push(
        text("If you had existing window/layer rules, check the Tools page for consolidation suggestions.")
            .size(12)
            .style(muted_text_style),
    );
    content = content.push(
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
        .spacing(12),
    );
    content
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
        .style(muted_text_style),
        text("Select which suggestions to apply:")
            .size(13)
            .style(muted_text_style),
    ]
    .spacing(12);

    // Scrollable list of suggestions
    let mut suggestion_list = Column::new().spacing(8);

    for (index, suggestion) in suggestions.iter().enumerate() {
        let rule_type = if suggestion.is_window_rule {
            "window"
        } else {
            "layer"
        };
        let patterns_preview = if suggestion.patterns.len() <= 3 {
            suggestion.patterns.join(", ")
        } else {
            format!(
                "{}, ... ({} more)",
                suggestion.patterns[..2].join(", "),
                suggestion.patterns.len() - 2
            )
        };

        let selected = suggestion.selected;

        suggestion_list = suggestion_list.push(
            container(
                row![
                    checkbox(suggestion.selected)
                        .on_toggle(move |_| Message::WizardConsolidationToggle(index)),
                    column![
                        text(&suggestion.description).size(12),
                        text(format!("{} rules: {}", rule_type, patterns_preview))
                            .size(11)
                            .style(muted_text_style),
                        text(format!("→ {}", suggestion.merged_pattern))
                            .size(11)
                            .style(accent_text_style),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding(10)
            .width(Length::Fill)
            .style(move |theme: &iced::Theme| selection_item_style(theme, selected)),
        );
    }

    content = content.push(scrollable(suggestion_list).height(Length::Fixed(200.0)));

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
        .spacing(12),
    );

    content
}

fn wizard_complete<'a>() -> Column<'a, Message> {
    column![
        text("You're All Set!").size(28),
        text("Niri Settings is ready to use.")
            .size(14)
            .style(muted_text_style),
        container(
            column![
                text("Tips:").size(13).style(accent_text_style),
                text("  - Changes apply instantly - no need to save manually")
                    .size(12)
                    .style(muted_text_style),
                text("  - Use Window Rules to customize per-app behavior")
                    .size(12)
                    .style(muted_text_style),
                text("  - Check Tools > Analyze Rules to consolidate similar rules")
                    .size(12)
                    .style(muted_text_style),
                text("  - Backups are saved to ~/.config/niri/.nirify-backups/")
                    .size(12)
                    .style(muted_text_style),
            ]
            .spacing(4)
        )
        .padding([12, 16])
        .style(info_box_style),
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
            .style(muted_text_style),
        text(format!("{} sections used default values", defaulted_count))
            .size(13)
            .style(muted_text_style),
    ]
    .spacing(12);

    if !warnings.is_empty() {
        let warnings_text = warnings.join("\n");

        content = content.push(text("Warnings:").size(14).style(warning_text_style));
        content = content.push(
            scrollable(
                container(text(warnings_text).size(12))
                    .padding(8)
                    .style(warning_box_style),
            )
            .height(Length::Fixed(150.0)),
        );
    }

    content = content.push(
        button(text("Close"))
            .on_press(Message::CloseDialog)
            .padding([8, 24]),
    );

    dialog_container(content, true, 600.0)
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
        .style(muted_text_style),
        text("Select suggestions to apply:")
            .size(13)
            .style(muted_text_style),
    ]
    .spacing(12);

    // Add suggestion items with checkboxes
    for (index, suggestion) in suggestions.iter().enumerate() {
        let rule_type = if suggestion.is_window_rule {
            "window"
        } else {
            "layer"
        };
        let patterns_preview = if suggestion.patterns.len() <= 3 {
            suggestion.patterns.join(", ")
        } else {
            format!(
                "{}, ... ({} more)",
                suggestion.patterns[..2].join(", "),
                suggestion.patterns.len() - 2
            )
        };

        let selected = suggestion.selected;

        content = content.push(
            container(
                row![
                    checkbox(suggestion.selected)
                        .on_toggle(move |_| Message::ConsolidationToggle(index)),
                    column![
                        text(&suggestion.description).size(13),
                        text(format!("Type: {} rules", rule_type))
                            .size(11)
                            .style(muted_text_style),
                        text(format!("Patterns: {}", patterns_preview))
                            .size(11)
                            .style(muted_text_style),
                        text(format!("Merged: {}", suggestion.merged_pattern))
                            .size(11)
                            .style(accent_text_style),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding(12)
            .width(Length::Fill)
            .style(move |theme: &iced::Theme| selection_item_style(theme, selected)),
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
        .spacing(12),
    );

    dialog_container(content, false, 600.0)
}

/// Diff view dialog showing before/after config changes
fn diff_view_dialog<'a>(title: &'a str, before: &'a str, after: &'a str) -> Element<'a, Message> {
    let content = column![
        text(title).size(24),
        text("Compare configuration changes before applying")
            .size(13)
            .style(muted_text_style),
        row![
            // Before panel
            column![
                text("Before").size(14).style(warning_text_style),
                scrollable(
                    container(text(before).size(12).font(iced::Font::MONOSPACE))
                        .padding(12)
                        .width(Length::Fill)
                        .style(|theme: &iced::Theme| {
                            let p = theme.extended_palette();
                            container::Style {
                                background: Some(p.danger.weak.color.into()),
                                text_color: Some(p.danger.weak.text),
                                border: Border {
                                    color: p.danger.strong.color,
                                    width: 1.0,
                                    radius: 4.0.into(),
                                },
                                ..Default::default()
                            }
                        })
                )
                .height(Length::Fixed(300.0))
            ]
            .spacing(8)
            .width(Length::Fill),
            // After panel
            column![
                text("After").size(14).style(success_text_style),
                scrollable(
                    container(text(after).size(12).font(iced::Font::MONOSPACE))
                        .padding(12)
                        .width(Length::Fill)
                        .style(|theme: &iced::Theme| {
                            let p = theme.extended_palette();
                            container::Style {
                                background: Some(p.success.weak.color.into()),
                                text_color: Some(p.success.weak.text),
                                border: Border {
                                    color: p.success.strong.color,
                                    width: 1.0,
                                    radius: 4.0.into(),
                                },
                                ..Default::default()
                            }
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

    // Themed + backdrop-dismissible, wider than the standard dialogs.
    dialog_container(content, true, 900.0)
}

/// Apply-then-confirm countdown dialog for risky live-applied changes
/// (output mode/disable, keybinding changes). Not dismissible via backdrop —
/// the user must explicitly Keep or Revert.
fn revert_countdown_dialog<'a>(description: &'a str, seconds_left: u8) -> Element<'a, Message> {
    let content = column![
        text("Keep these changes?").size(20),
        Space::new().height(12),
        text(format!(
            "{}. Reverting in {} s unless you confirm.",
            description, seconds_left
        ))
        .size(14),
        Space::new().height(24),
        row![
            button(text("Revert"))
                .on_press(Message::RevertNow)
                .style(button::secondary)
                .padding([8, 24]),
            Space::new().width(Length::Fill),
            button(text("Keep changes"))
                .on_press(Message::RevertKeep)
                .style(button::primary)
                .padding([8, 24]),
        ]
        .align_y(Alignment::Center),
    ];

    dialog_container(content, false, 600.0)
}

/// Wizard step shown when the user tries to skip setup before the niri include
/// line is present. Requires an explicit acknowledgement.
fn wizard_skip_warning<'a>() -> Column<'a, Message> {
    let code_line = container(text("include \"nirify/main.kdl\"").size(13))
        .padding(10)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.weak.color.into()),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: Some(palette.background.weak.text),
                ..Default::default()
            }
        })
        .width(Length::Fill);

    column![
        text("Skip setup?").size(22),
        Space::new().height(12),
        text(
            "Nothing you change in this app will affect niri until this line is \
             added to your niri config (~/.config/niri/config.kdl):"
        )
        .size(14),
        Space::new().height(8),
        code_line,
        Space::new().height(8),
        text("You can run setup later from the Tools page.").size(13),
        Space::new().height(24),
        row![
            button(text("Go back to setup"))
                .on_press(Message::WizardBack)
                .style(button::primary)
                .padding([8, 24]),
            Space::new().width(Length::Fill),
            button(text("Skip setup — I understand the app won't work yet"))
                .on_press(Message::WizardSkipConfirmed)
                .style(button::danger)
                .padding([8, 24]),
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(0)
}

/// Wraps content in a themed dialog box over a dimming backdrop.
///
/// When `dismissible` is true, clicking the backdrop (scrim) dispatches
/// `Message::CloseDialog`; clicks on the dialog box itself are swallowed.
fn dialog_container<'a>(
    content: impl Into<Element<'a, Message>>,
    dismissible: bool,
    width_px: f32,
) -> Element<'a, Message> {
    // Themed dialog box (adapts to light/dark theme).
    let dialog = container(content)
        .padding(32)
        .width(Length::Fixed(width_px))
        .max_height(700.0)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.base.color.into()),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 12.0.into(),
                },
                text_color: Some(palette.background.base.text),
                shadow: iced::Shadow {
                    color: IcedColor::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            }
        });

    let scrim_style = |_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(IcedColor {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.7,
        })),
        ..Default::default()
    };

    // Backdrop scrim: for a dismissible dialog a press closes it; for a modal
    // (non-dismissible) dialog the press is swallowed (NoOp) so it cannot fall
    // through to the UI underneath. Wrapping in `mouse_area` is the iced 0.14
    // modal pattern for capturing pointer events over the overlay.
    let scrim = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(scrim_style),
    )
    .on_press(if dismissible {
        Message::CloseDialog
    } else {
        Message::NoOp
    });

    // The dialog box itself always swallows clicks so presses on its
    // non-interactive areas never reach controls behind the overlay.
    let boxed = mouse_area(dialog).on_press(Message::NoOp);
    let centered = container(boxed)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill);

    stack![scrim, centered].into()
}
