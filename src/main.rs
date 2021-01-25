use iced::{Application, Settings};

mod gui;

pub fn main() -> iced::Result {
    gui::GUI::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}
