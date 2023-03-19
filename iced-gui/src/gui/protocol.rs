// tokio_serial::{
//     S
// }
use crate::controller;
use crate::gui::components::serial::SerialPortParams;

#[derive(Debug, Clone)]
pub enum Protocol {
    Debug,
    RefreshPorts,
    OpenPort(String),
    ChangeSlider(i32),
    PwmFrequency(u32),
    PwmDuty(u8),
    SerialPortParams(SerialPortParams),
    WorkerEvent(controller::WorkerEvent),
    WorkerCommand(controller::Commands),
}
