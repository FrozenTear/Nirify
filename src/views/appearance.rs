//! Appearance settings view

use iced::widget::{column, container, scrollable};
use iced::Element;

use super::widgets::*;
use crate::config::models::AppearanceSettings;
use crate::messages::{AppearanceMessage, Message};

/// Creates the appearance settings view
pub fn view(settings: &AppearanceSettings) -> Element<'_, Message> {
    // Capture settings state for rendering
    let focus_ring_enabled = settings.focus_ring_enabled;
    let border_enabled = settings.border_enabled;
    let focus_ring_width = settings.focus_ring_width;
    let border_thickness = settings.border_thickness;
    let gaps = settings.gaps;
    let corner_radius = settings.corner_radius;

    // Clone ColorOrGradient values for gradient pickers
    let focus_ring_active = settings.focus_ring_active.clone();
    let focus_ring_inactive = settings.focus_ring_inactive.clone();
    let focus_ring_urgent = settings.focus_ring_urgent.clone();
    let border_active = settings.border_active.clone();
    let border_inactive = settings.border_inactive.clone();
    let border_urgent = settings.border_urgent.clone();

    let content = column![
        page_title("Focus Ring"),
        info_text(
            "The focus ring is a colored outline around the currently focused window. \
             It helps you see which window will receive keyboard input."
        ),
        toggle_row(
            "Enable focus ring",
            "Show a colored ring around the focused window",
            focus_ring_enabled,
            |value| Message::Appearance(AppearanceMessage::ToggleFocusRing(value)),
        ),
        // Width setting - greyed out when focus ring is disabled
        slider_row_with_state(
            "Ring width",
            "Thickness of the focus ring in pixels",
            focus_ring_width,
            1.0,
            20.0,
            " px",
            focus_ring_enabled,
            |value| Message::Appearance(AppearanceMessage::SetFocusRingWidth(value)),
        ),
        gradient_picker(
            "Active window color",
            "Color or gradient for the focus ring around the active window",
            &focus_ring_active,
            |msg| Message::Appearance(AppearanceMessage::FocusRingActive(msg)),
        ),
        gradient_picker(
            "Inactive window color",
            "Color or gradient for the focus ring around inactive windows",
            &focus_ring_inactive,
            |msg| Message::Appearance(AppearanceMessage::FocusRingInactive(msg)),
        ),
        gradient_picker(
            "Urgent window color",
            "Color or gradient for the focus ring around urgent windows (notifications)",
            &focus_ring_urgent,
            |msg| Message::Appearance(AppearanceMessage::FocusRingUrgent(msg)),
        ),
        section_header("Window Border"),
        info_text(
            "Window borders are drawn around the edges of each window. \
             Unlike focus rings, borders are inside the window geometry."
        ),
        toggle_row(
            "Enable border",
            "Show a colored border around windows",
            border_enabled,
            |value| Message::Appearance(AppearanceMessage::ToggleBorder(value)),
        ),
        // Thickness setting - greyed out when border is disabled
        slider_row_with_state(
            "Border thickness",
            "Width of the border in pixels",
            border_thickness,
            1.0,
            20.0,
            " px",
            border_enabled,
            |value| Message::Appearance(AppearanceMessage::SetBorderThickness(value)),
        ),
        gradient_picker(
            "Active window border",
            "Color or gradient for the border around the active window",
            &border_active,
            |msg| Message::Appearance(AppearanceMessage::BorderActive(msg)),
        ),
        gradient_picker(
            "Inactive window border",
            "Color or gradient for the border around inactive windows",
            &border_inactive,
            |msg| Message::Appearance(AppearanceMessage::BorderInactive(msg)),
        ),
        gradient_picker(
            "Urgent window border",
            "Color or gradient for the border around urgent windows (notifications)",
            &border_urgent,
            |msg| Message::Appearance(AppearanceMessage::BorderUrgent(msg)),
        ),
        section_header("Layout"),
        slider_row(
            "Window gaps",
            "Spacing between windows in pixels",
            gaps,
            0.0,
            64.0,
            " px",
            |value| Message::Appearance(AppearanceMessage::SetGaps(value)),
        ),
        slider_row(
            "Corner radius",
            "Rounded corners for windows in pixels (0 = square corners)",
            corner_radius,
            0.0,
            32.0,
            " px",
            |value| Message::Appearance(AppearanceMessage::SetCornerRadius(value)),
        ),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
        .into()
}
