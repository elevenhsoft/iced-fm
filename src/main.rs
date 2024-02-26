use iced::{Application, Settings};

mod filepicker;

fn main() -> iced::Result {
    filepicker::FilePicker::run(Settings::default())
}
