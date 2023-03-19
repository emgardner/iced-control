#![no_std]
#![no_main]

pub mod app;
pub mod protocol;
use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::fmt::Write;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
};
use cortex_m_rt::{entry, exception};
use heapless::{
    pool,
    pool::singleton::{Box, Pool},
    Vec,
};
use panic_halt as _;
use stm32l4xx_hal::{
    dma::{self, DMAFrame, FrameReader, FrameSender},
    interrupt,
    pac::{self, TIM2},
    prelude::*,
    serial::{self, RxDma2, TxDma2},
    stm32,
    timer::{Event, Timer},
    pwm::{Pins, C2, Pwm}
};

use app::AppState;
use protocol::{parse_command, AppCommand};

static TIM: Mutex<RefCell<Option<Timer<TIM2>>>> = Mutex::new(RefCell::new(None));
static MILLIS: AtomicU32 = AtomicU32::new(0);
static MESSAGE_RECEIVED: AtomicBool = AtomicBool::new(false);
static MESSAGE_SENT: AtomicBool = AtomicBool::new(true);
static FRAME_SENDER: Mutex<RefCell<Option<FrameSender<Box<SerialDMA>, TxDma2, 100>>>> =
    Mutex::new(RefCell::new(None));
static FRAME_READER: Mutex<RefCell<Option<FrameReader<Box<SerialDMA>, RxDma2, 100>>>> =
    Mutex::new(RefCell::new(None));
// static PWM_HANDLE: Mutex<RefCell<Option<dyn Pins<TIM2, Channels=(bool, PB3, bool, bool)>>>> = Mutex::new(RefCell::new(None))
static PWM_HANDLE: Mutex<RefCell<Option<Pwm<TIM2, C2>>>> = Mutex::new(RefCell::new(None));
type MessageFrame = Vec<u8, 100>;
static MESSAGE: Mutex<RefCell<MessageFrame>> = Mutex::new(RefCell::new(Vec::new()));

pool!(SerialDMA: DMAFrame<100>);


#[exception]
fn SysTick() {
    MILLIS.fetch_add(1, Ordering::SeqCst);
}

// Declare the timer interrupt
#[interrupt]
fn TIM2() {
    // Run the critical section code
    free(|cs| {
        // Get the timer refernce and clear then event timeout so it isn't
        // triggered immediately
        let mut tim_ref = TIM.borrow(cs).borrow_mut();
        if let Some(ref mut tim) = tim_ref.deref_mut() {
            // Get the timer refernce and clear then event timeout so it isn't
            // triggered immediately
            tim.clear_interrupt(Event::TimeOut);
            // Add one to the millis count
            MILLIS.fetch_add(1, Ordering::SeqCst);
        }
    });
}

fn millis() -> u32 {
    MILLIS.load(Ordering::SeqCst)
}

// Declare the timer interrupt
#[interrupt]
fn DMA1_CH7() {
    free(|cs| {
        let mut fs_ref = FRAME_SENDER.borrow(cs).borrow_mut();
        if let Some(ref mut fs) = fs_ref.deref_mut() {
            if let Some(_buf) = fs.transfer_complete_interrupt() {
                // Frame sent, drop the buffer to return it too the pool
                MESSAGE_SENT.store(true, Ordering::Relaxed);
            }
        }
    });
}

#[interrupt]
fn DMA1_CH6() {
    free(|cs| {
        let mut fr_ref = FRAME_READER.borrow(cs).borrow_mut();
        if let Some(ref mut fr) = fr_ref.deref_mut() {
            if let Some(dma_buf) = SerialDMA::alloc() {
                let dma_buf = dma_buf.init(DMAFrame::new());
                let _buf = fr.transfer_complete_interrupt(dma_buf);
                // Frame sent, drop the buffer to return it too the pool
                MESSAGE_RECEIVED.store(true, Ordering::Relaxed);
            }
        }
    });
}

// Declare the timer interrupt
#[interrupt]
fn USART2() {
    free(|cs| {
        let mut fr_ref = FRAME_READER.borrow(cs).borrow_mut();
        // let mut fs_ref = FRAME_SENDER.borrow(cs).borrow_mut();
        if let Some(ref mut fr) = fr_ref.deref_mut() {
            if fr.check_character_match(true) {
                if let Some(dma_buf) = SerialDMA::alloc() {
                    let dma_buf = dma_buf.init(DMAFrame::new());
                    let mut msg = MESSAGE.borrow(cs).borrow_mut();
                    if let ref mut msg_ref = msg.deref_mut() {
                        let buf = fr.character_match_interrupt(dma_buf);
                        msg_ref.extend_from_slice(buf.read());
                        MESSAGE_RECEIVED.store(true, Ordering::Relaxed);
                    }
                    // Echo the buffer back over the serial
                    // cx.resources.frame_sender.send(buf).ok();
                }
            }
            // if let Some(_buf) = fs.transfer_complete_interrupt() {
            // Frame sent, drop the buffer to return it too the pool
            // }
        }
    });
}

#[entry]
fn main() -> ! {
    static mut MEMORY: [u8; 1024] = [0; 1024];
    SerialDMA::grow(MEMORY);
    // Get a singleton to the peripherals of our device
    let p = pac::Peripherals::take().unwrap();
    // Get a singleton to the CorePeripherals of our device. Coreperipherals differ from Peripherals
    // the CorePeripherals are common to the cortex-m family.
    let cp = stm32l4xx_hal::device::CorePeripherals::take().unwrap();
    // From my understanding the constrain method works to provide different methods from the HAL on each of it's members
    let mut flash = p.FLASH.constrain();
    // Acquire clock control handle
    let mut rcc = p.RCC.constrain();
    // Acquire power control handle
    let mut pwr = p.PWR.constrain(&mut rcc.apb1r1);
    // Set the system clock and the peripheral clocks and enables them via freeze.
    let clocks = rcc
        .cfgr
        .sysclk(80.MHz())
        .pclk1(80.MHz())
        .pclk2(80.MHz())
        .freeze(&mut flash.acr, &mut pwr);

    let mut syst = cp.SYST;
    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    syst.set_reload(80_000_000 / 1000);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();

    // On our board the LED is tied to the PA5 pin. So we will need to get access to the GPIO A bank
    // The registers for GPIO A are controlled by the AHB2 (Advanced High-performance Bus 2)
    let mut gpioa = p.GPIOA.split(&mut rcc.ahb2);
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb2);
    // We configure the user_led to be a push pull output.
    let mut user_led = gpioa
        .pa5
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    // Unmask the TIM2 interrupt to allow the interrupt to trigger
    unsafe {
        NVIC::unmask(stm32::Interrupt::TIM2);
        NVIC::unmask(stm32::Interrupt::USART2);
        NVIC::unmask(stm32::Interrupt::DMA1_CH7);
        NVIC::unmask(stm32::Interrupt::DMA1_CH6);
    }
    // Setup a timer
    // let mut ms_timer = Timer::tim2(p.TIM2, 1000.Hz(), clocks, &mut rcc.apb1r1);
    // Listen for the timeout (overflow) event
    // ms_timer.listen(Event::TimeOut);
    // Place the references into their global variables
    // Configure transmit pin
    let tx = gpioa
        .pa2
        .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    // Configure receive pin
    let rx = gpioa
        .pa3
        .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    // Create serial peripheral
    let config = stm32l4xx_hal::serial::Config::default()
        .baudrate(115200.bps())
        .parity_none()
        .stopbits(stm32l4xx_hal::serial::StopBits::STOP1)
        .character_match(b'\n');
    let mut serial = serial::Serial::usart2(p.USART2, (tx, rx), config, clocks, &mut rcc.apb1r1);

    // Listen for interrupt on reception
    serial.listen(stm32l4xx_hal::serial::Event::CharacterMatch);
    // let (mut tx, mut rx) = serial.split();
    let (tx, rx) = serial.split();
    // Get DMA Channels
    let channels = p.DMA1.split(&mut rcc.ahb1);
    // Receive Channel
    let mut dma_ch6 = channels.6;
    // Transmit Channel
    let mut dma_ch7 = channels.7;
    // Listen for Reception Event
    dma_ch6.listen(dma::Event::TransferComplete);
    // Listen for TX Complete Event
    dma_ch7.listen(dma::Event::TransferComplete);
    // Get TxDma
    let tx_dma = tx.with_dma(dma_ch7);
    let rx_dma = rx.with_dma(dma_ch6);

    let fs: FrameSender<Box<SerialDMA>, _, 100> = tx_dma.frame_sender();
    let fr = if let Some(dma_buf) = SerialDMA::alloc() {
        let dma_buf = dma_buf.init(DMAFrame::new());
        rx_dma.frame_reader(dma_buf)
    } else {
        unreachable!()
    };


    // Using channel 2 of the TIM2
    let c2 = gpiob
        .pb3
        .into_alternate(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    // Create a pwm struct with a frequency of 1khz
    let mut pwm = p.TIM2.pwm(c2, 1.kHz(), clocks, &mut rcc.apb1r1);
    // let max_duty = pwm.get_max_duty();
    // pwm.set_duty(max_duty / 4);
    // pwm.enable();

    free(|cs| {
        FRAME_SENDER.borrow(cs).replace(Some(fs));
        FRAME_READER.borrow(cs).replace(Some(fr));
        // PWM_HANDLE.borrow(cs).replace(Some(pwm))
        // TIM.borrow(cs).replace(Some(ms_timer));
    });

    let mut app = AppState::new();
    let max_duty = pwm.get_max_duty();
    app.pwm_state = true;
    app.pwm_duty_cycle = (max_duty / 4) as u16;
    let max_duty = pwm.get_max_duty();
    pwm.set_duty(max_duty / 4);
    pwm.enable();

    
    let mut t = millis();
    loop {
        if MESSAGE_RECEIVED.load(Ordering::Relaxed) {
            free(|cs| {
                let mut msg = MESSAGE.borrow(cs).borrow_mut();
                if let ref mut msg_ref = msg.deref_mut() {
                    if let Some(app_command) = parse_command(msg_ref.as_slice()) {
                        match app_command {
                            AppCommand::SetGpioPin => {
                                app.led_state = true;
                                user_led.set_high()
                            }
                            AppCommand::ClearGpioPin => {
                                app.led_state = false;
                                user_led.set_low()
                            }
                            AppCommand::PwmOn => {
                                app.pwm_state = true;
                                pwm.enable();
                            },
                            AppCommand::PwmOff => {
                                app.pwm_state = false;
                                pwm.disable();
                            },
                            AppCommand::PwmDuty(duty) => {
                                if duty > 100 {
                                } else {
                                    app.pwm_duty_cycle = duty; 
                                    let max_duty = pwm.get_max_duty();
                                    pwm.set_duty((((max_duty/100) as u16) * duty).into());
                                }
                            },
                            AppCommand::PwmSetFreq(hz) => {
                                if app.pwm_state {
                                    pwm.disable();
                                    unsafe {
                                        let mut tim_reg = &(*stm32l4xx_hal::stm32::TIM2::ptr());
                                        let mut bus_en = &(*stm32l4xx_hal::stm32::RCC::ptr()).apb1enr1;
                                        let mut bus_rst = &(*stm32l4xx_hal::stm32::RCC::ptr()).apb1rstr1;
                                        bus_en.modify(|_, w| w.tim2en().set_bit());
                                        cortex_m::asm::dsb();
                                        bus_rst.modify(|_, w| w.tim2rst().set_bit());
                                        bus_rst.modify(|_, w| w.tim2rst().clear_bit());
                                        tim_reg.ccmr1_output().modify(|_, w| w.oc2pe().set_bit().oc2m().bits(6));
                                        let clk = clocks.pclk1();
                                        let ticks = clk / hz.Hz::<1,1>();
                                        let psc = ticks / (1 << 16);
                                        tim_reg.psc.write(|w| { w.psc().bits(psc as u16) });
                                        let arr = ticks / (psc + 1);
                                        tim_reg.arr.write(|w| { w.arr().bits(arr as u32) });
                                        // tim_reg.cnt.write(|w| w.bits(0 as u32));
                                        tim_reg.cnt.reset();
                                        tim_reg.cr1.write(|w| {
                                            w.cms()
                                                .bits(0b00)
                                                .dir().clear_bit()
                                                .opm().clear_bit()
                                                .cen().set_bit()
                                                .arpe().set_bit()
                                        });
                                    }
                                    let max_duty = pwm.get_max_duty();
                                    pwm.set_duty((((max_duty/100) as u16) * app.pwm_duty_cycle).into());
                                    pwm.enable();
                                } else {
                                    unsafe {
                                        let mut tim_reg = &(*stm32l4xx_hal::stm32::TIM2::ptr());
                                        let mut bus_en = &(*stm32l4xx_hal::stm32::RCC::ptr()).apb1enr1;
                                        let mut bus_rst = &(*stm32l4xx_hal::stm32::RCC::ptr()).apb1rstr1;
                                        bus_en.modify(|_, w| w.tim2en().set_bit());
                                        cortex_m::asm::dsb();
                                        bus_rst.modify(|_, w| w.tim2rst().set_bit());
                                        bus_rst.modify(|_, w| w.tim2rst().clear_bit());
                                        tim_reg.ccmr1_output().modify(|_, w| w.oc2pe().set_bit().oc2m().bits(6));
                                        let clk = clocks.pclk1();
                                        let ticks = clk / hz.Hz::<1,1>();
                                        tim_reg.cnt.write(|w| w.bits(0 as u32));
                                        let psc = ticks / (1 << 16);
                                        tim_reg.psc.write(|w| { w.psc().bits(psc as u16) });
                                        let arr = ticks / (psc + 1);
                                        tim_reg.arr.write(|w| { w.arr().bits(arr as u32) });
                                        tim_reg.cnt.reset();
                                        tim_reg.cr1.write(|w| {
                                            w.cms()
                                                .bits(0b00)
                                                .dir().clear_bit()
                                                .opm().clear_bit()
                                                .cen().set_bit()
                                                .arpe().set_bit()
                                        });
                                    }
                                    let max_duty = pwm.get_max_duty();
                                    pwm.set_duty((((max_duty/100) as u16) * app.pwm_duty_cycle).into());
                                }
                            },
                            // AppCommand::GetTime => {},
                            // AppCommand::GetStatus => {},
                            _ => (),
                        };
                        let mut fs_ref = FRAME_SENDER.borrow(cs).borrow_mut();
                        if let Some(ref mut fs) = fs_ref.deref_mut() {
                            if let Some(dma_buf) = SerialDMA::alloc() {
                                let mut dma_buf = dma_buf.init(DMAFrame::new());
                                dma_buf.write_slice(msg_ref.as_slice());
                                if fs.send(dma_buf).is_ok() {
                                    MESSAGE_SENT.store(false, Ordering::SeqCst);
                                }
                            }
                        }
                    } else {
                        let mut fs_ref = FRAME_SENDER.borrow(cs).borrow_mut();
                        if let Some(ref mut fs) = fs_ref.deref_mut() {
                            if let Some(dma_buf) = SerialDMA::alloc() {
                                let mut dma_buf = dma_buf.init(DMAFrame::new());
                                let _ = write!(dma_buf, "X\n");
                                if fs.send(dma_buf).is_ok() {
                                    MESSAGE_SENT.store(false, Ordering::SeqCst);
                                }
                            }
                        }
                    }
                    msg_ref.clear();
                }
            });
            user_led.set_state(PinState::from(app.led_state));
            MESSAGE_RECEIVED.store(false, Ordering::SeqCst);
            // free(|cs| {
            //     let mut fr_ref = FRAME_READER.borrow(cs).borrow_mut();
            //     if let Some(ref mut fr) = fr_ref.deref_mut() {
            //         if let Some(dma_buf) = SerialDMA::alloc() {
            //             let mut dma_buf = dma_buf.init(DMAFrame::new());
            //             if fs.send(dma_buf).is_ok() {
            //                 MESSAGE_SENT.store(false, Ordering::SeqCst);
            //             }
            //         }
            //     }
            // });
        }
        // if MESSAGE_SENT.load(Ordering::Relaxed) {
        //     // let m = millis();
        //     // while (millis() - m) < 1000 {};
        //     free(|cs| {
        //         let mut fs_ref = FRAME_SENDER.borrow(cs).borrow_mut();
        //         if let Some(ref mut fs) = fs_ref.deref_mut() {
        //             if let Some(dma_buf) = SerialDMA::alloc() {
        //                 let mut dma_buf = dma_buf.init(DMAFrame::new());
        //                 // Write slice into frame
        //                 let _ = write!(dma_buf, "MILLIS: {}\n", millis());
        //                 // dma_buf.write_slice(dma);
        //                 // Echo the buffer back over the serial
        //                 if fs.send(dma_buf).is_ok() {
        //                     MESSAGE_SENT.store(false, Ordering::SeqCst);
        //                 }
        //             }
        //         }
        //     });
        // }
    }
}
