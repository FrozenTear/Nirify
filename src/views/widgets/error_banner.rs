//! Persistent top-of-window banner components.
//!
//! Renders two kinds of top-of-window banners:
//! - a non-dismissible "setup incomplete" banner when niri is not yet reading
//!   the managed settings (include line missing), and
//! - the dismissible error banner backed by [`ErrorBanner`] state.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Border, Element, Length};

use crate::app::ui_state::{ErrorBanner, ErrorBannerKind};
use crate::messages::{DialogState, Message, WizardStep};

/// Danger-styled container derived from the theme palette (no raw rgb).
fn danger_style(theme: &iced::Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.danger.weak.color.into()),
        border: Border {
            color: palette.danger.base.color,
            width: 1.0,
            radius: 8.0.into(),
        },
        text_color: Some(palette.danger.weak.text),
        ..Default::default()
    }
}

/// Warning-styled container (amber-ish) derived from the theme palette.
fn warning_style(theme: &iced::Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.secondary.weak.color.into()),
        border: Border {
            color: palette.secondary.strong.color,
            width: 1.0,
            radius: 8.0.into(),
        },
        text_color: Some(palette.secondary.weak.text),
        ..Default::default()
    }
}

/// Renders the persistent banners. Returns `None` if there is nothing to show.
pub fn error_banners<'a>(
    error: Option<&'a ErrorBanner>,
    include_line_present: Option<bool>,
) -> Option<Element<'a, Message>> {
    let mut col = column![].spacing(6);
    let mut any = false;

    // Setup-incomplete banner (non-dismissible): only when the include line is
    // known to be absent.
    if include_line_present == Some(false) {
        any = true;
        col = col.push(include_missing_banner());
    }

    if let Some(banner) = error {
        any = true;
        col = col.push(error_banner_view(banner));
    }

    if any {
        Some(col.width(Length::Fill).into())
    } else {
        None
    }
}

fn include_missing_banner<'a>() -> Element<'a, Message> {
    let body = column![
        text("Setup incomplete — niri is not reading these settings").size(14),
        text("Add the include line to your niri config or run setup.").size(13),
    ]
    .spacing(2)
    .width(Length::Fill);

    let run_setup = button(text("Run setup").size(13)).on_press(Message::ShowDialog(
        DialogState::FirstRunWizard {
            step: WizardStep::Welcome,
        },
    ));

    container(row![body, run_setup].spacing(12).align_y(Alignment::Center))
        .style(warning_style)
        .width(Length::Fill)
        .padding(10)
        .into()
}

fn error_banner_view<'a>(banner: &'a ErrorBanner) -> Element<'a, Message> {
    let mut details_col = column![text(banner.title.clone()).size(14)].spacing(2);
    for d in banner.details.iter().take(5) {
        details_col = details_col.push(text(d.clone()).size(13));
    }
    if banner.details.len() > 5 {
        details_col =
            details_col.push(text(format!("…and {} more", banner.details.len() - 5)).size(13));
    }

    let mut buttons = row![].spacing(8);
    if banner.kind == ErrorBannerKind::LoadFailed {
        buttons = buttons.push(
            button(text("Overwrite with current values").size(13))
                .on_press(Message::OverwriteFailedCategories),
        );
    }
    buttons = buttons.push(button(text("Dismiss").size(13)).on_press(Message::DismissErrorBanner));

    container(
        row![
            details_col.width(Length::Fill),
            Space::new().width(Length::Fixed(8.0)),
            buttons
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .style(danger_style)
    .width(Length::Fill)
    .padding(10)
    .into()
}
