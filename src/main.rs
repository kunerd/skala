use iced::{
    Theme, Widget,
    widget::{center, text},
    window,
};

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

enum Message {}

struct Skala {}

impl Skala {
    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, msg: Message) {
        match msg {}
    }

    fn view(&self) -> impl Widget<Message> {
        center(text("Nothing, yet!"))
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }
}
