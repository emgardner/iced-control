use bytes::BytesMut;
use futures::stream::StreamExt;
use std::fmt::Write;
use std::{io, str};
use tokio::io::AsyncWriteExt;
use tokio_serial::SerialStream;
use tokio_util::codec::{Decoder, Encoder};

struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}

impl Encoder<String> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct DeviceDriver {
    port: SerialStream,
}

#[derive(Debug, Copy, Clone)]
pub enum DeviceCommands {
    PwmOn,
    PwmOff,
    PwmDuty(u8),
    PwmSetFreq(u32),
    SetGpioPin,
    ClearGpioPin,
    GetTime,
    GetStatus,
}

pub type DeviceResponse = Option<Result<String, std::io::Error>>;

impl DeviceDriver {
    pub fn new(port: SerialStream) -> Self {
        println!("New Port");
        Self { port }
    }

    pub async fn close(self) -> SerialStream {
        self.port
    }

    async fn write_command(&mut self, buffer: &[u8]) -> Result<usize, std::io::Error> {
        self.port.write(&buffer).await
    }

    async fn read_response(&mut self) -> Option<Result<String, std::io::Error>> {
        let mut reader = LineCodec.framed(&mut self.port);
        let res = reader.next().await;
        println!("{:?}", res);
        res
    }

    pub async fn handle_command(&mut self, command: DeviceCommands) -> DeviceResponse {
        let mut buff_out = String::new();
        match command {
            DeviceCommands::SetGpioPin => {
                let _ = write!(buff_out, "P\n");
            }
            DeviceCommands::ClearGpioPin => {
                let _ = write!(buff_out, "C\n");
            }
            DeviceCommands::PwmOn => {
                let _ = write!(buff_out, "E\n");
            }
            DeviceCommands::PwmOff => {
                let _ = write!(buff_out, "O\n");
            }
            DeviceCommands::PwmDuty(duty) => {
                let _ = write!(buff_out, "D{}\n", duty);
            }
            DeviceCommands::PwmSetFreq(hz) => {
                let _ = write!(buff_out, "F{}\n", hz);
            }
            _ => (),
        }
        self.write_command(&buff_out.as_bytes()).await;
        self.read_response().await
    }

    pub async fn set_gpio(&mut self) -> DeviceResponse {
        self.handle_command(DeviceCommands::SetGpioPin).await
    }

    pub async fn clear_gpio(&mut self) -> DeviceResponse {
        self.handle_command(DeviceCommands::ClearGpioPin).await
    }

    pub async fn set_pwm_hz(&mut self, hz: u32) -> DeviceResponse {
        self.handle_command(DeviceCommands::PwmSetFreq(hz)).await
    }

    pub async fn set_pwm_duty(&mut self, percent: u8) -> DeviceResponse {
        self.handle_command(DeviceCommands::PwmDuty(percent)).await
    }
}
