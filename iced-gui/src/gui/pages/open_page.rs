use crate::gui::app::App;
use crate::gui::components::serial::SerialPortComponent;
use crate::gui::protocol::Protocol;
use iced::alignment::Alignment;

use iced::widget::{button, row, scrollable, text, Column, Container};
use iced::Element;
use iced::{Length};


pub fn main_page(app: &App) -> Element<Protocol> {
    // let _s = iced::widget::button::Appearance {
    //     shadow_offset: Vector::default(),
    //     background: None,
    //     border_radius: 0.0,
    //     border_width: 0.0,
    //     border_color: Color::TRANSPARENT,
    //     text_color: Color::BLACK,
    // };

    let sp = SerialPortComponent::<Protocol>::new(app.params, |params| {
        Protocol::SerialPortParams(params)
    });

    let b = button("Refresh Ports").on_press(Protocol::RefreshPorts);
    let c = Column::with_children(
        app.ports
            .iter()
            .map(|port| {
                row![
                    text(&port.port_name.to_string()),
                    button("Open Port").on_press(Protocol::OpenPort(port.port_name.to_string()))
                ]
                // row![
                //     text(port).width(150),
                //     button("Open Port").on_press(Protocol::OpenPort(port.to_string()))
                // ]
                .spacing(20)
                .align_items(Alignment::Center)
                .into()
            })
            .collect(),
    )
    .spacing(10)
    .padding(10);

    let port_container = scrollable(c).height(400);

    let content = Column::new()
        .align_items(Alignment::Center)
        .spacing(10)
        .push(sp)
        .push(text("Select a port from the list below"))
        .push(b)
        // .push(c)
        .push(port_container);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}
