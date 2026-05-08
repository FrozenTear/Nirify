//! Debug settings view — neon modal style

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use super::widgets::{info_text, picker_row, toggle_row};
use crate::config::models::{DebugSettings, PreviewRenderMode};
use crate::messages::{DebugMessage, Message};
use crate::theme::{fonts, neon};

/// Creates the debug settings view (with scrollable wrapper)
pub fn view(settings: &DebugSettings) -> Element<'_, Message> {
    let content = column![view_section(settings),]
        .spacing(0)
        .width(Length::Fill);

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

/// Inner content without scrollable wrapper
pub fn view_section(settings: &DebugSettings) -> Element<'_, Message> {
    let render_drm = settings.render_drm_device.clone().unwrap_or_default();
    let ignore_drm = settings.ignore_drm_devices.clone();

    // Build ignored DRM devices list
    let mut drm_list = column![].spacing(4);
    if ignore_drm.is_empty() {
        drm_list = drm_list.push(
            text("None configured")
                .size(11)
                .color(neon::OUTLINE_VARIANT),
        );
    } else {
        for (idx, device) in ignore_drm.iter().enumerate() {
            drm_list = drm_list.push(
                container(
                    row![
                        text(device.clone())
                            .size(12)
                            .font(fonts::MONO_FONT)
                            .color(neon::ON_SURFACE)
                            .width(Length::Fill),
                        button(text("Remove").size(10).color(neon::ERROR))
                            .on_press(Message::Debug(DebugMessage::RemoveIgnoreDrmDevice(idx)))
                            .padding([2, 8])
                            .style(delete_button_style),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .padding(8)
                .style(crate::theme::card_style),
            );
        }
    }

    let content = column![
        // ── EXPERT MODE (full width) ──
        modal_section("\u{26A0}", "EXPERT MODE", neon::ERROR),
        info_text("Advanced settings for debugging. Most users won't need these."),
        Space::new().height(4),
        container(
            column![toggle_row(
                "Expert Mode",
                "Show potentially dangerous advanced settings across the app",
                settings.expert_mode,
                |v| Message::Debug(DebugMessage::SetExpertMode(v)),
            ),]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
        Space::new().height(16),
        // ── 2-COLUMN: RENDERING | PERFORMANCE ──
        row![
            // Left: Rendering + Device
            column![
                modal_section("\u{25A3}", "RENDERING", neon::PRIMARY),
                Space::new().height(4),
                picker_row(
                    "Preview Render",
                    "Render monitors as if recording",
                    PreviewRenderMode::all(),
                    Some(settings.preview_render),
                    |mode| Message::Debug(DebugMessage::SetPreviewRender(mode)),
                ),
                container(
                    column![
                        toggle_row(
                            "Enable Overlay Planes",
                            "Direct scanout into overlay planes",
                            settings.enable_overlay_planes,
                            |v| Message::Debug(DebugMessage::SetEnableOverlayPlanes(v)),
                        ),
                        toggle_row(
                            "Disable Cursor Plane",
                            "May improve compatibility",
                            settings.disable_cursor_plane,
                            |v| Message::Debug(DebugMessage::SetDisableCursorPlane(v)),
                        ),
                        toggle_row(
                            "Disable Direct Scanout",
                            "Disable scanout to primary and overlay planes",
                            settings.disable_direct_scanout,
                            |v| Message::Debug(DebugMessage::SetDisableDirectScanout(v)),
                        ),
                        toggle_row(
                            "Restrict Scanout Format",
                            "Only scanout when buffer format matches",
                            settings.restrict_primary_scanout_to_matching_format,
                            |v| Message::Debug(
                                DebugMessage::SetRestrictPrimaryScanoutToMatchingFormat(v)
                            ),
                        ),
                    ]
                    .spacing(0),
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(12),
                modal_section("\u{2699}", "DEVICE", neon::TERTIARY),
                Space::new().height(4),
                styled_text_input(
                    "RENDER DRM DEVICE",
                    "e.g., /dev/dri/renderD128",
                    &render_drm,
                    |s| {
                        Message::Debug(DebugMessage::SetRenderDrmDevice(if s.is_empty() {
                            None
                        } else {
                            Some(s)
                        }))
                    },
                ),
                container(
                    column![
                        text("IGNORED DRM DEVICES")
                            .size(10)
                            .font(fonts::UI_FONT_SEMIBOLD)
                            .color(neon::OUTLINE_VARIANT),
                        Space::new().height(4),
                        drm_list,
                    ]
                    .spacing(2),
                )
                .padding(12)
                .style(crate::theme::card_style),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            // Right: Performance + Hardware + Window Behavior + Screencasting
            column![
                modal_section("\u{26A1}", "PERFORMANCE", neon::SECONDARY),
                Space::new().height(4),
                container(
                    column![
                        toggle_row(
                            "Wait for Frame Completion",
                            "Wait until every frame is rendered before queueing",
                            settings.wait_for_frame_completion_before_queueing,
                            |v| Message::Debug(
                                DebugMessage::SetWaitForFrameCompletionBeforeQueueing(v)
                            ),
                        ),
                        toggle_row(
                            "Disable Resize Throttling",
                            "Send resize events as quickly as possible",
                            settings.disable_resize_throttling,
                            |v| Message::Debug(DebugMessage::SetDisableResizeThrottling(v)),
                        ),
                        toggle_row(
                            "Disable Transactions",
                            "Disable synchronized window resizing",
                            settings.disable_transactions,
                            |v| Message::Debug(DebugMessage::SetDisableTransactions(v)),
                        ),
                        toggle_row(
                            "Emulate Zero Presentation Time",
                            "Simulate unknown presentation time",
                            settings.emulate_zero_presentation_time,
                            |v| Message::Debug(DebugMessage::SetEmulateZeroPresentationTime(v)),
                        ),
                        toggle_row(
                            "Skip Cursor-Only Updates (VRR)",
                            "Skip redraws from cursor movement during VRR",
                            settings.skip_cursor_only_updates_during_vrr,
                            |v| Message::Debug(DebugMessage::SetSkipCursorOnlyUpdatesDuringVrr(v)),
                        ),
                    ]
                    .spacing(0),
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(12),
                modal_section("\u{25CE}", "HARDWARE", neon::TERTIARY),
                Space::new().height(4),
                container(
                    column![
                        toggle_row(
                            "D-Bus in Non-Session",
                            "Create D-Bus interfaces in non-session instances",
                            settings.dbus_interfaces_in_non_session_instances,
                            |v| Message::Debug(
                                DebugMessage::SetDbusInterfacesInNonSessionInstances(v)
                            ),
                        ),
                        toggle_row(
                            "Keep Panel On (Lid Closed)",
                            "Keep laptop panel on when lid is closed",
                            settings.keep_laptop_panel_on_when_lid_is_closed,
                            |v| Message::Debug(DebugMessage::SetKeepLaptopPanelOnWhenLidIsClosed(
                                v
                            )),
                        ),
                        toggle_row(
                            "Disable Monitor Names",
                            "Disable EDID monitor name reading",
                            settings.disable_monitor_names,
                            |v| Message::Debug(DebugMessage::SetDisableMonitorNames(v)),
                        ),
                        toggle_row(
                            "Force Disable Connectors on Resume",
                            "Blank all outputs on TTY switch/resume",
                            settings.force_disable_connectors_on_resume,
                            |v| Message::Debug(DebugMessage::SetForceDisableConnectorsOnResume(v)),
                        ),
                    ]
                    .spacing(0),
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(12),
                modal_section("\u{25A6}", "WINDOW BEHAVIOR", neon::PRIMARY),
                Space::new().height(4),
                container(
                    column![
                        toggle_row(
                            "Strict New Window Focus",
                            "Only focus windows with valid xdg-activation token",
                            settings.strict_new_window_focus_policy,
                            |v| Message::Debug(DebugMessage::SetStrictNewWindowFocusPolicy(v)),
                        ),
                        toggle_row(
                            "Honor XDG Activation (Invalid Serial)",
                            "Allow focus via invalid xdg-activation serial",
                            settings.honor_xdg_activation_with_invalid_serial,
                            |v| Message::Debug(
                                DebugMessage::SetHonorXdgActivationWithInvalidSerial(v)
                            ),
                        ),
                        toggle_row(
                            "Deactivate Unfocused Windows",
                            "Drop activated state for unfocused windows",
                            settings.deactivate_unfocused_windows,
                            |v| Message::Debug(DebugMessage::SetDeactivateUnfocusedWindows(v)),
                        ),
                    ]
                    .spacing(0),
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(12),
                modal_section("\u{25B6}", "SCREENCASTING", neon::SECONDARY),
                Space::new().height(4),
                container(
                    column![toggle_row(
                        "Force PipeWire Invalid Modifier",
                        "Force invalid DRM modifier for PipeWire",
                        settings.force_pipewire_invalid_modifier,
                        |v| Message::Debug(DebugMessage::SetForcePipewireInvalidModifier(v)),
                    ),]
                    .spacing(0),
                )
                .padding(8)
                .style(crate::theme::card_style),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
    ]
    .spacing(0);

    content.into()
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn modal_section<'a>(icon: &'a str, label: &'a str, accent: iced::Color) -> Element<'a, Message> {
    row![
        text(icon).size(14).color(accent),
        Space::new().width(6),
        text(label)
            .size(11)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(accent),
        Space::new().width(12),
        container(Space::new().width(Length::Fill).height(1))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color { a: 0.25, ..accent })),
                ..Default::default()
            }),
    ]
    .spacing(0)
    .align_y(Alignment::Center)
    .padding([14, 0])
    .into()
}

fn styled_text_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let v = value.to_string();
    container(
        column![
            text(label)
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(neon::OUTLINE_VARIANT),
            text_input(placeholder, &v)
                .on_input(on_change)
                .padding(10)
                .size(13),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}

fn delete_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let danger = theme.palette().danger;
    let bg = match status {
        button::Status::Hovered => iced::Color { a: 0.3, ..danger },
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: danger,
        ..Default::default()
    }
}
