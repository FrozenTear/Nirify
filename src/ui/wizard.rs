//! First-run wizard for niri-settings
//!
//! Guides users through initial setup:
//! - Step 0: Welcome screen with feature highlights
//! - Step 1: Include line setup (auto add or manual)
//! - Step 2: Completion with import summary

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{Container, Empty, Label, Stack};
use std::sync::Arc;

use crate::config::{
    save_settings, smart_replace_config, ConfigPaths, ImportResult, Settings,
};
use crate::ui::theme::{
    theme, FONT_SIZE_BASE, FONT_SIZE_LG, FONT_SIZE_SM, FONT_SIZE_XS, RADIUS_LG, RADIUS_MD,
    SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
};
use std::sync::Mutex;

/// Wizard state
#[derive(Clone)]
pub struct WizardState {
    /// Current step (0-2)
    pub step: RwSignal<usize>,
    /// Whether the wizard is visible
    pub visible: RwSignal<bool>,
    /// Whether the include line exists
    pub has_include_line: RwSignal<bool>,
    /// Import result from first run
    pub import_result: RwSignal<Option<ImportResult>>,
    /// Config paths
    pub paths: Arc<ConfigPaths>,
    /// Settings reference for saving
    pub settings: Arc<Mutex<Settings>>,
    /// Error message if setup fails
    pub error_message: RwSignal<Option<String>>,
}

impl WizardState {
    pub fn new(
        paths: Arc<ConfigPaths>,
        settings: Arc<Mutex<Settings>>,
        import_result: Option<ImportResult>,
    ) -> Self {
        let has_include = paths.has_include_line();
        Self {
            step: RwSignal::new(0),
            visible: RwSignal::new(true),
            has_include_line: RwSignal::new(has_include),
            import_result: RwSignal::new(import_result),
            paths,
            settings,
            error_message: RwSignal::new(None),
        }
    }
}

/// Create the first-run wizard overlay
pub fn wizard_view(state: WizardState) -> impl IntoView {
    let visible = state.visible;

    dyn_view(move || {
        if visible.get() {
            wizard_modal(state.clone()).into_any()
        } else {
            Empty::new().into_any()
        }
    })
    .style(|s| s.width_full().height_full())
}

/// The wizard modal dialog
fn wizard_modal(state: WizardState) -> impl IntoView {
    let step = state.step;
    let state_for_content = state.clone();
    let state_for_nav = state.clone();

    // Modal overlay background - use Stack for proper flex layout
    Stack::vertical((
        // Dialog box centered in overlay
        Container::new(
            Stack::vertical((
                // Progress indicator
                progress_bar(step),
                // Step content
                dyn_view(move || {
                    match step.get() {
                        0 => step_welcome(state_for_content.clone()).into_any(),
                        1 => step_include_line(state_for_content.clone()).into_any(),
                        _ => step_complete(state_for_content.clone()).into_any(),
                    }
                })
                .style(|s| s.flex_grow(1.0).width_full()),
                // Navigation buttons
                navigation_buttons(state_for_nav),
            ))
            .style(|s| s.padding(SPACING_LG).gap(SPACING_MD).width_full().height_full()),
        )
        .style(move |s| {
            let t = theme();
            s.width(550.0)
                .height(450.0)
                .background(t.bg_elevated)
                .border_radius(RADIUS_LG)
                .box_shadow_blur(30.0)
                .box_shadow_color(t.border_subtle.with_alpha(0.5))
        }),
    ))
    .style(move |s| {
        let t = theme();
        s.width_full()
            .height_full()
            .flex_grow(1.0)
            .background(t.bg_base.with_alpha(0.85))
            .justify_center()
            .items_center()
    })
}

/// Progress bar showing current step
fn progress_bar(step: RwSignal<usize>) -> impl IntoView {
    Stack::horizontal(
        (0..3)
            .map(move |i| {
                Empty::new().style(move |s| {
                    let t = theme();
                    let is_active = i <= step.get();
                    s.height(8.0)
                        .flex_grow(1.0)
                        .border_radius(4.0)
                        .background(if is_active { t.accent } else { t.border_subtle })
                })
            })
            .collect::<Vec<_>>(),
    )
    .style(|s| s.width_full().gap(SPACING_XS))
}

/// Step 0: Welcome screen
fn step_welcome(state: WizardState) -> impl IntoView {
    let _ = state; // Keep for consistency

    Stack::vertical((
        // Title
        Label::new("Welcome to Niri Settings").style(move |s| {
            s.font_size(28.0)
                .font_bold()
                .color(theme().text_primary)
                .margin_bottom(SPACING_MD)
        }),
        // Description
        Label::new("This application helps you configure niri, the scrollable-tiling Wayland compositor, without editing config files directly.")
            .style(move |s| {
                s.font_size(FONT_SIZE_LG)
                    .color(theme().text_secondary)
                    .margin_bottom(SPACING_LG)
            }),
        // Feature list
        Stack::vertical((
            feature_item("Live preview - changes apply immediately"),
            feature_item("Safe - backups created before any changes"),
            feature_item("Organized - settings stored in separate files"),
        ))
        .style(|s| s.gap(SPACING_SM)),
    ))
    .style(|s| s.items_center().justify_center().width_full().padding(SPACING_LG))
}

/// Feature item with green dot
fn feature_item(text: &'static str) -> impl IntoView {
    Stack::horizontal((
        // Green dot
        Empty::new().style(move |s| {
            s.width(8.0)
                .height(8.0)
                .border_radius(4.0)
                .background(theme().success)
        }),
        Label::new(text).style(move |s| s.font_size(FONT_SIZE_BASE).color(theme().text_secondary)),
    ))
    .style(|s| s.gap(SPACING_SM).items_center())
}

/// Step 1: Include line setup
fn step_include_line(state: WizardState) -> impl IntoView {
    let has_include = state.has_include_line;
    let error_msg = state.error_message;

    Stack::vertical((
        // Title
        Label::new("Connect to Your Config").style(move |s| {
            s.font_size(24.0)
                .font_bold()
                .color(theme().text_primary)
                .margin_bottom(SPACING_MD)
        }),
        // Description
        Label::new("Niri Settings manages common settings (layout, input, appearance). Your custom settings will be preserved.")
            .style(move |s| {
                s.font_size(FONT_SIZE_BASE)
                    .color(theme().text_secondary)
                    .margin_bottom(SPACING_MD)
            }),
        // Include line display
        Container::new(
            Label::new("include \"niri-settings/main.kdl\"").style(move |s| {
                s.font_size(FONT_SIZE_BASE)
                    .color(theme().accent)
                    .font_family("monospace".to_string())
            }),
        )
        .style(move |s| {
            let t = theme();
            s.padding(SPACING_MD)
                .border_radius(RADIUS_MD)
                .background(t.bg_base)
                .width_full()
                .margin_bottom(SPACING_MD)
        }),
        // Status or action buttons
        dyn_view(move || {
            if has_include.get() {
                // Already configured
                already_configured_view().into_any()
            } else {
                // Need to add include line
                add_include_buttons(state.clone()).into_any()
            }
        }),
        // Error message display
        dyn_view(move || {
            if let Some(err) = error_msg.get() {
                Container::new(Label::new(err).style(move |s| {
                    s.font_size(FONT_SIZE_SM).color(theme().error)
                }))
                .style(move |s| {
                    let t = theme();
                    s.padding(SPACING_SM)
                        .border_radius(RADIUS_MD)
                        .background(t.error.with_alpha(0.1))
                        .margin_top(SPACING_MD)
                })
                .into_any()
            } else {
                Empty::new().into_any()
            }
        }),
    ))
    .style(|s| s.width_full().padding(SPACING_MD))
}

/// Already configured status
fn already_configured_view() -> impl IntoView {
    Stack::horizontal((
        // Checkmark circle
        Container::new(Label::new("+").style(move |s| {
            s.font_size(14.0)
                .font_bold()
                .color(theme().text_primary)
        }))
        .style(move |s| {
            s.width(20.0)
                .height(20.0)
                .border_radius(10.0)
                .background(theme().success)
                .justify_center()
                .items_center()
        }),
        Label::new("Already configured! Your config includes our settings.").style(move |s| {
            s.font_size(FONT_SIZE_BASE).color(theme().success)
        }),
    ))
    .style(|s| s.gap(SPACING_SM).items_center().justify_center())
}

/// Buttons to add include line
fn add_include_buttons(state: WizardState) -> impl IntoView {
    let step = state.step;
    let has_include = state.has_include_line;
    let error_msg = state.error_message;
    let paths = state.paths.clone();
    let settings = state.settings.clone();

    Stack::vertical((
        Label::new("Choose how to add this line:").style(move |s| {
            s.font_size(FONT_SIZE_BASE)
                .color(theme().text_secondary)
                .margin_bottom(SPACING_SM)
        }),
        Stack::horizontal((
            // Add Automatically button
            {
                let paths = paths.clone();
                let settings = settings.clone();
                Container::new(Label::new("Add Automatically").style(move |s| {
                    s.font_size(FONT_SIZE_SM).color(theme().text_primary)
                }))
                .style(move |s| {
                    let t = theme();
                    s.padding_horiz(SPACING_MD)
                        .padding_vert(SPACING_SM)
                        .border_radius(RADIUS_MD)
                        .background(t.accent)
                        .cursor(floem::style::CursorStyle::Pointer)
                })
                .on_click_stop(move |_| {
                    // Perform smart replace
                    let backup_dir = paths
                        .niri_config
                        .parent()
                        .map(|p| p.join(".backup"))
                        .unwrap_or_else(|| paths.backup_dir.clone());

                    match smart_replace_config(&paths.niri_config, &backup_dir) {
                        Ok(result) => {
                            log::info!(
                                "Config setup: {} replaced, {} preserved",
                                result.replaced_count,
                                result.preserved_count
                            );

                            // Save settings so files exist
                            if let Ok(s) = settings.lock() {
                                if let Err(e) = save_settings(&paths, &s) {
                                    log::error!("Failed to save settings: {}", e);
                                    error_msg.set(Some(format!("Failed to save: {}", e)));
                                    return;
                                }
                            }

                            has_include.set(true);
                            step.set(2);
                            error_msg.set(None);
                        }
                        Err(e) => {
                            log::error!("Failed to setup config: {}", e);
                            error_msg.set(Some(format!("Setup failed: {}", e)));
                        }
                    }
                })
            },
            // Manual button
            Container::new(Label::new("I'll Add It Myself").style(move |s| {
                s.font_size(FONT_SIZE_SM).color(theme().text_primary)
            }))
            .style(move |s| {
                let t = theme();
                s.padding_horiz(SPACING_MD)
                    .padding_vert(SPACING_SM)
                    .border_radius(RADIUS_MD)
                    .background(t.bg_surface)
                    .border(1.0)
                    .border_color(t.border_subtle)
                    .cursor(floem::style::CursorStyle::Pointer)
            })
            .on_click_stop(move |_| {
                step.set(2);
            }),
        ))
        .style(|s| s.gap(SPACING_MD)),
        // Helper text
        Label::new("This will reorganize your config.kdl and create a backup").style(move |s| {
            s.font_size(FONT_SIZE_XS)
                .color(theme().text_tertiary)
                .margin_top(SPACING_SM)
        }),
    ))
}

/// Step 2: Completion
fn step_complete(state: WizardState) -> impl IntoView {
    let has_include = state.has_include_line;
    let import_result = state.import_result;

    Stack::vertical((
        // Success checkmark
        Container::new(Label::new("✓").style(move |s| {
            s.font_size(32.0)
                .font_bold()
                .color(theme().text_primary)
        }))
        .style(move |s| {
            s.width(64.0)
                .height(64.0)
                .border_radius(32.0)
                .background(theme().success)
                .justify_center()
                .items_center()
                .margin_bottom(SPACING_MD)
        }),
        // Title
        Label::new("You're All Set!").style(move |s| {
            s.font_size(24.0)
                .font_bold()
                .color(theme().text_primary)
                .margin_bottom(SPACING_SM)
        }),
        // Status message
        dyn_view(move || {
            let msg = if has_include.get() {
                "Your configuration is connected. Changes apply immediately when niri reloads."
            } else {
                "Remember to add the include line to your config.kdl before changes take effect."
            };
            Label::new(msg)
                .style(move |s| {
                    s.font_size(FONT_SIZE_BASE)
                        .color(theme().text_secondary)
                        .margin_bottom(SPACING_MD)
                })
                .into_any()
        }),
        // Import summary
        dyn_view(move || {
            if let Some(result) = import_result.get() {
                if result.has_imports() {
                    import_summary_view(result).into_any()
                } else {
                    Label::new("Using default settings (no existing config found)")
                        .style(move |s| {
                            s.font_size(FONT_SIZE_SM).color(theme().text_tertiary)
                        })
                        .into_any()
                }
            } else {
                Empty::new().into_any()
            }
        }),
        // Tip
        Label::new("Tip: Press Super+Shift+R to reload niri's config.").style(move |s| {
            s.font_size(FONT_SIZE_SM)
                .color(theme().text_tertiary)
                .margin_top(SPACING_MD)
        }),
    ))
    .style(|s| s.items_center().justify_center().width_full().padding(SPACING_LG))
}

/// Import summary display
fn import_summary_view(result: ImportResult) -> impl IntoView {
    Container::new(
        Stack::vertical((
            Label::new("Imported from your config:").style(move |s| {
                s.font_size(FONT_SIZE_XS).color(theme().text_tertiary)
            }),
            Label::new(result.summary()).style(move |s| {
                s.font_size(FONT_SIZE_SM).color(theme().text_secondary)
            }),
        ))
        .style(|s| s.gap(SPACING_XS)),
    )
    .style(move |s| {
        let t = theme();
        s.padding(SPACING_MD)
            .border_radius(RADIUS_MD)
            .background(t.bg_base)
            .width_full()
    })
}

/// Navigation buttons at bottom of wizard
fn navigation_buttons(state: WizardState) -> impl IntoView {
    let step = state.step;
    let visible = state.visible;
    let has_include = state.has_include_line;

    Stack::horizontal((
        // Skip/Cancel button (steps 0-1)
        dyn_view(move || {
            if step.get() < 2 {
                Container::new(Label::new("Skip Setup").style(move |s| {
                    s.font_size(FONT_SIZE_SM).color(theme().text_secondary)
                }))
                .style(move |s| {
                    let t = theme();
                    s.padding_horiz(SPACING_MD)
                        .padding_vert(SPACING_SM)
                        .border_radius(RADIUS_MD)
                        .background(t.bg_surface)
                        .cursor(floem::style::CursorStyle::Pointer)
                })
                .on_click_stop(move |_| {
                    visible.set(false);
                })
                .into_any()
            } else {
                Empty::new().into_any()
            }
        }),
        // Spacer
        Empty::new().style(|s| s.flex_grow(1.0)),
        // Back button (step 1 only)
        dyn_view(move || {
            if step.get() == 1 {
                Container::new(Label::new("Back").style(move |s| {
                    s.font_size(FONT_SIZE_SM).color(theme().text_primary)
                }))
                .style(move |s| {
                    let t = theme();
                    s.padding_horiz(SPACING_MD)
                        .padding_vert(SPACING_SM)
                        .border_radius(RADIUS_MD)
                        .background(t.bg_surface)
                        .border(1.0)
                        .border_color(t.border_subtle)
                        .cursor(floem::style::CursorStyle::Pointer)
                        .margin_right(SPACING_SM)
                })
                .on_click_stop(move |_| {
                    step.set(0);
                })
                .into_any()
            } else {
                Empty::new().into_any()
            }
        }),
        // Next/Continue/Finish button
        dyn_view(move || {
            let (label, action): (&str, Box<dyn Fn() + 'static>) = match step.get() {
                0 => ("Get Started", Box::new(move || step.set(1))),
                1 if has_include.get() => ("Continue", Box::new(move || step.set(2))),
                1 => return Empty::new().into_any(), // No button when include line buttons shown
                _ => ("Start Using Niri Settings", Box::new(move || visible.set(false))),
            };

            Container::new(Label::new(label).style(move |s| {
                s.font_size(FONT_SIZE_SM).color(theme().text_primary)
            }))
            .style(move |s| {
                let t = theme();
                s.padding_horiz(SPACING_MD)
                    .padding_vert(SPACING_SM)
                    .border_radius(RADIUS_MD)
                    .background(t.accent)
                    .cursor(floem::style::CursorStyle::Pointer)
            })
            .on_click_stop(move |_| {
                action();
            })
            .into_any()
        }),
    ))
    .style(|s| s.width_full())
}
