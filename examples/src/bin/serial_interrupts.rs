//! This shows some of the interrupts that can be generated by UART/Serial.
//! Use a proper serial terminal to connect to the board (espmonitor and
//! espflash won't work)

//% CHIPS: esp32 esp32c2 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3
//% FEATURES: embedded-hal-02

#![no_std]
#![no_main]

use core::{cell::RefCell, fmt::Write};

use critical_section::Mutex;
use embedded_hal_02::{serial::Read, timer::CountDown};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    interrupt::{self, Priority},
    peripherals::{Interrupt, Peripherals, UART0},
    prelude::*,
    timer::TimerGroup,
    uart::{config::AtCmdConfig, Uart},
    Blocking,
};
use nb::block;

static SERIAL: Mutex<RefCell<Option<Uart<UART0, Blocking>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timg0.timer0;

    let mut uart0 = Uart::new(peripherals.UART0, &clocks);
    uart0.set_at_cmd(AtCmdConfig::new(None, None, None, b'#', None));
    uart0.set_rx_fifo_full_threshold(30).unwrap();
    uart0.listen_at_cmd();
    uart0.listen_rx_fifo_full();

    interrupt::enable(Interrupt::UART0, Priority::Priority2).unwrap();

    timer0.start(1u64.secs());

    critical_section::with(|cs| SERIAL.borrow_ref_mut(cs).replace(uart0));

    loop {
        critical_section::with(|cs| {
            let mut serial = SERIAL.borrow_ref_mut(cs);
            let serial = serial.as_mut().unwrap();
            writeln!(serial, "Hello World! Send a single `#` character or send at least 30 characters and see the interrupts trigger.").ok();
        });

        block!(timer0.wait()).unwrap();
    }
}

#[interrupt]
fn UART0() {
    critical_section::with(|cs| {
        let mut serial = SERIAL.borrow_ref_mut(cs);
        let serial = serial.as_mut().unwrap();

        let mut cnt = 0;
        while let nb::Result::Ok(_c) = serial.read() {
            cnt += 1;
        }
        writeln!(serial, "Read {} bytes", cnt,).ok();

        writeln!(
            serial,
            "Interrupt AT-CMD: {} RX-FIFO-FULL: {}",
            serial.at_cmd_interrupt_set(),
            serial.rx_fifo_full_interrupt_set(),
        )
        .ok();

        serial.reset_at_cmd_interrupt();
        serial.reset_rx_fifo_full_interrupt();
    });
}
