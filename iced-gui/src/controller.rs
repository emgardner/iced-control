use crate::gui::components::serial::SerialPortParams;


use iced::{subscription, Subscription};
use iced_driver::{DeviceCommands, DeviceDriver};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_serial::{SerialPortBuilderExt};


pub enum WorkerState {
    Disconnected,
    Ready(UnboundedReceiver<Commands>),
    Connected(UnboundedReceiver<Commands>, DeviceDriver),
    Error,
}

#[derive(Debug, Clone)]
pub enum Commands {
    Nothing,
    Disconnect,
    Connect(String, SerialPortParams),
    DeviceCommand(DeviceCommands),
}

#[derive(Debug, Clone)]
pub enum WorkerEvent {
    WorkerHandle(UnboundedSender<Commands>),
    Connected,
    Disconnected,
    McuEvent(McuEvent),
    Idle,
    Error,
}

#[derive(Debug, Clone)]
pub enum McuEvent {
    Millis(u32),
    Error,
}

#[derive(Debug, Clone)]
pub enum DeviceResponse {
    Error,
    Time(u32),
    None,
}

pub fn connect() -> Subscription<WorkerEvent> {
    struct Worker;
    subscription::unfold(
        std::any::TypeId::of::<Worker>(),
        WorkerState::Disconnected,
        |state| async move {
            match state {
                WorkerState::Disconnected => {
                    let (mtx, srx) = unbounded_channel::<Commands>();
                    (
                        Some(WorkerEvent::WorkerHandle(mtx)),
                        WorkerState::Ready(srx),
                    )
                }
                WorkerState::Ready(mut srx) => {
                    if let Some(command) = srx.recv().await {
                        match command {
                            Commands::Connect(pn, sp) => {
                                let port = tokio_serial::new(pn, sp.baudrate)
                                    .data_bits(sp.data_bits)
                                    .flow_control(tokio_serial::FlowControl::None)
                                    .stop_bits(sp.stop_bits)
                                    .parity(sp.parity)
                                    .timeout(sp.timeout)
                                    .open_native_async();
                                println!("Open result: {:?}", port);
                                match port {
                                    Ok(p) => (
                                        Some(WorkerEvent::Connected),
                                        WorkerState::Connected(srx, DeviceDriver::new(p)),
                                    ),
                                    Err(_e) => (Some(WorkerEvent::Error), WorkerState::Ready(srx)),
                                }
                            }
                            _ => (Some(WorkerEvent::Error), WorkerState::Ready(srx)),
                        }
                    } else {
                        (Some(WorkerEvent::Idle), WorkerState::Ready(srx))
                    }
                }
                WorkerState::Connected(mut srx, mut device) => {
                    if let Some(command) = srx.recv().await {
                        println!("Command: {:?}", command);
                        match command {
                            Commands::Nothing => (None, WorkerState::Connected(srx, device)),
                            Commands::Disconnect => {
                                (Some(WorkerEvent::Disconnected), WorkerState::Ready(srx))
                            }
                            Commands::DeviceCommand(cmd) => {
                                let resp = device.handle_command(cmd).await;
                                println!("{:?}", resp);
                                (None, WorkerState::Connected(srx, device))
                            }
                            _ => (Some(WorkerEvent::Error), WorkerState::Error),
                        }
                    } else {
                        (Some(WorkerEvent::Idle), WorkerState::Connected(srx, device))
                    }
                }
                WorkerState::Error => (Some(WorkerEvent::Error), WorkerState::Error),
                _ => (Some(WorkerEvent::Error), WorkerState::Error),
            }
        },
    )
}
