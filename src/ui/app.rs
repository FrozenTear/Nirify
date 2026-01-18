//! Main application view
//!
//! Crystalline Dark - A refined settings interface built with Freya

use freya::prelude::*;

use crate::ui::components::section;
use crate::ui::nav::{footer, header, sidebar};
use crate::ui::state::{AppState, Category, NavGroup};
use crate::ui::theme::*;

/// Create the main application view
pub fn app_view(state: AppState) -> impl IntoElement {
    let current_category = use_state(|| Category::Appearance);
    let current_nav_group = use_state(|| NavGroup::Appearance);

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(BG_DEEP)
        .child(header(current_nav_group, current_category))
        .child(sidebar(current_nav_group, current_category))
        .child(content_area(state.clone(), current_category))
        .child(footer())
}

/// Content area with the current page
fn content_area(state: AppState, current_category: State<Category>) -> impl IntoElement {
    let cat = *current_category.read();
    let settings = state.get_settings();

    ScrollView::new()
        .child(
            rect()
                .width(Size::fill())
                .padding((SPACING_2XL, SPACING_2XL, SPACING_2XL, SPACING_2XL))
                .background(BG_BASE)
                .child(page_content(cat, settings)),
        )
}

/// Get the page content for a category
fn page_content(category: Category, settings: crate::config::Settings) -> impl IntoElement {
    section(
        category.label(),
        rect()
            .width(Size::fill())
            .spacing(SPACING_MD)
            .child(setting_row(
                "Gap Size",
                "Space between windows",
                format!("{} px", settings.appearance.gaps as i32),
            ))
            .child(setting_row(
                "Focus Ring",
                "Show ring around focused window",
                if settings.appearance.focus_ring_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
            ))
            .child(setting_row(
                "Border",
                "Show border around windows",
                if settings.appearance.border_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
            )),
    )
}

/// A simple setting row with label, description, and value
fn setting_row(label: &str, description: &str, value: impl ToString) -> impl IntoElement {
    rect()
        .horizontal()
        .width(Size::fill())
        .padding((SPACING_MD, 0.0, SPACING_MD, 0.0))
        .child(
            rect()
                .width(Size::fill())
                .child(
                    rect()
                        .color(TEXT_PRIMARY)
                        .font_size(FONT_SIZE_BASE)
                        .child(label.to_string()),
                )
                .child(
                    rect()
                        .color(TEXT_SECONDARY)
                        .font_size(FONT_SIZE_SM)
                        .child(description.to_string()),
                ),
        )
        .child(
            rect()
                .color(TEXT_PRIMARY)
                .font_size(FONT_SIZE_BASE)
                .child(value.to_string()),
        )
}
