#![no_std]
#![no_main]

use btoi::btoi;

#[derive(Debug, PartialEq)]
pub enum AppCommand {
    PwmOn,
    PwmOff,
    PwmDuty(u16),
    PwmSetFreq(u32),
    SetGpioPin,
    ClearGpioPin,
    GetTime,
    GetStatus,
}

pub fn parse_pwm_duty(input: &[u8]) -> Option<AppCommand> {
    if input.len() <= 2 {
        return None;
    }
    if let Ok(num) = btoi::<u16>(&input[1..(input.len() - 1)]) {
        Some(AppCommand::PwmDuty(num))
    } else {
        None
    }
}

pub fn parse_pwm_frequency(input: &[u8]) -> Option<AppCommand> {
    if input.len() <= 2 {
        return None;
    }
    if let Ok(num) = btoi::<u32>(&input[1..(input.len() - 1)]) {
        Some(AppCommand::PwmSetFreq(num))
    } else {
        None
    }
}

pub fn parse_command(buffer: &[u8]) -> Option<AppCommand> {
    match buffer[0] {
        b'E' => Some(AppCommand::PwmOn),
        b'O' => Some(AppCommand::PwmOff),
        b'D' => parse_pwm_duty(buffer),
        b'F' => parse_pwm_frequency(buffer),
        b'P' => Some(AppCommand::SetGpioPin),
        b'C' => Some(AppCommand::ClearGpioPin),
        b'T' => Some(AppCommand::GetTime),
        b'S' => Some(AppCommand::GetStatus),
        _ => None,
    }
}
