use dasp::Sample;
use iced::{
    button, scrollable, Align, Button, Column, Container, Element, Length, Radio, Row, Rule,
    Sandbox, Scrollable, Settings, Text,
};
use impulse_editor::style;
use impulse_editor::widgets::spectrogram::BufferSize;
use impulse_editor::widgets::Spectrogram;
use native_dialog::FileDialog;
use std::sync::mpsc::{self, Receiver, Sender};

struct Channel<'a, T> {
    samples: Vec<&'a T>,
    channel: (Sender<T>, Receiver<T>),
}

impl<'a, T> Channel<'a, T>
where
    T: Sample,
    T: Default,
{
    fn new() -> Self {
        Self {
            channel: mpsc::channel(),
            samples: vec![],
        }
    }
    fn assign_sender(&self) -> Sender<T> {
        self.channel.0.clone()
    }
}

// The App's state, which contains values that the program uses.
#[derive(Default)]
struct State<'a, T> {
    audio_playing: bool,
    theme: style::Theme,
    sidebar_scroll: scrollable::State,
    play_button: button::State,
    pause_button: button::State,
    spectrogram_display_scroll: scrollable::State,
    add_new_channel_button: button::State,
    import_audio_button: button::State,
    spectrograms: Vec<Spectrogram<'a, T>>,
    channels: Vec<Channel<'a, T>>,
}

// The Events that the program will send and recieve to change values in the state.
#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(style::Theme),
    PlayButtonPressed,
    PauseButtonPressed,
    AddNewChannelButtonPressed,
    ImportAudioButtonPressed,
}

// The app itself
impl<'a, T> Sandbox for State<'a, T>
where
    T: Sample,
    T: Default,
{
    type Message = Message;
    fn new() -> Self {
        State::default()
    }

    fn title(&self) -> String {
        String::from("Impulse")
    }

    // Will be triggered when a visual component is updated
    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => self.theme = theme,
            Message::PlayButtonPressed => self.audio_playing = true,
            Message::PauseButtonPressed => self.audio_playing = false,
            Message::AddNewChannelButtonPressed => {
                self.channels.push(Channel::new());
                self.spectrograms.push(Spectrogram::<T>::new(
                    self.channels[self.channels.len() - 1].assign_sender(),
                ))
            }
            Message::ImportAudioButtonPressed => {
                let channel_out = Channel::<T>::new();

                let file = FileDialog::new()
                    .set_location("~")
                    .add_filter("FLAC Audio File", &["flac"])
                    .add_filter("MPEG-3 Audio File", &["mp3"])
                    .add_filter("Ogg-Vorbis Audio File", &["ogg"])
                    .add_filter("WAV Audio File", &["wav"])
                    .show_open_single_file()
                    .unwrap();

                if file.is_some() {
                    println!("Opening from {:?}", file.unwrap());

                    self.channels.push(channel_out);
                    self.spectrograms.push(Spectrogram::<T>::new(
                        self.channels[self.channels.len() - 1].assign_sender(),
                    ))
                }
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        // The theme selector, automatically constructing radios from available `Theme` enums. (see ./style/mod.rs)
        let choose_theme = style::Theme::ALL.iter().fold(
            Column::new().spacing(10).push(Text::new("Choose a theme:")),
            |column, theme| {
                column.push(
                    Radio::new(
                        *theme,
                        &format!("{:?}", theme),
                        Some(self.theme),
                        Message::ThemeChanged,
                    )
                    .style(self.theme),
                )
            },
        );

        let play_button = Button::new(&mut self.play_button, Text::new("Play"))
            .padding(10)
            .on_press(Message::PlayButtonPressed)
            .style(self.theme);

        let pause_button = Button::new(&mut self.pause_button, Text::new("Pause"))
            .padding(10)
            .on_press(Message::PauseButtonPressed)
            .style(self.theme);

        let add_new_channel_button = Button::new(
            &mut self.add_new_channel_button,
            Text::new("Add new channel"),
        )
        .padding(10)
        .on_press(Message::AddNewChannelButtonPressed)
        .style(self.theme);

        let import_audio_button =
            Button::new(&mut self.import_audio_button, Text::new("Import audio"))
                .padding(10)
                .on_press(Message::ImportAudioButtonPressed)
                .style(self.theme);

        let sidebar = Scrollable::new(&mut self.sidebar_scroll)
            .style(self.theme)
            .push(
                Column::new()
                    .spacing(20)
                    .padding(20)
                    .width(Length::Units(300))
                    .push(choose_theme),
            );

        let samples_clone: Vec<Vec<&T>> = self.channels.iter().map(|c| c.samples.clone()).collect();

        let col: Element<_> = self
            .spectrograms
            .iter()
            .enumerate()
            .fold(Column::new(), |acc, (i, s)| {
                let mut cloned = s.clone();
                cloned.load(samples_clone[i].clone(), BufferSize::All);
                acc.push(s.clone())
            })
            .into();

        let spectrogram_display =
            iced_graphics::Scrollable::new(&mut self.spectrogram_display_scroll)
                .style(self.theme)
                .push(col);

        let audio_playing_label = Text::new(if self.audio_playing {
            "Currently playing audio"
        } else {
            "No audio playing"
        });

        let content = Column::new()
            .padding(10)
            .push(
                Row::new()
                    .spacing(10)
                    .height(Length::Units(40))
                    .push(play_button)
                    .push(pause_button)
                    .push(Rule::vertical(0).style(self.theme))
                    .align_items(Align::Center)
                    .push(audio_playing_label)
                    .push(Rule::vertical(0).style(self.theme))
                    .push(add_new_channel_button)
                    .push(import_audio_button),
            )
            .push(Rule::horizontal(38).style(self.theme))
            .push(
                Row::new()
                    .push(sidebar)
                    .push(Rule::vertical(38).style(self.theme))
                    .push(Element::new(
                        Column::new()
                            .spacing(10)
                            .push(Text::new(format!(
                                "{} {}",
                                self.spectrograms.len().to_string(),
                                if self.spectrograms.len() == 1 {
                                    "track"
                                } else {
                                    "tracks"
                                }
                            )))
                            .push(spectrogram_display),
                    )),
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(self.theme)
            .into()
    }
}
pub fn main() -> iced::Result {
    State::<f32>::run(Settings::default())
}
