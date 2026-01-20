//! Window picker modal component for selecting running windows
//!
//! Used by window rules to easily select app_id and title from running windows.

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Container, Empty, Label, Scroll, Stack};
use std::rc::Rc;

use crate::ipc::{get_windows, WindowInfo};
use crate::ui::theme::{
    theme, FONT_SIZE_BASE, FONT_SIZE_LG, FONT_SIZE_SM, FONT_SIZE_XS, RADIUS_LG, RADIUS_MD,
    RADIUS_SM, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
};

/// State for the window picker modal
#[derive(Clone)]
pub struct WindowPickerState {
    /// Whether the modal is visible
    pub visible: RwSignal<bool>,
    /// Error message if fetch fails
    pub error: RwSignal<Option<String>>,
    /// List of fetched windows
    pub windows: RwSignal<Vec<WindowInfo>>,
    /// Search/filter text
    pub search: RwSignal<String>,
    /// Callback when a window is selected (app_id, title)
    on_select: Rc<dyn Fn(String, String) + 'static>,
}

impl WindowPickerState {
    /// Create a new window picker state with a selection callback
    pub fn new<F>(on_select: F) -> Self
    where
        F: Fn(String, String) + 'static,
    {
        Self {
            visible: RwSignal::new(false),
            error: RwSignal::new(None),
            windows: RwSignal::new(Vec::new()),
            search: RwSignal::new(String::new()),
            on_select: Rc::new(on_select),
        }
    }

    /// Open the modal and fetch windows synchronously
    pub fn open(&self) {
        self.visible.set(true);
        self.error.set(None);
        self.search.set(String::new());

        // Fetch windows synchronously (IPC is fast with 2s timeout)
        match get_windows() {
            Ok(win_list) => {
                self.windows.set(win_list);
            }
            Err(e) => {
                self.error.set(Some(format!("{}", e)));
            }
        }
    }

    /// Close the modal
    pub fn close(&self) {
        self.visible.set(false);
    }

    /// Select a window and close the modal
    pub fn select(&self, app_id: String, title: String) {
        (self.on_select)(app_id, title);
        self.close();
    }
}

/// Create the window picker modal overlay
/// NOTE: This must be placed inside a Stack::new() with Position::Relative parent.
/// The absolute positioning is applied only when visible to avoid blocking clicks.
pub fn window_picker_modal(state: WindowPickerState) -> impl IntoView {
    let visible = state.visible;

    // Only apply Position::Absolute when visible, otherwise Empty doesn't block
    dyn_view(move || {
        if visible.get() {
            window_picker_content(state.clone())
                .style(|s| {
                    s.position(floem::style::Position::Absolute)
                        .inset(0.0)
                        .width_full()
                        .height_full()
                })
                .into_any()
        } else {
            Empty::new().into_any()
        }
    })
}

/// The window picker modal content
fn window_picker_content(state: WindowPickerState) -> impl IntoView {
    let visible = state.visible;
    let error = state.error;
    let windows = state.windows;
    let search = state.search;

    // Modal overlay background
    Stack::vertical((Container::new(
        Stack::vertical((
            // Header with title and close button
            Stack::horizontal((
                Label::new("Pick a Window").style(move |s| {
                    let t = theme();
                    s.font_size(FONT_SIZE_LG)
                        .font_bold()
                        .color(t.text_primary)
                }),
                Empty::new().style(|s| s.flex_grow(1.0)),
                // Close button
                Container::new(Label::new("✕").style(move |s| {
                    let t = theme();
                    s.font_size(FONT_SIZE_SM).color(t.text_muted)
                }))
                .style(move |s| {
                    let t = theme();
                    s.width(28.0)
                        .height(28.0)
                        .border_radius(RADIUS_SM)
                        .items_center()
                        .justify_center()
                        .cursor(floem::style::CursorStyle::Pointer)
                        .hover(|s| s.background(t.hover_bg))
                })
                .on_click_stop(move |_| {
                    visible.set(false);
                }),
            ))
            .style(|s| s.width_full().items_center().margin_bottom(SPACING_MD)),
            // Search input
            text_input(search)
                .placeholder("Search windows...")
                .style(move |s| {
                    let t = theme();
                    s.width_full()
                        .padding(SPACING_SM)
                        .background(t.bg_base)
                        .border_radius(RADIUS_MD)
                        .border(1.0)
                        .border_color(t.border_subtle)
                        .color(t.text_primary)
                        .font_size(FONT_SIZE_BASE)
                        .margin_bottom(SPACING_MD)
                }),
            // Content area (error or window list)
            dyn_view(move || {
                if let Some(err) = error.get() {
                    // Error state
                    Container::new(
                        Stack::vertical((
                            Label::new("Could not fetch windows").style(move |s| {
                                let t = theme();
                                s.font_size(FONT_SIZE_SM).font_bold().color(t.error)
                            }),
                            Label::new(err).style(move |s| {
                                let t = theme();
                                s.font_size(FONT_SIZE_XS).color(t.text_muted)
                            }),
                            Label::new("Make sure niri is running").style(move |s| {
                                let t = theme();
                                s.font_size(FONT_SIZE_XS)
                                    .color(t.text_tertiary)
                                    .margin_top(SPACING_SM)
                            }),
                        ))
                        .style(|s| s.gap(SPACING_XS).items_center()),
                    )
                    .style(move |s| {
                        let t = theme();
                        s.width_full()
                            .padding(SPACING_LG)
                            .background(t.error.with_alpha(0.1))
                            .border_radius(RADIUS_MD)
                    })
                    .into_any()
                } else {
                    // Window list
                    window_list(state.clone(), windows, search).into_any()
                }
            })
            .style(|s| s.flex_grow(1.0).width_full()),
        ))
        .style(|s| s.padding(SPACING_LG).width_full().height_full()),
    )
    .style(move |s| {
        let t = theme();
        s.width(450.0)
            .height(400.0)
            .background(t.bg_elevated)
            .border_radius(RADIUS_LG)
            .box_shadow_blur(30.0)
            .box_shadow_color(t.border_subtle.with_alpha(0.5))
    }),))
    .style(move |s| {
        let t = theme();
        // NOTE: Position::Absolute is applied externally in window_rules.rs
        // (matching the wizard pattern from app.rs)
        s.width_full()
            .height_full()
            .background(t.bg_base.with_alpha(0.85))
            .justify_center()
            .items_center()
    })
}

/// Scrollable list of windows
fn window_list(
    state: WindowPickerState,
    windows: RwSignal<Vec<WindowInfo>>,
    search: RwSignal<String>,
) -> impl IntoView {
    dyn_view(move || {
        let search_text = search.get().to_lowercase();
        let filtered: Vec<WindowInfo> = windows
            .get()
            .into_iter()
            .filter(|w| {
                if search_text.is_empty() {
                    true
                } else {
                    w.app_id.to_lowercase().contains(&search_text)
                        || w.title.to_lowercase().contains(&search_text)
                }
            })
            .collect();

        if filtered.is_empty() {
            Container::new(
                Label::new(if search_text.is_empty() {
                    "No windows found"
                } else {
                    "No matching windows"
                })
                .style(move |s| {
                    let t = theme();
                    s.font_size(FONT_SIZE_SM).color(t.text_muted)
                }),
            )
            .style(|s| s.width_full().padding(SPACING_LG).items_center())
            .into_any()
        } else {
            Scroll::new(
                Stack::vertical(
                    filtered
                        .into_iter()
                        .map(|window| window_row(state.clone(), window))
                        .collect::<Vec<_>>(),
                )
                .style(|s| s.width_full().gap(SPACING_XS)),
            )
            .style(move |s| {
                let t = theme();
                s.flex_grow(1.0)
                    .width_full()
                    .background(t.bg_base)
                    .border_radius(RADIUS_MD)
                    .padding(SPACING_XS)
            })
            .into_any()
        }
    })
}

/// Single window row in the picker
fn window_row(state: WindowPickerState, window: WindowInfo) -> impl IntoView {
    let app_id = window.app_id.clone();
    let title = window.title.clone();
    let app_id_display = window.app_id.clone();
    let title_display = window.title.clone();
    let is_floating = window.is_floating;

    Container::new(
        Stack::horizontal((
            // Window info
            Stack::vertical((
                // App ID (bold)
                Label::new(app_id_display).style(move |s| {
                    let t = theme();
                    s.font_size(FONT_SIZE_SM)
                        .font_bold()
                        .color(t.text_primary)
                        .font_family("monospace".to_string())
                }),
                // Title (secondary)
                Label::new(if title_display.len() > 50 {
                    format!("{}...", &title_display[..47])
                } else {
                    title_display
                })
                .style(move |s| {
                    let t = theme();
                    s.font_size(FONT_SIZE_XS).color(t.text_secondary)
                }),
            ))
            .style(|s| s.gap(2.0).flex_grow(1.0)),
            // Floating badge
            {
                if is_floating {
                    Container::new(Label::new("float").style(move |s| {
                        let t = theme();
                        s.font_size(FONT_SIZE_XS).color(t.secondary)
                    }))
                    .style(move |s| {
                        let t = theme();
                        s.padding_horiz(SPACING_XS)
                            .padding_vert(2.0)
                            .border_radius(RADIUS_SM)
                            .background(t.secondary.with_alpha(0.15))
                    })
                    .into_any()
                } else {
                    Empty::new().into_any()
                }
            },
        ))
        .style(|s| s.width_full().items_center().gap(SPACING_SM)),
    )
    .style(move |s| {
        let t = theme();
        s.width_full()
            .padding(SPACING_SM)
            .border_radius(RADIUS_SM)
            .cursor(floem::style::CursorStyle::Pointer)
            .hover(|s| s.background(t.hover_bg))
    })
    .on_click_stop(move |_| {
        state.select(app_id.clone(), title.clone());
    })
}
