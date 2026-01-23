//! Niri Settings entry point

fn main() -> iced::Result {
    env_logger::init();
    nirify::app::run()
}
