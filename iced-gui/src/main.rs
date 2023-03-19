use iced::{Application, Settings};
pub mod controller;
pub mod gui;

#[tokio::main]
async fn main() -> iced::Result {
    gui::app::App::run(Settings::default())
}
