//! File path picker widget with browse button
//!
//! Allows users to select files or directories using a native file dialog.

use iced::widget::{button, column, row, text, text_input};
use iced::{Alignment, Element, Length, Task};

/// Messages for file path picker interactions
#[derive(Debug, Clone)]
pub enum FilePathMessage {
    TextChanged(String),
    Browse,
    PathSelected(Option<String>),
}

/// Type of file picker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePickerType {
    File,
    Directory,
    Save,
}

/// Creates a file path picker widget with text input and browse button
pub fn file_path_picker<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    path: &'a str,  // Add 'a lifetime to avoid memory leak
    _picker_type: FilePickerType,
    on_change: impl Fn(FilePathMessage) -> Message + 'a + Copy,
) -> Element<'a, Message> {
    let content = column![
        text(label).size(16),
        text(description).size(12).color([0.7, 0.7, 0.7]),
        row![
            text_input("", path)
                .on_input(move |input| on_change(FilePathMessage::TextChanged(input)))
                .padding(8)
                .width(Length::Fill),
            button(text("Browse"))
                .on_press(on_change(FilePathMessage::Browse))
                .padding([8, 16]),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(4);

    content.into()
}

/// Helper function to open file dialog (use in async Task)
pub fn open_file_dialog(picker_type: FilePickerType) -> Option<String> {
    let result = match picker_type {
        FilePickerType::File => {
            rfd::FileDialog::new()
                .pick_file()
                .map(|path| path.to_string_lossy().to_string())
        }
        FilePickerType::Directory => {
            rfd::FileDialog::new()
                .pick_folder()
                .map(|path| path.to_string_lossy().to_string())
        }
        FilePickerType::Save => {
            rfd::FileDialog::new()
                .save_file()
                .map(|path| path.to_string_lossy().to_string())
        }
    };
    result
}

/// Creates a Task that opens a file dialog
pub fn browse_task<Message: 'static + Send>(
    picker_type: FilePickerType,
    on_result: impl Fn(Option<String>) -> Message + 'static + Send,
) -> Task<Message> {
    Task::perform(
        async move {
            // Run file dialog in blocking context
            open_file_dialog(picker_type)
        },
        on_result,
    )
}
