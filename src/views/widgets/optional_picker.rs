//! Optional boolean picker widget - three-state picker for optional booleans
//!
//! Provides a picker for Option<bool> with three states:
//! - None: "Use Global" / "Default" / "Any"
//! - Some(true): "True" / "Enabled" / "Yes"
//! - Some(false): "False" / "Disabled" / "No"

use iced::widget::{column, container, pick_list, row, text};
use iced::{Alignment, Element, Length};

use crate::theme::muted_text_container;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalBool {
    None,
    True,
    False,
}

impl OptionalBool {
    pub fn all() -> Vec<Self> {
        vec![Self::None, Self::True, Self::False]
    }

    pub fn from_option(opt: Option<bool>) -> Self {
        match opt {
            None => Self::None,
            Some(true) => Self::True,
            Some(false) => Self::False,
        }
    }

    pub fn to_option(self) -> Option<bool> {
        match self {
            Self::None => None,
            Self::True => Some(true),
            Self::False => Some(false),
        }
    }
}

impl std::fmt::Display for OptionalBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Use Global"),
            Self::True => write!(f, "True"),
            Self::False => write!(f, "False"),
        }
    }
}

/// Creates an optional boolean picker row
///
/// Uses "Use Global" / "True" / "False" as labels
///
/// # Example
/// ```rust,ignore
/// optional_bool_picker(
///     "Variable Refresh Rate",
///     "Override global VRR setting",
///     Some(true),
///     Message::SetVrr,
/// )
/// ```
pub fn optional_bool_picker<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    value: Option<bool>,
    on_change: impl Fn(Option<bool>) -> Message + 'a,
) -> Element<'a, Message> {
    let current = OptionalBool::from_option(value);
    let options = OptionalBool::all();

    row![
        // Left side: Label and description
        column![
            text(label).size(16),
            container(text(description).size(12)).style(muted_text_container),
        ]
        .spacing(4)
        .width(Length::Fill),
        // Right side: Picker dropdown
        pick_list(options, Some(current), move |selected| {
            on_change(selected.to_option())
        })
        .width(Length::Fixed(200.0))
        .padding([8, 12])
        .text_size(14),
    ]
    .spacing(20)
    .padding(12)
    .align_y(Alignment::Center)
    .into()
}
