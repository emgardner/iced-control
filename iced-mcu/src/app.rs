#![no_std]
#![no_main]

pub struct AppState {
    pub pwm_period: u16,
    pub pwm_duty_cycle: u16,
    pub pwm_state: bool,
    pub led_state: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            pwm_period: 0,
            pwm_duty_cycle: 0,
            pwm_state: false,
            led_state: false,
        }
    }
}
