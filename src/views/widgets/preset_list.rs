//! Shared preset width/height entry row
//!
//! Used by global Layout extras and the per-output / per-workspace
//! `LayoutOverride` editor so both cycle-preset lists look and behave the same.

use iced::widget::{button, container, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::theme::fonts;

const PRESET_KINDS: [&str; 2] = ["Proportion", "Fixed"];

/// A single preset entry: kind pick-list + numeric value + remove button.
pub fn preset_entry_row<'a, Message: Clone + 'a>(
    kind: &'a str,
    value: &str,
    is_proportion: bool,
    on_kind: impl Fn(&str) -> Message + 'a,
    on_value: impl Fn(String, String) -> Message + 'a,
    on_remove: Message,
) -> Element<'a, Message> {
    let value_owned = value.to_string();
    let kind_owned = kind.to_string();
    let hint = if is_proportion { "0.0 - 1.0" } else { "pixels" };
    let selected = if is_proportion { "Proportion" } else { "Fixed" };

    container(
        row![
            pick_list(PRESET_KINDS.to_vec(), Some(selected), move |k: &str| {
                on_kind(k)
            })
            .width(Length::Fixed(120.0)),
            text_input(hint, &value_owned)
                .on_input(move |v| on_value(kind_owned.clone(), v))
                .padding(6)
                .font(fonts::MONO_FONT)
                .size(12)
                .width(Length::Fill),
            button(text("×").size(14))
                .on_press(on_remove)
                .padding([4, 10]),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(8)
    .style(crate::theme::card_style)
    .into()
}
