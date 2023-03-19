use super::protocol::Protocol;
use crate::controller::{connect, Commands, WorkerEvent};
use crate::gui::components::serial::SerialPortParams;
use crate::gui::pages::{control::control_page, open_page::main_page};


use iced::executor;
use iced::theme::{Palette, Theme};
use iced::widget::Container;
use iced::{Application, Element, Subscription};
use iced::{Color, Command, Length};

use tokio::sync::mpsc::UnboundedSender;
use tokio_serial::{self, SerialPortInfo};

pub enum AppState {
    HomePage,
    ControlPage,
}

pub struct App {
    pub state: AppState,
    pub slide_value: i32,
    pub pwm_duty: u8,
    pub pwm_frequency: u32,
    pub ports: Vec<SerialPortInfo>,
    pub params: SerialPortParams,
    pub device_handle: Option<UnboundedSender<Commands>>,
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Protocol;
    type Theme = Theme;

    fn new(_flags: ()) -> (App, Command<Protocol>) {
        (
            App {
                state: AppState::HomePage,
                slide_value: 0,
                pwm_duty: 50,
                pwm_frequency: 1000,
                ports: Vec::new(),
                params: SerialPortParams::new(),
                device_handle: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Iced Control")
    }

    fn update(&mut self, message: Protocol) -> Command<Protocol> {
        match message {
            Protocol::PwmFrequency(x) => {
                self.pwm_frequency = x;
                Command::none()
            }
            Protocol::PwmDuty(x) => {
                self.pwm_duty = x;
                Command::none()
            }
            Protocol::RefreshPorts => {
                if let Ok(ports) = tokio_serial::available_ports() {
                    self.ports = ports;
                }
                Command::none()
            }
            Protocol::SerialPortParams(x) => {
                self.params = x;
                Command::none()
            }
            Protocol::WorkerEvent(e) => {
                println!("Worker Event: {:?}", e);
                match e {
                    WorkerEvent::WorkerHandle(mtx) => {
                        self.device_handle = Some(mtx);
                        Command::none()
                    }
                    WorkerEvent::Connected => {
                        self.state = AppState::ControlPage;
                        Command::none()
                    }
                    WorkerEvent::Disconnected => {
                        self.state = AppState::HomePage;
                        Command::none()
                    }
                    _ => Command::none(),
                }
            }
            Protocol::WorkerCommand(cmd) => {
                if let Some(worker_handle) = &self.device_handle {
                    worker_handle.send(cmd);
                }
                Command::none()
            }
            Protocol::OpenPort(s) => {
                self.device_handle
                    .as_ref()
                    .map(|h| h.send(Commands::Connect(s, self.params)));
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn subscription(&self) -> Subscription<Protocol> {
        connect().map(Protocol::WorkerEvent)
    }

    fn view(&self) -> Element<Protocol> {
        let c = match self.state {
            AppState::HomePage => main_page(&self),
            AppState::ControlPage => control_page(&self),
        };
        Container::new(c)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::custom(Palette {
            background: Color::from_rgb(0.24, 0.24, 0.27),
            text: Color::from_rgb8(244, 244, 245),
            primary: Color::from_rgb8(198, 151, 75),
            success: Color::from_rgb8(250, 204, 21),
            danger: Color::from_rgb8(248, 113, 113),
        })
    }
}
