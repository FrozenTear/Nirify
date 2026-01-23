//! Config editor message handler

use iced::widget::text_editor;
use iced::Task;

use crate::messages::{ConfigEditorMessage, Message};
use crate::views::config_editor::CONFIG_FILES;

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
                // Reset edit state when loading new file
                self.ui.config_editor_state.has_unsaved_changes = false;
                // Initialize editor content from loaded file
                if let Ok(ref content) = result {
                    self.ui.config_editor_content = text_editor::Content::with_text(content);
                } else {
                    self.ui.config_editor_content = text_editor::Content::new();
                }
                self.ui.config_editor_state.file_content = Some(result);
                Task::none()
            }

            ConfigEditorMessage::ToggleEditMode(enabled) => {
                self.ui.config_editor_state.edit_mode = enabled;
                if enabled {
                    // Initialize editor content from current file content
                    if let Some(Ok(content)) = &self.ui.config_editor_state.file_content {
                        self.ui.config_editor_content = text_editor::Content::with_text(content);
                    }
                } else {
                    // Exiting edit mode - discard changes
                    self.ui.config_editor_state.has_unsaved_changes = false;
                }
                Task::none()
            }

            ConfigEditorMessage::EditorAction(action) => {
                // Check if this action modifies the content
                let is_edit = action.is_edit();
                self.ui.config_editor_content.perform(action);
                if is_edit {
                    self.ui.config_editor_state.has_unsaved_changes = true;
                }
                Task::none()
            }

            ConfigEditorMessage::SaveEdits => {
                if let Some(idx) = self.ui.config_editor_state.selected_file {
                    let paths = self.paths.clone();
                    let filename = CONFIG_FILES[idx].to_string();
                    let content = self.ui.config_editor_content.text();

                    Task::perform(
                        async move {
                            save_config_file(&paths.managed_dir, &filename, &content)
                        },
                        |result| Message::ConfigEditor(ConfigEditorMessage::SaveCompleted(result)),
                    )
                } else {
                    Task::none()
                }
            }

            ConfigEditorMessage::DiscardEdits => {
                // Reset to original file content
                if let Some(Ok(content)) = &self.ui.config_editor_state.file_content {
                    self.ui.config_editor_content = text_editor::Content::with_text(content);
                }
                self.ui.config_editor_state.has_unsaved_changes = false;
                Task::none()
            }

            ConfigEditorMessage::SaveCompleted(result) => {
                match result {
                    Ok(()) => {
                        // Update file_content to match saved content
                        let saved_content = self.ui.config_editor_content.text();
                        self.ui.config_editor_state.file_content = Some(Ok(saved_content));
                        self.ui.config_editor_state.has_unsaved_changes = false;
                    }
                    Err(e) => {
                        // Show error - could update file_content to show error
                        log::error!("Failed to save config file: {}", e);
                    }
                }
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

/// Save content to a config file in the managed directory
fn save_config_file(managed_dir: &std::path::Path, filename: &str, content: &str) -> Result<(), String> {
    let file_path = managed_dir.join(filename);

    // Ensure the directory exists
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))
}
