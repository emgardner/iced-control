use crate::gui::app::App;
use crate::gui::protocol::Protocol;
use iced::alignment::{Alignment, Horizontal};

use crate::controller::Commands;
use iced::widget::{button, row, slider, text, Column, Container};
use iced::{Length};
use iced::{Element, Theme};

use iced_driver::DeviceCommands;
use iced_native::widget::container::{Appearance, StyleSheet};

const SPACING: f32 = 20.0;

pub struct ContainerStyles(Appearance);

impl From<ContainerStyles> for iced::theme::Container {
    fn from(cs: ContainerStyles) -> Self {
        iced::theme::Container::Custom(Box::new(cs))
    }
}

impl StyleSheet for ContainerStyles {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> Appearance {
        self.0
    }
}

pub fn center_aligned_button(
    content: String,
    width: f32,
) -> iced::widget::Button<'static, Protocol> {
    button(
        Container::new(text(content))
            .width(width)
            .center_x()
            .center_y(),
    )
}

pub fn control_page(app: &App) -> Element<Protocol> {
    // let my_app = ContainerStyles(Appearance {
    //     text_color: None,
    //     background: Some(iced::Background::Color(Color::from_rgba8(0,0,0,0.0))),
    //     border_radius: 2.0,
    //     border_width: 2.0,
    //     border_color: Color::from_rgb8(255,255,255),
    // });

    let mut main_column = Column::new()
        .spacing(SPACING)
        .align_items(Alignment::Center);

    main_column = main_column.push("Iced Device Control");
    main_column = main_column.push(
        row![
            center_aligned_button("LED ON".into(), 100.0).on_press(Protocol::WorkerCommand(
                Commands::DeviceCommand(DeviceCommands::SetGpioPin)
            )),
            center_aligned_button("LED OFF".into(), 100.0).on_press(Protocol::WorkerCommand(
                Commands::DeviceCommand(DeviceCommands::ClearGpioPin)
            ))
        ]
        .spacing(SPACING)
        .align_items(Alignment::Center),
    );

    main_column = main_column.push(
        row![
            center_aligned_button("PWM ON".into(), 100.0).on_press(Protocol::WorkerCommand(
                Commands::DeviceCommand(DeviceCommands::PwmOn)
            )),
            center_aligned_button("PWM OFF".into(), 100.0).on_press(Protocol::WorkerCommand(
                Commands::DeviceCommand(DeviceCommands::PwmOff)
            ))
        ]
        .spacing(SPACING)
        .align_items(Alignment::Center),
    );
    let mut duty: String = app.pwm_duty.to_string();
    duty.push_str(" %");
    let mut hz: String = app.pwm_frequency.to_string();
    hz.push_str(" Hz");

    main_column = main_column.push(
        row![
            slider(0..=100, app.pwm_duty, |x| { Protocol::PwmDuty(x) }).width(200),
            text(duty)
                .width(75)
                .horizontal_alignment(Horizontal::Center),
            button(Container::new("Set Duty").width(150).center_x().center_y()).on_press(
                Protocol::WorkerCommand(Commands::DeviceCommand(DeviceCommands::PwmDuty(
                    app.pwm_duty
                )))
            )
        ]
        .spacing(SPACING)
        .align_items(Alignment::Center),
    );

    main_column = main_column.push(
        row![
            slider(0..=5000, app.pwm_frequency, |x| {
                Protocol::PwmFrequency(x)
            })
            .width(200),
            text(hz).width(75).horizontal_alignment(Horizontal::Center),
            button(
                Container::new("Set Frequency")
                    .width(150)
                    .center_x()
                    .center_y()
            )
            .on_press(Protocol::WorkerCommand(Commands::DeviceCommand(
                DeviceCommands::PwmSetFreq(app.pwm_frequency)
            )))
        ]
        .spacing(SPACING)
        .align_items(Alignment::Center),
    );
    main_column =
        main_column.push(button("Close").on_press(Protocol::WorkerCommand(Commands::Disconnect)));

    Container::new(main_column)
        // .style(my_app)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}
