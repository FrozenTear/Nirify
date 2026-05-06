//! Rules screen — window rules and layer rules with sub-tab navigation

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;

use crate::config::models::{LayerRulesSettings, WindowRulesSettings};
use crate::messages::{LayerRulesMessage, Message, RulesFilter, RulesSubTab, WindowRulesMessage};
use crate::theme::{fonts, neon};
use crate::views;

/// Rules screen with Window/Layer sub-tabs
pub fn view<'a>(
    sub_tab: RulesSubTab,
    window_rules: &'a WindowRulesSettings,
    search: &'a str,
    filter: RulesFilter,
    window_rule_sections: &'a HashMap<(u32, String), bool>,
    window_rule_errors: &'a HashMap<(u32, String), String>,
    available_workspaces: &'a [String],
    layer_rules: &'a LayerRulesSettings,
    layer_rule_sections: &'a HashMap<(u32, String), bool>,
    layer_rule_errors: &'a HashMap<(u32, String), String>,
) -> Element<'a, Message> {
    let tab_content: Element<'a, Message> = match sub_tab {
        RulesSubTab::WindowRules => views::window_rules::view(
            window_rules,
            search,
            filter,
            window_rule_sections,
            window_rule_errors,
            available_workspaces,
        ),
        RulesSubTab::LayerRules => views::layer_rules::view(
            layer_rules,
            search,
            filter,
            layer_rule_sections,
            layer_rule_errors,
        ),
    };

    let new_rule_message = match sub_tab {
        RulesSubTab::WindowRules => Message::WindowRules(WindowRulesMessage::AddRule),
        RulesSubTab::LayerRules => Message::LayerRules(LayerRulesMessage::AddRule),
    };

    let content = column![
        // Header row with hero + add button
        row![
            column![
                super::hero_header(
                    "RULE ENGINE",
                    "Window & Layer Rules",
                    "Define per-application behavior with window rules and control layer shell surfaces with layer rules.",
                    neon::PRIMARY,
                ),
            ]
            .width(Length::Fill),
            button(
                row![
                    text("+").size(16),
                    text("New Rule").size(14).font(fonts::UI_FONT_MEDIUM),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .on_press(new_rule_message)
            .padding([10, 20])
            .style(|_: &iced::Theme, status| {
                let bg = match status {
                    iced::widget::button::Status::Hovered => neon::PRIMARY,
                    _ => iced::Color { a: 0.8, ..neon::PRIMARY },
                };
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: neon::ON_SURFACE,
                    border: iced::Border {
                        radius: 12.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }),
        ]
        .align_y(Alignment::End),
        Space::new().height(16),
        super::sub_tab_bar(
            RulesSubTab::all(),
            sub_tab,
            Message::SetRulesSubTab,
        ),
        scrollable(
            container(tab_content)
                .width(Length::Fill)
        )
        .height(Length::Fill),
    ]
    .spacing(12)
    .padding(32)
    .width(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
