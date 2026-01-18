//! Setting row components - refined controls for the Crystalline Dark theme
//!
//! Each row follows a two-column layout:
//! - Left: Label + optional description
//! - Right: Interactive control (toggle, slider, color picker, etc.)
//!
//! All components support an optional on_change callback for auto-save wiring.

use floem::event::{Event, EventListener};
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::ui_events::pointer::{PointerButtonEvent, PointerEvent, PointerUpdate};
use floem::views::{text_input, Container, Empty, Label, Stack};
use std::rc::Rc;

use crate::ui::theme::{
    color_input_container_style, color_swatch_style, icon_button_style, parse_hex_color,
    setting_row_style, text_input_style, ACCENT, BG_ELEVATED, BORDER,
    BORDER_SUBTLE, FONT_SIZE_BASE, FONT_SIZE_SM, RADIUS_FULL, RADIUS_SM, SPACING_MD, SPACING_SM,
    SPACING_XS, SURFACE1, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY,
};

/// Callback type for value changes
pub type OnChange<T> = Option<Rc<dyn Fn(T)>>;

// ============================================================================
// Toggle Row - Custom styled switch
// ============================================================================

/// A setting row with a custom toggle switch
pub fn toggle_row(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<bool>,
) -> impl IntoView {
    toggle_row_with_callback(label_text, description, value, None)
}

/// A setting row with a custom toggle switch and change callback
pub fn toggle_row_with_callback(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<bool>,
    on_change: OnChange<bool>,
) -> impl IntoView {
    Stack::horizontal((
        // Left side: label + description
        setting_label(label_text, description),
        // Right side: custom toggle switch
        toggle_switch_with_callback(value, on_change),
    ))
    .style(setting_row_style)
}

/// Custom toggle switch component
fn toggle_switch_with_callback(value: RwSignal<bool>, on_change: OnChange<bool>) -> impl IntoView {
    let is_on = move || value.get();

    Container::new(
        // Toggle knob
        Container::new(Empty::new()).style(move |s| {
            let base = s
                .width(18.0)
                .height(18.0)
                .border_radius(RADIUS_FULL)
                .background(TEXT_PRIMARY);

            if is_on() {
                base.margin_left(20.0)
            } else {
                base.margin_left(2.0)
            }
        }),
    )
    .style(move |s| {
        let base = s
            .width(44.0)
            .height(24.0)
            .border_radius(RADIUS_FULL)
            .border(1.0)
            .items_center();

        if is_on() {
            base.background(ACCENT).border_color(ACCENT)
        } else {
            base.background(SURFACE1).border_color(BORDER)
        }
    })
    .on_click_stop(move |_| {
        let new_val = !value.get();
        value.set(new_val);
        if let Some(ref cb) = on_change {
            cb(new_val);
        }
    })
}

// ============================================================================
// Slider Row - Styled slider with value display
// ============================================================================

/// A setting row with a slider and value badge
pub fn slider_row(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<f64>,
    min: f64,
    max: f64,
    step: f64,
    unit: &'static str,
) -> impl IntoView {
    slider_row_with_callback(label_text, description, value, min, max, step, unit, None)
}

/// A setting row with a slider, editable value input, and change callback
#[allow(clippy::too_many_arguments)]
pub fn slider_row_with_callback(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<f64>,
    min: f64,
    max: f64,
    step: f64,
    unit: &'static str,
    on_change: OnChange<f64>,
) -> impl IntoView {
    // Create a string signal for the text input, synced with the value
    let text_value = RwSignal::new(format!("{}", value.get() as i32));

    // Keep text in sync when value changes from slider
    floem::reactive::Effect::new(move |_| {
        let v = value.get();
        text_value.set(format!("{}", v as i32));
    });

    let on_change_input = on_change.clone();

    Stack::horizontal((
        // Left side: label + description
        setting_label(label_text, description),
        // Right side: slider + editable value input
        Stack::horizontal((
            // Slider track with knob (clamped to visual range)
            slider_control_with_callback(value, min, max, step, on_change),
            // Editable value input (allows values beyond slider range)
            Stack::horizontal((
                text_input(text_value)
                    .on_event_stop(EventListener::FocusLost, move |_| {
                        // Parse and apply the value on blur
                        if let Ok(v) = text_value.get().parse::<f64>() {
                            // Allow extended range beyond slider, clamped to reasonable max
                            let clamped = v.clamp(min.min(0.0), max.max(200.0));
                            value.set(clamped);
                            text_value.set(format!("{}", clamped as i32));
                            if let Some(ref cb) = on_change_input {
                                cb(clamped);
                            }
                        } else {
                            // Reset to current value if parse fails
                            text_value.set(format!("{}", value.get() as i32));
                        }
                    })
                    .style(|s| {
                        s.width(45.0)
                            .padding_vert(SPACING_XS)
                            .padding_horiz(SPACING_SM)
                            .background(BG_ELEVATED)
                            .border_radius(RADIUS_SM)
                            .border(1.0)
                            .border_color(BORDER_SUBTLE)
                            .color(TEXT_PRIMARY)
                            .font_size(FONT_SIZE_SM)
                    }),
                Label::derived(move || unit.to_string())
                    .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
            ))
            .style(|s| s.items_center().gap(SPACING_XS)),
        ))
        .style(|s| s.items_center().gap(SPACING_MD)),
    ))
    .style(setting_row_style)
}

/// Slider dimensions
const SLIDER_TRACK_WIDTH: f64 = 140.0;
const SLIDER_TRACK_HEIGHT: f64 = 6.0;
const SLIDER_HANDLE_SIZE: f64 = 16.0;
const SLIDER_PADDING: f64 = SLIDER_HANDLE_SIZE / 2.0; // Padding on each side for handle

/// Custom slider control - draggable track with visual feedback
fn slider_control_with_callback(
    value: RwSignal<f64>,
    min: f64,
    max: f64,
    step: f64,
    on_change: OnChange<f64>,
) -> impl IntoView {
    let percentage = move || ((value.get() - min) / (max - min)).clamp(0.0, 1.0);
    let is_dragging = RwSignal::new(false);

    // Clone callbacks for different event handlers
    let on_change_down = on_change.clone();
    let on_change_move = on_change.clone();
    let on_change_up = on_change;

    // The total interactive width includes padding for the handle on both sides
    let total_width = SLIDER_TRACK_WIDTH + SLIDER_PADDING * 2.0;

    // Helper to calculate value from x position (accounting for handle padding)
    let calc_value = move |x: f64| -> f64 {
        // x is relative to the interactive container
        // Subtract left padding to get position relative to track start
        // Then divide by track width to get percentage
        let track_x = x - SLIDER_PADDING;
        let pct = (track_x / SLIDER_TRACK_WIDTH).clamp(0.0, 1.0);
        let raw_val = min + pct * (max - min);
        // Round to nearest step
        let stepped = ((raw_val - min) / step).round() * step + min;
        stepped.clamp(min, max)
    };

    // Single container that handles all interaction
    // Track and handle are positioned inside
    Stack::horizontal((
        // Left padding spacer
        Empty::new().style(|s| s.width(SLIDER_PADDING)),
        // Track with fill
        Container::new(
            Container::new(Empty::new()).style(move |s| {
                let pct = percentage();
                s.width(pct * SLIDER_TRACK_WIDTH)
                    .height(SLIDER_TRACK_HEIGHT)
                    .border_radius(RADIUS_FULL)
                    .background(ACCENT)
            }),
        )
        .style(|s| {
            s.width(SLIDER_TRACK_WIDTH)
                .height(SLIDER_TRACK_HEIGHT)
                .border_radius(RADIUS_FULL)
                .background(SURFACE1)
        }),
        // Handle - positioned relative to track end, moved back by percentage
        Container::new(Empty::new()).style(move |s| {
            let pct = percentage();
            // Handle starts after track, move it left based on inverse percentage
            let handle_offset = -(1.0 - pct) * SLIDER_TRACK_WIDTH - SLIDER_HANDLE_SIZE / 2.0;
            s.width(SLIDER_HANDLE_SIZE)
                .height(SLIDER_HANDLE_SIZE)
                .border_radius(RADIUS_FULL)
                .background(TEXT_PRIMARY)
                .border(2.0)
                .border_color(ACCENT)
                .margin_left(handle_offset)
        }),
        // Right padding spacer
        Empty::new().style(|s| s.width(SLIDER_PADDING)),
    ))
    .style(move |s| {
        s.width(total_width)
            .height(SLIDER_HANDLE_SIZE)
            .items_center()
            .cursor(floem::style::CursorStyle::Pointer)
    })
    .on_event(EventListener::PointerDown, move |e| {
        if let Event::Pointer(PointerEvent::Down(PointerButtonEvent { state, .. })) = e {
            is_dragging.set(true);
            let new_val = calc_value(state.logical_point().x);
            value.set(new_val);
            if let Some(ref cb) = on_change_down {
                cb(new_val);
            }
        }
        floem::event::EventPropagation::Stop
    })
    .on_event(EventListener::PointerMove, move |e| {
        if is_dragging.get() {
            if let Event::Pointer(PointerEvent::Move(PointerUpdate { current, .. })) = e {
                let new_val = calc_value(current.logical_point().x);
                value.set(new_val);
                if let Some(ref cb) = on_change_move {
                    cb(new_val);
                }
            }
        }
        floem::event::EventPropagation::Continue
    })
    .on_event(EventListener::PointerUp, move |e| {
        if is_dragging.get() {
            is_dragging.set(false);
            if let Event::Pointer(PointerEvent::Up(PointerButtonEvent { state, .. })) = e {
                let new_val = calc_value(state.logical_point().x);
                value.set(new_val);
                if let Some(ref cb) = on_change_up {
                    cb(new_val);
                }
            }
        }
        floem::event::EventPropagation::Stop
    })
}

// ============================================================================
// Color Row - Color swatch with hex input
// ============================================================================

/// A setting row with color swatch and hex input
pub fn color_row(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<String>,
) -> impl IntoView {
    color_row_with_callback(label_text, description, value, None)
}

/// A setting row with color swatch, hex input, and change callback
pub fn color_row_with_callback(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<String>,
    on_change: OnChange<String>,
) -> impl IntoView {
    let on_change_clear = on_change.clone();

    Stack::horizontal((
        // Left side: label + description
        setting_label(label_text, description),
        // Right side: color swatch + hex input
        Stack::horizontal((
            // Color swatch preview
            Container::new(Empty::new()).style(move |s| {
                let hex = value.get();
                let color = parse_hex_color(&hex);
                color_swatch_style(s).background(color)
            }),
            // Hex input container
            Stack::horizontal((
                text_input(value)
                    .on_event_stop(floem::event::EventListener::FocusLost, move |_| {
                        if let Some(ref cb) = on_change {
                            cb(value.get());
                        }
                    })
                    .style(|s| {
                        s.width(80.0)
                            .padding(SPACING_XS)
                            .background(BG_ELEVATED)
                            .border_radius(RADIUS_SM)
                            .color(TEXT_PRIMARY)
                            .font_size(FONT_SIZE_SM)
                            .border(0.0)
                    }),
                // Clear button
                Label::derived(|| "✕".to_string())
                    .style(icon_button_style)
                    .on_click_stop(move |_| {
                        value.set(String::new());
                        if let Some(ref cb) = on_change_clear {
                            cb(String::new());
                        }
                    }),
            ))
            .style(color_input_container_style),
        ))
        .style(|s| s.items_center().gap(SPACING_SM)),
    ))
    .style(setting_row_style)
}

// ============================================================================
// Text Row - Text input field
// ============================================================================

/// A setting row with a text input field
pub fn text_row(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<String>,
    placeholder: &'static str,
) -> impl IntoView {
    Stack::horizontal((
        setting_label(label_text, description),
        text_input(value)
            .placeholder(placeholder)
            .style(|s| text_input_style(s).width(200.0)),
    ))
    .style(setting_row_style)
}

// ============================================================================
// Dropdown Row - Select field
// ============================================================================

/// A setting row with a dropdown selector
pub fn dropdown_row<T: Clone + PartialEq + 'static>(
    label_text: &'static str,
    description: Option<&'static str>,
    options: Vec<(&'static str, T)>,
    selected: RwSignal<T>,
) -> impl IntoView {
    let options_for_label = options.clone();

    Stack::horizontal((
        setting_label(label_text, description),
        // Dropdown display (simplified as styled label)
        Stack::horizontal((
            Label::derived(move || {
                let current = selected.get();
                options_for_label
                    .iter()
                    .find(|(_, v)| *v == current)
                    .map(|(name, _)| (*name).to_string())
                    .unwrap_or_else(|| "Select...".to_string())
            })
            .style(|s| s.color(TEXT_PRIMARY).font_size(FONT_SIZE_SM)),
            // Chevron indicator
            Label::derived(|| "▼".to_string())
                .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
        ))
        .style(|s| {
            s.padding_horiz(SPACING_MD)
                .padding_vert(SPACING_SM)
                .background(BG_ELEVATED)
                .border_radius(RADIUS_SM)
                .border(1.0)
                .border_color(BORDER_SUBTLE)
                .min_width(160.0)
                .gap(SPACING_SM)
                .items_center()
                .justify_between()
        }),
    ))
    .style(setting_row_style)
}

// ============================================================================
// Helper Components
// ============================================================================

/// Create the label + description column for setting rows
fn setting_label(label_text: &'static str, description: Option<&'static str>) -> impl IntoView {
    Stack::vertical((
        Label::derived(move || label_text.to_string()).style(|s| {
            s.color(TEXT_PRIMARY)
                .font_size(FONT_SIZE_BASE)
        }),
        match description {
            Some(desc) => Label::derived(move || desc.to_string())
                .style(|s| {
                    s.font_size(FONT_SIZE_SM)
                        .color(TEXT_SECONDARY)
                        .margin_top(SPACING_XS)
                })
                .into_any(),
            None => Empty::new().into_any(),
        },
    ))
    .style(|s| s.flex_grow(1.0))
}
