use iced::alignment::Alignment;
// use iced::theme::{Palette, Theme};
use iced::widget::{self, text_input};
use iced::widget::{pick_list, row, text, Column};
use iced::Element;
// use iced::{Color, Command, Length};
use iced_lazy::Component;
use iced_native;
use iced_style;
use std::time::Duration;
use tokio_serial::{self, DataBits, Parity, StopBits};

const BAUDRATES: [u32; 14] = [
    110, 300, 600, 1200, 2400, 4800, 9600, 14400, 19200, 38400, 57600, 115200, 128000, 256000,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuiParity(Parity);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuiDataBits(DataBits);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuiStopBits(StopBits);

impl GuiParity {
    const ALL: [GuiParity; 3] = [
        GuiParity(Parity::None),
        GuiParity(Parity::Odd),
        GuiParity(Parity::Even),
    ];
}

impl GuiDataBits {
    const ALL: [GuiDataBits; 4] = [
        GuiDataBits(DataBits::Five),
        GuiDataBits(DataBits::Six),
        GuiDataBits(DataBits::Seven),
        GuiDataBits(DataBits::Eight),
    ];
}

impl GuiStopBits {
    const ALL: [GuiStopBits; 2] = [GuiStopBits(StopBits::One), GuiStopBits(StopBits::Two)];
}

impl From<GuiParity> for Parity {
    fn from(item: GuiParity) -> Self {
        item.0
    }
}

impl From<GuiDataBits> for DataBits {
    fn from(item: GuiDataBits) -> Self {
        item.0
    }
}

impl From<GuiStopBits> for StopBits {
    fn from(item: GuiStopBits) -> Self {
        item.0
    }
}

impl std::fmt::Display for GuiParity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GuiParity(Parity::None) => "None",
                GuiParity(Parity::Odd) => "Odd",
                GuiParity(Parity::Even) => "Even",
            }
        )
    }
}

impl std::fmt::Display for GuiDataBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GuiDataBits(DataBits::Five) => "5",
                GuiDataBits(DataBits::Six) => "6",
                GuiDataBits(DataBits::Seven) => "7",
                GuiDataBits(DataBits::Eight) => "8",
            }
        )
    }
}

impl std::fmt::Display for GuiStopBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GuiStopBits(StopBits::One) => "1",
                GuiStopBits(StopBits::Two) => "2",
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SerialPortParams {
    pub baudrate: u32,
    pub parity: Parity,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub timeout: std::time::Duration,
}

pub struct SerialPortComponent<Message> {
    params: SerialPortParams,
    on_change: Box<dyn Fn(SerialPortParams) -> Message>,
}

impl<Message> SerialPortComponent<Message> {
    pub fn new(
        params: SerialPortParams,
        on_change: impl Fn(SerialPortParams) -> Message + 'static,
    ) -> Self {
        Self {
            params: params,
            on_change: Box::new(on_change),
        }
    }
}

impl<Message, Renderer> Component<Message, Renderer> for SerialPortComponent<Message>
where
    Renderer: iced_native::text::Renderer + 'static,
    Renderer::Theme: widget::text::StyleSheet
        + widget::text_input::StyleSheet
        + widget::pick_list::StyleSheet
        + widget::scrollable::StyleSheet
        + widget::container::StyleSheet
        + iced_native::overlay::menu::StyleSheet,
    <Renderer::Theme as iced::overlay::menu::StyleSheet>::Style:
        From<<Renderer::Theme as iced_style::pick_list::StyleSheet>::Style>,
{
    type State = ();
    type Event = SerialPortParamsMessage;

    fn update(
        &mut self,
        _state: &mut Self::State,
        event: SerialPortParamsMessage,
    ) -> Option<Message> {
        match event {
            SerialPortParamsMessage::BaudrateChanged(br) => self.params.baudrate = br,
            SerialPortParamsMessage::ParityChanged(p) => self.params.parity = p,
            SerialPortParamsMessage::DataBitsChanged(db) => self.params.data_bits = db,
            SerialPortParamsMessage::StopBitsChanged(sb) => self.params.stop_bits = sb,
            SerialPortParamsMessage::TimeoutChanged(d) => self.params.timeout = d,
            _ => (),
        };
        Some(self.on_change.as_ref()(self.params))
    }

    fn view(&self, _state: &Self::State) -> Element<'static, Self::Event, Renderer> {
        let pl = pick_list(&BAUDRATES[..], Some(self.params.baudrate), |x| {
            SerialPortParamsMessage::BaudrateChanged(x)
        });
        let c = Column::new()
            .push(text("Baudrate"))
            .push(pl)
            .align_items(Alignment::Center)
            .spacing(10);
        let plp = pick_list(
            &GuiParity::ALL[..],
            Some(GuiParity(self.params.parity)),
            |x| SerialPortParamsMessage::ParityChanged(Parity::from(x)),
        );
        let cp = Column::new()
            .push(text("Parity"))
            .push(plp)
            .align_items(Alignment::Center)
            .spacing(10);
        let pld = pick_list(
            &GuiDataBits::ALL[..],
            Some(GuiDataBits(self.params.data_bits)),
            |x| SerialPortParamsMessage::DataBitsChanged(DataBits::from(x)),
        );
        let cd = Column::new()
            .push(text("Data Bits"))
            .push(pld)
            .align_items(Alignment::Center)
            .spacing(10);
        let pls = pick_list(
            &GuiStopBits::ALL[..],
            Some(GuiStopBits(self.params.stop_bits)),
            |x| SerialPortParamsMessage::StopBitsChanged(StopBits::from(x)),
        );
        let cs = Column::new()
            .push(text("Stop Bits"))
            .push(pls)
            .align_items(Alignment::Center)
            .spacing(10);
        let ti = text_input("(ms)", &self.params.timeout.as_millis().to_string(), |x| {
            let num = x.parse::<u64>();
            match num {
                Ok(n) => {
                    SerialPortParamsMessage::TimeoutChanged(std::time::Duration::from_millis(n))
                }
                _ => SerialPortParamsMessage::None,
            }
        })
        .width(200);
        let ts = Column::new()
            .push(text("Timeout (ms)"))
            .push(ti)
            .align_items(Alignment::Center)
            .spacing(10);
        Column::new()
            .push(text("Select Port Settings"))
            .push(row![c, cp, cd, cs, ts].padding(10).spacing(10))
            .spacing(10)
            .padding(10)
            .align_items(Alignment::Center)
            .into()
    }
}

impl<'a, Message, Renderer> From<SerialPortComponent<Message>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_native::text::Renderer + 'static,
    Renderer::Theme: widget::text::StyleSheet
        + widget::text_input::StyleSheet
        + widget::container::StyleSheet
        + widget::pick_list::StyleSheet
        + widget::scrollable::StyleSheet
        + iced_native::overlay::menu::StyleSheet,
    <Renderer::Theme as iced::overlay::menu::StyleSheet>::Style:
        From<<Renderer::Theme as iced_style::pick_list::StyleSheet>::Style>,
{
    fn from(sp: SerialPortComponent<Message>) -> Self {
        iced_lazy::component(sp)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialPortParamsMessage {
    BaudrateChanged(u32),
    ParityChanged(Parity),
    DataBitsChanged(DataBits),
    StopBitsChanged(StopBits),
    TimeoutChanged(std::time::Duration),
    None,
}

impl SerialPortParams {
    pub fn new() -> Self {
        Self {
            baudrate: 9600,
            parity: Parity::None,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            timeout: Duration::from_secs(1),
        }
    }
}

// impl<Message, Renderer> Component<Message, Renderer> for SerialPortParams
// where
//     Renderer: iced_native::text::Renderer + 'static,
//     Renderer::Theme: widget::text::StyleSheet
//         + widget::pick_list::StyleSheet
//         + widget::scrollable::StyleSheet
//         + widget::container::StyleSheet
//         + iced_native::overlay::menu::StyleSheet,
//     <Renderer::Theme as iced::overlay::menu::StyleSheet>::Style:
//         From<<Renderer::Theme as iced_style::pick_list::StyleSheet>::Style>,
// {
//     type State = ();
//     type Event = SerialPortParamsMessage;
//
//     fn update(
//         &mut self,
//         _state: &mut Self::State,
//         event: SerialPortParamsMessage,
//     ) -> Option<Message> {
//         match event {
//             SerialPortParamsMessage::BaudrateChanged(br) => self.baudrate = br,
//             SerialPortParamsMessage::ParityChanged(p) => self.parity = p,
//             SerialPortParamsMessage::DataBitsChanged(db) => self.data_bits = db,
//             SerialPortParamsMessage::StopBitsChanged(sb) => self.stop_bits = sb,
//             SerialPortParamsMessage::TimeoutChanged(d) => self.timeout = d,
//         };
//         None
//     }
//
//     fn view(&self, _state: &Self::State) -> Element<'static, Self::Event, Renderer> {
//         let pl = pick_list(&BAUDRATES[..], Some(self.baudrate), |x| {
//             SerialPortParamsMessage::BaudrateChanged(x)
//         });
//         let c = Column::new()
//             .push(text("Baudrate"))
//             .push(pl)
//             .align_items(Alignment::Center)
//             .spacing(10);
//         let plp = pick_list(&GuiParity::ALL[..], Some(GuiParity(self.parity)), |x| {
//             SerialPortParamsMessage::ParityChanged(Parity::from(x))
//         });
//         let cp = Column::new()
//             .push(text("Parity"))
//             .push(plp)
//             .align_items(Alignment::Center)
//             .spacing(10);
//         Column::new()
//             .push(text("Select Port Settings"))
//             .push(row![ c, cp].padding(10).spacing(10))
//             .spacing(10)
//             .padding(10)
//             .align_items(Alignment::Center)
//             .into()
//     }
// }
// impl<'a, Message, Renderer> From<SerialPortParams> for Element<'a, Message, Renderer>
// where
//     Message: 'a,
//     Renderer: iced_native::text::Renderer + 'static,
//     Renderer::Theme: widget::text::StyleSheet
//         + widget::container::StyleSheet
//         + widget::pick_list::StyleSheet
//         + widget::scrollable::StyleSheet
//         + iced_native::overlay::menu::StyleSheet,
//     <Renderer::Theme as iced::overlay::menu::StyleSheet>::Style:
//         From<<Renderer::Theme as iced_style::pick_list::StyleSheet>::Style>,
// {
//     fn from(sp: SerialPortParams) -> Self {
//         iced_lazy::component(sp)
//     }
// }
// impl Sandbox for SerialPortParams {
//     type Message = SerialPortParamsMessage;
//
//     fn new() -> Self {
//         SerialPortParams {
//             baudrate: 9600,
//             parity: Parity::None,
//             data_bits: DataBits::Eight,
//             stop_bits: StopBits::One,
//             timeout: Duration::from_secs(1)
//         }
//     }
//
//     fn title(&self) -> String {
//         String::from("Serial Port Parameters")
//     }
//
//     fn update(&mut self, message: Self::Message) {
//         match message {
//             SerialPortParamsMessage::BaudrateChanged(b) => self.baudrate = b,
//             SerialPortParamsMessage::ParityChanged(p) => self.parity = p,
//             SerialPortParamsMessage::DataBitsChanged(db) => self.data_bits = db,
//             SerialPortParamsMessage::StopBitsChanged(sb) => self.stop_bits = sb,
//             SerialPortParamsMessage::TimeoutChanged(dur) => self.timeout =  dur
//         }
//     }
//
//     fn view(&self) -> Element<Self::Message> {
//         let pl = pick_list(
//                     &BAUDRATES[..],
//                     Some(9600),
//                     |x| { Self::Message::BaudrateChanged(x)}
//         );
//         let c = Column::new()
//                         .push(text("Baudrate"))
//                         .push(pl)
//                         .align_items(Alignment::Center)
//                         .spacing(10);
//         let plp = pick_list(
//                     &GuiParity::ALL[..],
//                     Some(GuiParity(Parity::None)),
//                     |x| { Self::Message::ParityChanged(Parity::from(x))}
//         );
//         let cp = Column::new()
//                         .push(text("Parity"))
//                         .push(plp)
//                         .align_items(Alignment::Center)
//                         .spacing(10);
//         row![
//             c,
//             cp,
//         ].padding(10).spacing(10).into()
//         // text("Hello").into()
//     }
// }
