//! Keybindings settings page

use floem::event::EventListener;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Container, Label, Stack};
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::config::models::{KeybindAction, Keybinding};
use crate::config::SettingsCategory;
use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{
    button_primary_style, icon_button_style, text_input_style, ACCENT, BG_ELEVATED, BG_SURFACE,
    BORDER_SUBTLE, ERROR, FONT_SIZE_SM, RADIUS_MD, RADIUS_SM, SPACING_LG, SPACING_MD, SPACING_SM,
    SPACING_XS, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY, WARNING,
};

/// Categories for grouping keybindings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeybindCategory {
    /// Audio and volume controls
    Audio,
    /// Media playback controls
    Media,
    /// Screen brightness controls
    Brightness,
    /// Window management (close, move, resize)
    Windows,
    /// Workspace navigation and management
    Workspaces,
    /// Monitor/display controls
    Monitors,
    /// Application launchers
    Applications,
    /// System controls (lock, screenshots, etc)
    System,
    /// Other/uncategorized
    Other,
}

/// Sub-categories for fine-grained grouping within categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeybindSubCategory {
    /// Focus/navigate to windows or columns
    Focus,
    /// Move windows or columns
    Move,
    /// Resize, fullscreen, maximize
    Resize,
    /// Column-specific operations
    Columns,
    /// Close, quit operations
    Close,
    /// No sub-category
    None,
}

impl KeybindSubCategory {
    /// Display name for sub-category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Focus => "Focus",
            Self::Move => "Move",
            Self::Resize => "Resize",
            Self::Columns => "Columns",
            Self::Close => "Close",
            Self::None => "",
        }
    }

    /// Sub-categories for Windows in display order
    pub fn windows_order() -> &'static [KeybindSubCategory] {
        &[
            Self::Focus,
            Self::Move,
            Self::Resize,
            Self::Columns,
            Self::Close,
            Self::None,
        ]
    }

    /// Sub-categories for Workspaces in display order
    pub fn workspaces_order() -> &'static [KeybindSubCategory] {
        &[Self::Focus, Self::Move, Self::None]
    }
}

impl KeybindCategory {
    /// Display name for the category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Audio => "Audio",
            Self::Media => "Media",
            Self::Brightness => "Brightness",
            Self::Windows => "Windows",
            Self::Workspaces => "Workspaces",
            Self::Monitors => "Monitors",
            Self::Applications => "Applications",
            Self::System => "System",
            Self::Other => "Other",
        }
    }

    /// Icon/emoji for the category
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Audio => "🔊",
            Self::Media => "🎵",
            Self::Brightness => "☀",
            Self::Windows => "🪟",
            Self::Workspaces => "🗂",
            Self::Monitors => "🖥",
            Self::Applications => "🚀",
            Self::System => "⚙",
            Self::Other => "•",
        }
    }

    /// All categories in display order
    pub fn all() -> &'static [KeybindCategory] {
        &[
            Self::Applications,
            Self::Windows,
            Self::Workspaces,
            Self::Monitors,
            Self::Audio,
            Self::Media,
            Self::Brightness,
            Self::System,
            Self::Other,
        ]
    }
}

/// Categorize a keybinding based on its action
fn categorize_keybinding(bind: &Keybinding) -> KeybindCategory {
    // Check key combo first for hardware keys
    let key = bind.key_combo.to_lowercase();
    if key.contains("audio") {
        if key.contains("pause") || key.contains("play") || key.contains("next") || key.contains("prev") {
            return KeybindCategory::Media;
        }
        return KeybindCategory::Audio;
    }
    if key.contains("brightness") {
        return KeybindCategory::Brightness;
    }

    // Check action
    match &bind.action {
        KeybindAction::Spawn(args) => categorize_spawn(args),
        KeybindAction::NiriAction(action) => categorize_niri_action(action),
        KeybindAction::NiriActionWithArgs(action, _) => categorize_niri_action(action),
    }
}

/// Categorize spawn commands
fn categorize_spawn(args: &[String]) -> KeybindCategory {
    if args.is_empty() {
        return KeybindCategory::Other;
    }

    let first = &args[0];

    // Handle dms ipc call pattern
    if first == "dms" && args.len() >= 4 && args[1] == "ipc" && args[2] == "call" {
        let service = &args[3];
        return match service.as_str() {
            "audio" => KeybindCategory::Audio,
            "brightness" => KeybindCategory::Brightness,
            "processlist" | "settings" | "notifications" | "notepad" => KeybindCategory::System,
            "lock" => KeybindCategory::System,
            "wallpaper" | "dankdash" => KeybindCategory::System,
            _ => KeybindCategory::System,
        };
    }

    // Handle playerctl
    if first == "playerctl" || args.join(" ").contains("playerctl") {
        return KeybindCategory::Media;
    }

    // Handle spawn-sh
    if first == "spawn-sh" || first == "-sh" {
        let cmd = args.join(" ");
        if cmd.contains("playerctl") {
            return KeybindCategory::Media;
        }
        return KeybindCategory::Applications;
    }

    // Default: treat spawn as app launcher
    KeybindCategory::Applications
}

/// Categorize niri built-in actions
fn categorize_niri_action(action: &str) -> KeybindCategory {
    // Window management
    if action.contains("window") || action.contains("column") || action.contains("fullscreen") {
        return KeybindCategory::Windows;
    }

    // Workspaces
    if action.contains("workspace") {
        return KeybindCategory::Workspaces;
    }

    // Monitors
    if action.contains("monitor") || action.contains("output") {
        return KeybindCategory::Monitors;
    }

    // System actions
    if action.contains("screenshot")
        || action.contains("power")
        || action.contains("suspend")
        || action.contains("quit")
        || action.contains("hotkey")
        || action.contains("overview")
    {
        return KeybindCategory::System;
    }

    KeybindCategory::Other
}

/// Get sub-category for a keybinding (for Windows and Workspaces)
fn get_subcategory(bind: &Keybinding, category: KeybindCategory) -> KeybindSubCategory {
    let action_str = match &bind.action {
        KeybindAction::NiriAction(a) => a.as_str(),
        KeybindAction::NiriActionWithArgs(a, _) => a.as_str(),
        KeybindAction::Spawn(_) => return KeybindSubCategory::None,
    };

    match category {
        KeybindCategory::Windows => {
            if action_str.contains("close") || action_str.contains("quit") {
                KeybindSubCategory::Close
            } else if action_str.starts_with("focus-") {
                KeybindSubCategory::Focus
            } else if action_str.starts_with("move-") {
                KeybindSubCategory::Move
            } else if action_str.contains("fullscreen")
                || action_str.contains("maximize")
                || action_str.contains("height")
                || action_str.contains("width")
                || action_str.contains("preset")
            {
                KeybindSubCategory::Resize
            } else if action_str.contains("column")
                || action_str.contains("consume")
                || action_str.contains("expel")
            {
                KeybindSubCategory::Columns
            } else {
                KeybindSubCategory::None
            }
        }
        KeybindCategory::Workspaces => {
            if action_str.starts_with("focus-") {
                KeybindSubCategory::Focus
            } else if action_str.starts_with("move-") {
                KeybindSubCategory::Move
            } else {
                KeybindSubCategory::None
            }
        }
        _ => KeybindSubCategory::None,
    }
}

/// Group keybindings within a category by sub-category
fn group_by_subcategory(
    bindings: &[Keybinding],
    category: KeybindCategory,
) -> BTreeMap<KeybindSubCategory, Vec<Keybinding>> {
    let mut groups: BTreeMap<KeybindSubCategory, Vec<Keybinding>> = BTreeMap::new();

    for bind in bindings {
        let subcat = get_subcategory(bind, category);
        groups.entry(subcat).or_default().push(bind.clone());
    }

    groups
}

/// Group keybindings by category
fn group_by_category(bindings: &[Keybinding]) -> BTreeMap<KeybindCategory, Vec<Keybinding>> {
    let mut groups: BTreeMap<KeybindCategory, Vec<Keybinding>> = BTreeMap::new();

    for bind in bindings {
        let category = categorize_keybinding(bind);
        groups.entry(category).or_default().push(bind.clone());
    }

    groups
}

/// Check if a keybinding matches a search query
fn keybinding_matches_search(bind: &Keybinding, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let query_lower = query.to_lowercase();

    // Match against key combo
    if bind.key_combo.to_lowercase().contains(&query_lower) {
        return true;
    }

    // Match against friendly label
    let friendly = action_to_friendly_label(&bind.action);
    if friendly.to_lowercase().contains(&query_lower) {
        return true;
    }

    // Match against raw action string
    let raw_action = action_to_string(&bind.action);
    if raw_action.to_lowercase().contains(&query_lower) {
        return true;
    }

    // Match against hotkey overlay title if present
    if let Some(title) = &bind.hotkey_overlay_title {
        if title.to_lowercase().contains(&query_lower) {
            return true;
        }
    }

    false
}

/// Filter keybindings by search query
fn filter_keybindings(bindings: &[Keybinding], query: &str) -> Vec<Keybinding> {
    bindings
        .iter()
        .filter(|b| keybinding_matches_search(b, query))
        .cloned()
        .collect()
}

/// Create the keybindings settings page
pub fn keybindings_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();

    // Create signals for keybindings list
    let bindings = RwSignal::new(settings.keybindings.bindings.clone());
    let next_id = RwSignal::new(
        settings
            .keybindings
            .bindings
            .iter()
            .map(|b| b.id)
            .max()
            .unwrap_or(0)
            + 1,
    );

    // Only show error if we have no bindings (complete load failure)
    // If we have bindings, the user's config was loaded successfully
    let initial_error = if settings.keybindings.bindings.is_empty() {
        settings.keybindings.error.clone()
    } else {
        None
    };

    Stack::vertical((
        // Error section if loading completely failed
        if let Some(err) = initial_error {
            section(
                "Error",
                Stack::vertical((Label::derived(move || {
                    format!("Failed to load keybindings: {}", err.clone())
                })
                .style(|s| s.color(WARNING)),)),
            )
            .into_any()
        } else {
            floem::views::Empty::new().into_any()
        },
        section(
            "Keyboard Shortcuts",
            Stack::vertical((
                // List of existing keybindings
                keybinding_list(state.clone(), bindings, next_id),
                // Add button
                add_keybinding_button(state.clone(), bindings, next_id),
            ))
            .style(|s| s.width_full().gap(SPACING_MD)),
        ),
        section(
            "About Keybindings",
            Stack::vertical((Label::derived(|| {
                "Use Mod+Key format for shortcuts (e.g., 'Mod+Space', 'Mod+Shift+Q'). \
                 Actions can be 'spawn <command>' to run programs, or built-in niri actions \
                 like 'close-window', 'toggle-overview', 'focus-workspace browser', etc."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}

/// List of keybinding rows grouped by category with search filtering
fn keybinding_list(
    state: AppState,
    bindings: RwSignal<Vec<Keybinding>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    let search_query = state.search_query;

    floem::views::dyn_container(
        move || (bindings.get(), search_query.get()),
        move |(bind_list, query)| {
            // Filter bindings by search query
            let filtered = filter_keybindings(&bind_list, &query);
            let is_searching = !query.is_empty();

            if bind_list.is_empty() {
                Label::derived(|| "No keybindings configured.".to_string())
                    .style(|s| s.color(TEXT_MUTED).padding_vert(SPACING_MD))
                    .into_any()
            } else if filtered.is_empty() {
                // No results for search
                Stack::vertical((
                    Label::derived(move || format!("No keybindings matching \"{}\"", query.clone()))
                        .style(|s| s.color(TEXT_MUTED).padding_vert(SPACING_MD)),
                ))
                .into_any()
            } else {
                // Group filtered bindings by category
                let groups = group_by_category(&filtered);

                // Build category sections in display order
                let sections: Vec<_> = KeybindCategory::all()
                    .iter()
                    .filter_map(|cat| {
                        groups.get(cat).map(|binds| {
                            category_section(
                                state.clone(),
                                *cat,
                                binds.clone(),
                                bindings,
                                next_id,
                                is_searching,
                            )
                        })
                    })
                    .collect();

                Stack::vertical(sections)
                    .style(|s| s.width_full().gap(SPACING_MD))
                    .into_any()
            }
        },
    )
}

/// A collapsible category section with header and keybinding rows
fn category_section(
    state: AppState,
    category: KeybindCategory,
    category_bindings: Vec<Keybinding>,
    all_bindings: RwSignal<Vec<Keybinding>>,
    next_id: RwSignal<u32>,
    force_expanded: bool,
) -> impl IntoView {
    // Auto-expand when searching, otherwise start collapsed
    let expanded = RwSignal::new(force_expanded);
    let count = category_bindings.len();

    // Check if this category should have sub-categories
    let has_subcategories =
        category == KeybindCategory::Windows || category == KeybindCategory::Workspaces;

    Stack::vertical((
        // Category header
        Container::new(
            Stack::horizontal((
                // Expand/collapse indicator
                Label::derived(move || {
                    if expanded.get() { "▾" } else { "▸" }.to_string()
                })
                .style(|s| s.color(TEXT_MUTED).min_width(16.0)),
                // Category icon and name
                Label::derived(move || format!("{} {}", category.icon(), category.display_name()))
                    .style(|s| s.color(TEXT_PRIMARY).font_bold()),
                // Count badge
                Label::derived(move || format!("{}", count))
                    .style(|s| {
                        s.color(TEXT_TERTIARY)
                            .font_size(FONT_SIZE_SM)
                            .padding_horiz(SPACING_XS)
                            .margin_left(SPACING_SM)
                            .background(BG_SURFACE)
                            .border_radius(RADIUS_SM)
                    }),
            ))
            .style(|s| s.items_center().gap(SPACING_XS)),
        )
        .style(|s| {
            s.width_full()
                .padding_vert(SPACING_XS)
                .padding_horiz(SPACING_SM)
                .cursor(floem::style::CursorStyle::Pointer)
                .hover(|s| s.background(BG_SURFACE.with_alpha(0.5)))
                .border_radius(RADIUS_SM)
        })
        .on_click_stop(move |_| expanded.set(!expanded.get())),
        // Keybinding rows (collapsible)
        floem::views::dyn_container(
            move || expanded.get(),
            move |is_expanded| {
                if is_expanded {
                    if has_subcategories {
                        // Show sub-category groups
                        category_content_with_subcategories(
                            state.clone(),
                            category,
                            category_bindings.clone(),
                            all_bindings,
                            next_id,
                        )
                        .into_any()
                    } else {
                        // Flat list for small categories
                        Stack::vertical(
                            category_bindings
                                .iter()
                                .map(|bind| {
                                    keybinding_row(
                                        state.clone(),
                                        bind.clone(),
                                        all_bindings,
                                        next_id,
                                    )
                                })
                                .collect::<Vec<_>>(),
                        )
                        .style(|s| s.width_full().gap(SPACING_SM).padding_left(SPACING_MD))
                        .into_any()
                    }
                } else {
                    floem::views::Empty::new().into_any()
                }
            },
        ),
    ))
    .style(|s| s.width_full())
}

/// Content for categories with sub-categories (Windows, Workspaces)
fn category_content_with_subcategories(
    state: AppState,
    category: KeybindCategory,
    bindings: Vec<Keybinding>,
    all_bindings: RwSignal<Vec<Keybinding>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    let subgroups = group_by_subcategory(&bindings, category);

    let order = match category {
        KeybindCategory::Windows => KeybindSubCategory::windows_order(),
        KeybindCategory::Workspaces => KeybindSubCategory::workspaces_order(),
        _ => &[KeybindSubCategory::None],
    };

    let sections: Vec<_> = order
        .iter()
        .filter_map(|subcat| {
            subgroups.get(subcat).map(|binds| {
                subcategory_section(
                    state.clone(),
                    *subcat,
                    binds.clone(),
                    all_bindings,
                    next_id,
                )
            })
        })
        .collect();

    Stack::vertical(sections)
        .style(|s| s.width_full().gap(SPACING_SM).padding_left(SPACING_MD))
}

/// A collapsible sub-category section within a category
fn subcategory_section(
    state: AppState,
    subcategory: KeybindSubCategory,
    bindings: Vec<Keybinding>,
    all_bindings: RwSignal<Vec<Keybinding>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    let count = bindings.len();
    let has_label = subcategory != KeybindSubCategory::None;
    let expanded = RwSignal::new(false);

    if has_label {
        // Collapsible sub-category with header
        Stack::vertical((
            // Sub-category header (clickable)
            Container::new(
                Stack::horizontal((
                    // Expand/collapse indicator
                    Label::derived(move || {
                        if expanded.get() { "▾" } else { "▸" }.to_string()
                    })
                    .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM).min_width(12.0)),
                    // Sub-category name and count
                    Label::derived(move || format!("{} ({})", subcategory.display_name(), count))
                        .style(|s| s.color(TEXT_MUTED).font_size(FONT_SIZE_SM)),
                ))
                .style(|s| s.items_center().gap(SPACING_XS)),
            )
            .style(|s| {
                s.padding_vert(SPACING_XS)
                    .padding_horiz(SPACING_XS)
                    .cursor(floem::style::CursorStyle::Pointer)
                    .hover(|s| s.background(BG_SURFACE.with_alpha(0.3)))
                    .border_radius(RADIUS_SM)
            })
            .on_click_stop(move |_| expanded.set(!expanded.get())),
            // Keybinding rows (collapsible)
            floem::views::dyn_container(
                move || expanded.get(),
                move |is_expanded| {
                    if is_expanded {
                        Stack::vertical(
                            bindings
                                .iter()
                                .map(|bind| {
                                    keybinding_row(
                                        state.clone(),
                                        bind.clone(),
                                        all_bindings,
                                        next_id,
                                    )
                                })
                                .collect::<Vec<_>>(),
                        )
                        .style(|s| s.width_full().gap(SPACING_SM).padding_left(SPACING_SM))
                        .into_any()
                    } else {
                        floem::views::Empty::new().into_any()
                    }
                },
            ),
        ))
        .style(|s| s.width_full())
        .into_any()
    } else {
        // No sub-category label - just show rows directly
        Stack::vertical(
            bindings
                .iter()
                .map(|bind| keybinding_row(state.clone(), bind.clone(), all_bindings, next_id))
                .collect::<Vec<_>>(),
        )
        .style(|s| s.width_full().gap(SPACING_SM))
        .into_any()
    }
}

/// Single keybinding row with expandable details
fn keybinding_row(
    state: AppState,
    bind: Keybinding,
    bindings: RwSignal<Vec<Keybinding>>,
    _next_id: RwSignal<u32>,
) -> impl IntoView {
    let bind_id = bind.id;
    let key_combo_signal = RwSignal::new(bind.key_combo.clone());
    let action_signal = RwSignal::new(action_to_string(&bind.action));
    let expanded = RwSignal::new(false);

    // Derive friendly label from the action signal
    let friendly_label = RwSignal::new(action_to_friendly_label(&bind.action));

    // Save helper
    let save = {
        let state = state.clone();
        Rc::new(move || {
            state.update_settings(|s| {
                s.keybindings.bindings = bindings.get();
                s.keybindings.loaded = true;
            });
            state.mark_dirty_and_save(SettingsCategory::Keybindings);
        })
    };

    // Key combo change
    let save_key = save.clone();
    let on_key_change = move || {
        bindings.update(|b_list| {
            if let Some(b) = b_list.iter_mut().find(|b| b.id == bind_id) {
                b.key_combo = key_combo_signal.get();
            }
        });
        save_key();
    };

    // Action change - also update friendly label
    let save_action = save.clone();
    let on_action_change = move || {
        let action_str = action_signal.get();
        let action = string_to_action(&action_str);
        friendly_label.set(action_to_friendly_label(&action));
        bindings.update(|b_list| {
            if let Some(b) = b_list.iter_mut().find(|b| b.id == bind_id) {
                b.action = action;
            }
        });
        save_action();
    };

    // Delete callback
    let state_delete = state.clone();
    let on_delete = move || {
        bindings.update(|b_list| {
            b_list.retain(|b| b.id != bind_id);
        });
        state_delete.update_settings(|s| {
            s.keybindings.bindings = bindings.get();
            s.keybindings.loaded = true;
        });
        state_delete.mark_dirty_and_save(SettingsCategory::Keybindings);
    };

    Stack::vertical((
        // Header row - shows key combo and friendly label
        Stack::horizontal((
            // Key combo input (wider for long key names)
            text_input(key_combo_signal)
                .placeholder("Mod+Key")
                .on_event_stop(EventListener::FocusLost, move |_| {
                    on_key_change();
                })
                .style(|s| {
                    text_input_style(s)
                        .width(180.0)
                        .font_family("monospace".to_string())
                }),
            // Friendly action label (read-only display)
            Label::derived(move || friendly_label.get())
                .style(|s| {
                    s.flex_grow(1.0)
                        .padding_horiz(SPACING_SM)
                        .color(TEXT_SECONDARY)
                }),
            // Expand/collapse button (shows details with raw action)
            Container::new(
                Label::derived(move || if expanded.get() { "⌃" } else { "⌄" }.to_string())
                    .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
            )
            .style(|s| icon_button_style(s).hover(|s| s.color(ACCENT)))
            .on_click_stop(move |_| expanded.set(!expanded.get())),
            // Delete button
            Container::new(
                Label::derived(|| "✕".to_string())
                    .style(|s| s.color(TEXT_MUTED).font_size(FONT_SIZE_SM)),
            )
            .style(|s| {
                icon_button_style(s).hover(|s| s.background(ERROR.with_alpha(0.2)).color(ERROR))
            })
            .on_click_stop(move |_| on_delete()),
        ))
        .style(|s| s.width_full().items_center().gap(SPACING_SM)),
        // Expanded settings (includes raw action input)
        {
            let save = save.clone();
            let on_action_change = on_action_change.clone();

            floem::views::dyn_container(
                move || expanded.get(),
                move |is_expanded| {
                    if is_expanded {
                        keybinding_details_with_action(
                            bind_id,
                            bind.clone(),
                            bindings,
                            action_signal,
                            on_action_change.clone(),
                            save.clone(),
                        )
                        .into_any()
                    } else {
                        floem::views::Empty::new().into_any()
                    }
                },
            )
        },
    ))
    .style(|s| {
        s.width_full()
            .padding(SPACING_SM)
            .background(BG_ELEVATED)
            .border_radius(RADIUS_MD)
            .border(1.0)
            .border_color(BORDER_SUBTLE)
    })
}

/// Expanded keybinding details with raw action input
fn keybinding_details_with_action<F>(
    bind_id: u32,
    bind: Keybinding,
    bindings: RwSignal<Vec<Keybinding>>,
    action_signal: RwSignal<String>,
    on_action_change: F,
    save: Rc<dyn Fn()>,
) -> impl IntoView
where
    F: Fn() + Clone + 'static,
{
    let allow_locked = RwSignal::new(bind.allow_when_locked);
    let repeat = RwSignal::new(bind.repeat);
    let title_signal = RwSignal::new(bind.hotkey_overlay_title.clone().unwrap_or_default());

    let save_locked = save.clone();
    let save_repeat = save.clone();
    let save_title = save.clone();

    Stack::vertical((
        // Raw action input (editable command)
        Stack::horizontal((
            Label::derived(|| "Action".to_string()).style(|s| {
                s.color(TEXT_TERTIARY)
                    .font_size(FONT_SIZE_SM)
                    .min_width(100.0)
            }),
            text_input(action_signal)
                .placeholder("close-window or spawn <app>")
                .on_event_stop(EventListener::FocusLost, move |_| {
                    on_action_change();
                })
                .style(|s| {
                    text_input_style(s)
                        .flex_grow(1.0)
                        .font_family("monospace".to_string())
                        .font_size(FONT_SIZE_SM)
                }),
        ))
        .style(|s| s.width_full().items_center().gap(SPACING_SM)),
        // Hotkey overlay title
        Stack::horizontal((
            Label::derived(|| "Title".to_string()).style(|s| {
                s.color(TEXT_TERTIARY)
                    .font_size(FONT_SIZE_SM)
                    .min_width(100.0)
            }),
            text_input(title_signal)
                .placeholder("(shown in hotkey overlay)")
                .on_event_stop(EventListener::FocusLost, move |_| {
                    let title = title_signal.get();
                    bindings.update(|b_list| {
                        if let Some(b) = b_list.iter_mut().find(|b| b.id == bind_id) {
                            b.hotkey_overlay_title = if title.is_empty() {
                                None
                            } else {
                                Some(title.clone())
                            };
                        }
                    });
                    save_title();
                })
                .style(|s| text_input_style(s).flex_grow(1.0)),
        ))
        .style(|s| s.width_full().items_center().gap(SPACING_SM)),
        // Options row
        Stack::horizontal((
            // Allow when locked toggle
            option_chip("Allow locked", allow_locked, move |val| {
                bindings.update(|b_list| {
                    if let Some(b) = b_list.iter_mut().find(|b| b.id == bind_id) {
                        b.allow_when_locked = val;
                    }
                });
                save_locked();
            }),
            // Repeat toggle
            option_chip("Repeat", repeat, move |val| {
                bindings.update(|b_list| {
                    if let Some(b) = b_list.iter_mut().find(|b| b.id == bind_id) {
                        b.repeat = val;
                    }
                });
                save_repeat();
            }),
        ))
        .style(|s| s.gap(SPACING_SM)),
    ))
    .style(|s| {
        s.width_full()
            .gap(SPACING_SM)
            .padding_top(SPACING_SM)
            .border_top(1.0)
            .border_color(BORDER_SUBTLE)
            .margin_top(SPACING_SM)
    })
}

/// Small toggle chip for options
fn option_chip<F>(label: &'static str, value: RwSignal<bool>, on_change: F) -> impl IntoView
where
    F: Fn(bool) + 'static,
{
    let is_on = move || value.get();

    Container::new(Label::derived(move || label.to_string()).style(move |s| {
        let base = s
            .font_size(FONT_SIZE_SM)
            .padding_horiz(SPACING_SM)
            .padding_vert(SPACING_XS);
        if is_on() {
            base.color(ACCENT)
        } else {
            base.color(TEXT_MUTED)
        }
    }))
    .style(move |s| {
        let base = s.border_radius(RADIUS_SM).border(1.0);
        if is_on() {
            base.background(ACCENT.with_alpha(0.15))
                .border_color(ACCENT)
        } else {
            base.background(BG_SURFACE)
                .border_color(BORDER_SUBTLE)
                .hover(|s| s.border_color(TEXT_MUTED))
        }
    })
    .on_click_stop(move |_| {
        let new_val = !value.get();
        value.set(new_val);
        on_change(new_val);
    })
}

/// Add new keybinding button
fn add_keybinding_button(
    state: AppState,
    bindings: RwSignal<Vec<Keybinding>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    let on_add = move || {
        let id = next_id.get();
        next_id.set(id + 1);

        let new_bind = Keybinding {
            id,
            key_combo: String::new(),
            hotkey_overlay_title: None,
            allow_when_locked: false,
            cooldown_ms: None,
            repeat: false,
            action: KeybindAction::NiriAction(String::new()),
        };

        // Get current bindings first, then add the new one
        let mut current_bindings = bindings.get();
        log::debug!(
            "Adding keybinding. Current count: {}, new id: {}",
            current_bindings.len(),
            id
        );
        current_bindings.push(new_bind);

        // Update the signal - but DON'T save yet!
        // The new keybinding has empty key_combo and action, which is invalid.
        // We only update the UI state so the user can edit it.
        // Saving happens when the user fills in valid values and edits are made.
        bindings.set(current_bindings.clone());

        // Update internal state but don't save to disk
        state.update_settings(|s| {
            s.keybindings.bindings = current_bindings;
            s.keybindings.loaded = true;
            s.keybindings.error = None;
        });
        // NOTE: We intentionally don't call mark_dirty_and_save here.
        // The empty keybinding would crash niri. User must fill in values first.
    };

    Container::new(
        Stack::horizontal((
            Label::derived(|| "+".to_string()).style(|s| s.font_size(FONT_SIZE_SM).font_bold()),
            Label::derived(|| "Add Keybinding".to_string()).style(|s| s.font_size(FONT_SIZE_SM)),
        ))
        .style(|s| s.items_center().gap(SPACING_XS)),
    )
    .style(|s| button_primary_style(s).margin_top(SPACING_SM))
    .on_click_stop(move |_| on_add())
}

/// Convert action to editable string
fn action_to_string(action: &KeybindAction) -> String {
    match action {
        KeybindAction::Spawn(args) => {
            if args.is_empty() {
                "spawn".to_string()
            } else {
                format!("spawn {}", shell_words::join(args))
            }
        }
        KeybindAction::NiriAction(action) => action.clone(),
        KeybindAction::NiriActionWithArgs(action, args) => {
            format!("{} {}", action, args.join(" "))
        }
    }
}

/// Convert action to a friendly, human-readable label
fn action_to_friendly_label(action: &KeybindAction) -> String {
    match action {
        KeybindAction::Spawn(args) => parse_spawn_friendly(args),
        KeybindAction::NiriAction(action) => parse_niri_action_friendly(action, &[]),
        KeybindAction::NiriActionWithArgs(action, args) => {
            parse_niri_action_friendly(action, args)
        }
    }
}

/// Parse spawn commands into friendly labels
fn parse_spawn_friendly(args: &[String]) -> String {
    if args.is_empty() {
        return "Run Command".to_string();
    }

    let first = &args[0];

    // Handle dms ipc call pattern (common in this config)
    if first == "dms" && args.len() >= 4 && args[1] == "ipc" && args[2] == "call" {
        return parse_dms_ipc_friendly(&args[3..]);
    }

    // Handle spawn-sh for shell commands
    if first == "spawn-sh" || first == "-sh" {
        if args.len() >= 2 {
            return parse_shell_command_friendly(&args[1..]);
        }
        return "Run Shell Command".to_string();
    }

    // Handle playerctl
    if first == "playerctl" || (args.len() >= 2 && args[0].contains("playerctl")) {
        return parse_playerctl_friendly(args);
    }

    // Generic spawn - extract app name and prettify
    let app_name = extract_app_name(first);
    format!("Launch {}", titlecase(&app_name))
}

/// Parse dms ipc call commands
fn parse_dms_ipc_friendly(args: &[String]) -> String {
    if args.is_empty() {
        return "System Command".to_string();
    }

    let service = &args[0];
    let remaining = &args[1..];

    match service.as_str() {
        "audio" => parse_audio_command(remaining),
        "brightness" => parse_brightness_command(remaining),
        "processlist" => {
            if remaining.first().map(|s| s.as_str()) == Some("toggle") {
                "Toggle Process List".to_string()
            } else {
                "Process List".to_string()
            }
        }
        "settings" => {
            if remaining.first().map(|s| s.as_str()) == Some("toggle") {
                "Toggle Settings".to_string()
            } else {
                "Open Settings".to_string()
            }
        }
        "notifications" => {
            if remaining.first().map(|s| s.as_str()) == Some("toggle") {
                "Toggle Notifications".to_string()
            } else {
                "Notifications".to_string()
            }
        }
        "notepad" => {
            if remaining.first().map(|s| s.as_str()) == Some("toggle") {
                "Toggle Notepad".to_string()
            } else {
                "Open Notepad".to_string()
            }
        }
        "lock" => "Lock Screen".to_string(),
        "wallpaper" => "Change Wallpaper".to_string(),
        "dankdash" => {
            if remaining.first().map(|s| s.as_str()) == Some("wallpaper") {
                "Change Wallpaper".to_string()
            } else {
                format!("Dankdash {}", remaining.join(" "))
            }
        }
        _ => {
            // Generic service call
            if remaining.first().map(|s| s.as_str()) == Some("toggle") {
                format!("Toggle {}", titlecase(service))
            } else if !remaining.is_empty() {
                format!("{} {}", titlecase(service), remaining.join(" "))
            } else {
                titlecase(service)
            }
        }
    }
}

/// Parse audio commands
fn parse_audio_command(args: &[String]) -> String {
    if args.is_empty() {
        return "Audio".to_string();
    }

    match args[0].as_str() {
        "increment" => "Volume Up".to_string(),
        "decrement" => "Volume Down".to_string(),
        "mute" => "Mute Audio".to_string(),
        "micmute" => "Mute Microphone".to_string(),
        "unmute" => "Unmute Audio".to_string(),
        _ => format!("Audio {}", titlecase(&args[0])),
    }
}

/// Parse brightness commands
fn parse_brightness_command(args: &[String]) -> String {
    if args.is_empty() {
        return "Brightness".to_string();
    }

    match args[0].as_str() {
        "increment" => "Brightness Up".to_string(),
        "decrement" => "Brightness Down".to_string(),
        _ => format!("Brightness {}", titlecase(&args[0])),
    }
}

/// Parse shell commands (spawn-sh)
fn parse_shell_command_friendly(args: &[String]) -> String {
    let cmd = args.join(" ");

    if cmd.contains("playerctl") {
        if cmd.contains("play-pause") {
            return "Play/Pause Media".to_string();
        } else if cmd.contains("next") {
            return "Next Track".to_string();
        } else if cmd.contains("previous") {
            return "Previous Track".to_string();
        }
        return "Media Control".to_string();
    }

    // Generic shell command
    "Run Shell Command".to_string()
}

/// Parse playerctl commands
fn parse_playerctl_friendly(args: &[String]) -> String {
    let cmd = args.join(" ");

    if cmd.contains("play-pause") {
        "Play/Pause Media".to_string()
    } else if cmd.contains("next") {
        "Next Track".to_string()
    } else if cmd.contains("previous") || cmd.contains("prev") {
        "Previous Track".to_string()
    } else if cmd.contains("stop") {
        "Stop Media".to_string()
    } else {
        "Media Control".to_string()
    }
}

/// Parse niri built-in actions into friendly labels
fn parse_niri_action_friendly(action: &str, args: &[String]) -> String {
    if action.is_empty() {
        return "No Action".to_string();
    }

    // Handle common niri actions
    let friendly = match action {
        // Window management
        "close-window" => "Close Window".to_string(),
        "quit" => "Quit Application".to_string(),
        "fullscreen-window" => "Toggle Fullscreen".to_string(),
        "maximize-column" => "Maximize Column".to_string(),
        "center-column" => "Center Column".to_string(),
        "focus-window-up" => "Focus Window Above".to_string(),
        "focus-window-down" => "Focus Window Below".to_string(),
        "focus-column-left" => "Focus Column Left".to_string(),
        "focus-column-right" => "Focus Column Right".to_string(),
        "move-window-up" => "Move Window Up".to_string(),
        "move-window-down" => "Move Window Down".to_string(),
        "move-column-left" => "Move Column Left".to_string(),
        "move-column-right" => "Move Column Right".to_string(),
        "consume-window-into-column" => "Add Window to Column".to_string(),
        "expel-window-from-column" => "Remove Window from Column".to_string(),

        // Workspaces
        "focus-workspace" => {
            if let Some(ws) = args.first() {
                format!("Go to Workspace {}", titlecase(ws))
            } else {
                "Focus Workspace".to_string()
            }
        }
        "focus-workspace-up" => "Previous Workspace".to_string(),
        "focus-workspace-down" => "Next Workspace".to_string(),
        "move-window-to-workspace" => {
            if let Some(ws) = args.first() {
                format!("Move to Workspace {}", titlecase(ws))
            } else {
                "Move to Workspace".to_string()
            }
        }
        "move-column-to-workspace" => {
            if let Some(ws) = args.first() {
                format!("Move Column to Workspace {}", titlecase(ws))
            } else {
                "Move Column to Workspace".to_string()
            }
        }

        // Monitors
        "focus-monitor-left" => "Focus Monitor Left".to_string(),
        "focus-monitor-right" => "Focus Monitor Right".to_string(),
        "move-window-to-monitor-left" => "Move to Left Monitor".to_string(),
        "move-window-to-monitor-right" => "Move to Right Monitor".to_string(),

        // Overview and UI
        "toggle-overview" => "Toggle Overview".to_string(),
        "show-hotkey-overlay" => "Show Keyboard Shortcuts".to_string(),

        // Layout
        "switch-preset-column-width" => "Cycle Column Width".to_string(),
        "reset-window-height" => "Reset Window Height".to_string(),
        "set-window-height" => {
            if let Some(h) = args.first() {
                format!("Set Window Height {}", h)
            } else {
                "Set Window Height".to_string()
            }
        }

        // Screenshots
        "screenshot" => "Take Screenshot".to_string(),
        "screenshot-screen" => "Screenshot Screen".to_string(),
        "screenshot-window" => "Screenshot Window".to_string(),

        // Session
        "power-off-monitors" => "Turn Off Monitors".to_string(),
        "suspend" => "Suspend".to_string(),

        // If not recognized, prettify the action name
        _ => prettify_action_name(action),
    };

    // Append args if we have them and didn't already use them
    if !args.is_empty() && !friendly.contains(&args[0]) {
        format!("{} ({})", friendly, args.join(" "))
    } else {
        friendly
    }
}

/// Convert kebab-case action name to Title Case
fn prettify_action_name(action: &str) -> String {
    action
        .split('-')
        .map(|word| titlecase(word))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract app name from a path or command
fn extract_app_name(cmd: &str) -> String {
    // Get the last component of a path
    let name = cmd.rsplit('/').next().unwrap_or(cmd);
    // Remove common extensions
    name.strip_suffix(".sh")
        .or_else(|| name.strip_suffix(".py"))
        .or_else(|| name.strip_suffix(".AppImage"))
        .unwrap_or(name)
        .to_string()
}

/// Convert a string to Title Case
fn titlecase(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Parse string back to action
fn string_to_action(s: &str) -> KeybindAction {
    let trimmed = s.trim();
    if trimmed.starts_with("spawn ") {
        let cmd = trimmed.strip_prefix("spawn ").unwrap_or("");
        let args = shell_words::split(cmd).unwrap_or_else(|_| vec![cmd.to_string()]);
        KeybindAction::Spawn(args)
    } else if trimmed == "spawn" {
        KeybindAction::Spawn(vec![])
    } else if trimmed.contains(' ') {
        // Action with args
        let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
        if parts.len() == 2 {
            let args: Vec<String> = parts[1].split_whitespace().map(String::from).collect();
            KeybindAction::NiriActionWithArgs(parts[0].to_string(), args)
        } else {
            KeybindAction::NiriAction(trimmed.to_string())
        }
    } else {
        KeybindAction::NiriAction(trimmed.to_string())
    }
}
