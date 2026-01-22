//! Niri Settings entry point

fn main() -> iced::Result {
    env_logger::init();
    niri_settings::app::run()
}
