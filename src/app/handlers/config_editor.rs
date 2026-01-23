//! Config editor message handler

use crate::messages::{ConfigEditorMessage, Message};
use crate::views::config_editor::CONFIG_FILES;
use iced::Task;

impl super::super::App {
    /// Updates config editor state
    pub(in crate::app) fn update_config_editor(&mut self, msg: ConfigEditorMessage) -> Task<Message> {
        match msg {
            ConfigEditorMessage::SelectFile(idx) => {
                self.ui.config_editor_state.selected_file = Some(idx);
                self.ui.config_editor_state.loading = true;
                self.ui.config_editor_state.file_content = None;

                // Load the file asynchronously
                let paths = self.paths.clone();
                let filename = CONFIG_FILES[idx].to_string();

                Task::perform(
                    async move {
                        load_config_file(&paths.managed_dir, &filename)
                    },
                    |result| Message::ConfigEditor(ConfigEditorMessage::FileLoaded(result)),
                )
            }

            ConfigEditorMessage::Refresh => {
                if let Some(idx) = self.ui.config_editor_state.selected_file {
                    self.ui.config_editor_state.loading = true;

                    let paths = self.paths.clone();
                    let filename = CONFIG_FILES[idx].to_string();

                    Task::perform(
                        async move {
                            load_config_file(&paths.managed_dir, &filename)
                        },
                        |result| Message::ConfigEditor(ConfigEditorMessage::FileLoaded(result)),
                    )
                } else {
                    Task::none()
                }
            }

            ConfigEditorMessage::FileLoaded(result) => {
                self.ui.config_editor_state.loading = false;
                self.ui.config_editor_state.file_content = Some(result);
                Task::none()
            }
        }
    }
}

/// Load a config file from the managed directory
fn load_config_file(managed_dir: &std::path::Path, filename: &str) -> Result<String, String> {
    let file_path = managed_dir.join(filename);

    if !file_path.exists() {
        return Err(format!(
            "File does not exist yet. It will be created when you save settings that use it.\n\nExpected path: {}",
            file_path.display()
        ));
    }

    std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}
