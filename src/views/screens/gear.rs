//! Gear menu screen — app-level settings (tools, preferences, config editor, backups)

use iced::widget::{column, container, text_editor};
use iced::{Element, Length};

use crate::config::models::PreferencesSettings;
use crate::messages::{GearSubTab, Message};
use crate::theme::neon;
use crate::views;
use crate::views::backups::BackupsState;
use crate::views::config_editor::ConfigEditorState;
use crate::views::status_bar::NiriStatus;
use crate::views::tools::ToolsState;

/// Gear screen with Tools/Preferences/ConfigEditor/Backups sub-tabs
pub fn view<'a>(
    sub_tab: GearSubTab,
    tools_state: &'a ToolsState,
    niri_status: NiriStatus,
    preferences: &'a PreferencesSettings,
    show_search_bar: bool,
    config_editor_state: &'a ConfigEditorState,
    config_editor_content: &'a text_editor::Content,
    backups_state: &'a BackupsState,
) -> Element<'a, Message> {
    let niri_connected = matches!(niri_status, NiriStatus::Connected);

    let tab_content: Element<'a, Message> = match sub_tab {
        GearSubTab::Tools => views::tools::view(tools_state, niri_connected),
        GearSubTab::Preferences => views::preferences::view(
            preferences.float_settings_app,
            show_search_bar,
            &preferences.search_hotkey,
        ),
        GearSubTab::ConfigEditor => {
            views::config_editor::view(config_editor_state, config_editor_content)
        }
        GearSubTab::Backups => views::backups::view(backups_state),
    };

    let content = column![
        super::hero_header(
            "CONTROL CENTER",
            "Settings",
            "App tools, preferences, configuration editor, and backup management.",
            neon::PRIMARY,
        ),
        super::sub_tab_bar(GearSubTab::all(), sub_tab, Message::SetGearSubTab,),
        container(tab_content)
            .width(Length::Fill)
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
