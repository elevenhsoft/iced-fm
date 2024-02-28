use std::path::Path;
use std::{env, fs};

use iced::widget::{button, column, row, scrollable, text, text_input, Container};
use iced::{executor, Length};
use iced::{Application, Command, Element, Theme};

#[derive(Debug, Clone)]
pub enum Content {
    File(ContentData),
    Directory(ContentData),
    Corrupt,
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::File(file_data) | Content::Directory(file_data) => {
                write!(f, "{}", file_data.name)
            }
            Content::Corrupt => write!(f, "File or directory corrupted."),
        }
    }
}

impl Content {
    fn size(&self) -> u64 {
        match self {
            Content::File(files) => files.size,
            Content::Directory(dirs) => dirs.size,
            Content::Corrupt => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContentData {
    pub is_parent: bool,
    path: String,
    name: String,
    size: u64,
}

impl Default for ContentData {
    fn default() -> Self {
        Self {
            is_parent: false,
            path: "no path".to_string(),
            name: "unknown".to_string(),
            size: 0,
        }
    }
}

impl ContentData {
    fn new(path: String, parent: bool) -> ContentData {
        let name = if parent {
            "..".to_string()
        } else {
            Path::new(&path)
                .file_name()
                .unwrap()
                .to_str()
                .expect("file name")
                .to_string()
        };

        let size = match Path::new(&path).metadata() {
            Ok(meta) => meta.len() / 1024,
            Err(_) => 0,
        };

        ContentData {
            is_parent: parent,
            path,
            name,
            size,
        }
    }
}

pub struct FilePicker {
    path: String,
    content: Vec<Content>,
}

#[derive(Debug, Clone)]
pub enum Message {
    PathInput(String),
    PathChange,
    ContentClicked(Content),
    Sort,
}

impl Application for FilePicker {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (FilePicker, Command<Self::Message>) {
        let cwd = env::current_dir().expect("current working directory");
        let path = cwd.clone().to_str().unwrap().to_owned();
        (
            FilePicker {
                path: path.clone(),
                content: get_dir_content(path),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("FilePicker - Iced")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::PathInput(path) => self.path = path,
            Message::PathChange => self.content = get_dir_content(self.path.clone()),
            Message::ContentClicked(content) => match content {
                Content::Directory(dir) => {
                    self.path = dir.path.clone();
                    self.content = get_dir_content(dir.path);
                }
                Content::File(file) => {
                    self.path = file.path;
                }
                Content::Corrupt => {}
            },
            Message::Sort => {}
        };

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let mut content = column!();
        let adress_bar = text_input("Path: ", &self.path)
            .on_input(Message::PathInput)
            .on_submit(Message::PathChange)
            .padding(10);

        content = content.push(adress_bar);

        let row = row!(
            text("Name").width(Length::FillPortion(2)),
            text("Size").width(Length::FillPortion(1))
        )
        .height(48.);
        let header = button(row).on_press(Message::Sort);
        content = content.push(header);

        content = content.push(self.list_dir());

        Container::new(content).padding(20).into()
    }
}

impl FilePicker {
    fn list_dir(&self) -> Element<Message> {
        let mut col = column!();

        for file in &self.content {
            let size = text(format!("{} Kb", file.size())).width(Length::FillPortion(1));
            let filename = text(file.to_string()).width(Length::FillPortion(2));
            let row = row!(filename, size);
            let item = button(row)
                .on_press(Message::ContentClicked(file.clone()))
                .height(48.);
            col = col.push(item);
        }

        scrollable(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn get_dir_content(cwd: String) -> Vec<Content> {
    let mut files = Vec::new();
    let cwd = Path::new(&cwd);
    let parent_dir = match cwd.parent() {
        Some(parent) => parent.to_str().unwrap().to_string(),
        None => cwd.to_str().unwrap().to_string(),
    };

    files.push(Content::Directory(ContentData::new(parent_dir, true)));

    match fs::read_dir(cwd) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path().to_str().unwrap().to_string();

                        if entry.path().is_file() {
                            files.push(Content::File(ContentData::new(path, false)));
                        } else {
                            files.push(Content::Directory(ContentData::new(path, false)));
                        };
                    }
                    Err(_) => files.push(Content::Corrupt),
                }
            }
        }
        Err(_) => {
            files.push(Content::Corrupt);
        }
    }

    files
}
