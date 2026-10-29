use skala::{Candidate, Error, Reviewed};

use iced::{
    Font, Task, Theme, Widget, font,
    widget::{button, center, column, progress_bar, row, stack, text, text_input},
    window,
};

use std::path::{Path, PathBuf};

fn main() -> iced::Result {
    iced::application(Skala::new, Skala::update, Skala::view)
        .theme(Skala::theme)
        .window(window::Settings {
            resizable: true,
            platform_specific: window::settings::PlatformSpecific {
                application_id: "iced".to_string(),
                override_redirect: false,
            },
            ..Default::default()
        })
        .antialiasing(true)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    ChooseSearchPath,
    SearchPathChoosen(Option<PathBuf>),
    CandidatesFound(Result<Vec<Candidate>, Error>),
}

enum Skala {
    Setup {
        path: Option<PathBuf>,
    },
    Searching,
    Review {
        candidates: Vec<Candidate>,
        reviewed: Vec<Reviewed>,
    },
}

impl Skala {
    fn new() -> Self {
        Self::Setup { path: None }
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::ChooseSearchPath => Task::perform(pick_path(), Message::SearchPathChoosen),
            Message::SearchPathChoosen(Some(path)) => {
                *self = Self::Searching;

                Task::perform(Candidate::fetch_all(path, "mkv"), Message::CandidatesFound)
            }
            Message::SearchPathChoosen(None) => Task::none(),
            Message::CandidatesFound(Ok(candidates)) => {
                *self = Self::Review {
                    candidates,
                    reviewed: vec![],
                };

                Task::none()
            }
            Message::CandidatesFound(Err(err)) => {
                eprintln!("{err}");

                Task::none()
            }
        }
    }

    fn view(&self) -> impl Widget<Message> {
        match self {
            Skala::Setup { path } => {
                let input = text_input(
                    "Chose a path to search ...",
                    path.as_ref()
                        .map(PathBuf::as_path)
                        .and_then(Path::to_str)
                        .unwrap_or(""),
                );

                let search = button("...").on_press(Message::ChooseSearchPath);

                center(row![input, search].spacing(10).width(500)).boxed()
            }
            Skala::Searching => center(text("Searching ...")).boxed(),
            Skala::Review {
                candidates,
                reviewed,
            } => {
                let progress = {
                    let cur = reviewed.len() as f32;
                    let count = candidates.len() as f32;

                    let label = text!("{}/{}", cur, count)
                        .size(14)
                        .font(Font::MONOSPACE.weight(font::Weight::ExtraBold))
                        .color(self.theme().palette().primary.strong.text);

                    let bar = progress_bar(0.0..=1.0, cur / count)
                        .style(progress_bar::secondary)
                        .girth(16);

                    stack![bar, center(label)]
                };

                let list = column(candidates.iter().filter_map(|c| {
                    let path = c.path.as_path().to_str().map(text)?;
                    let hash = text!("{:02x}", c.hash)
                        .size(12)
                        .color(self.theme().palette().secondary.weak.color);

                    Some(column![path, hash].spacing(2))
                }))
                .spacing(5);

                column![progress, center(list).padding(10)]
                    .spacing(20)
                    .boxed()
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }
}

async fn pick_path() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|handle| handle.path().to_owned())
}
