//! Simple example of two tasks running concurrently.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::{AnyChannel, Channel, ExtiInput};
use embassy_stm32::gpio::{AnyPin, Input, Level, Output, Pin, Pull, Speed};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{with_timeout, Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

/// Global state
/// The signal generated by a button press
static BUTTON_SIGNAL: Signal<CriticalSectionRawMutex, ButtonEvent> = Signal::new();

enum ButtonEvent {
    ShortPress,
    LongPress,
}

/// Main Task
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Two Tasks!");
    let p = embassy_stm32::init(Default::default());

    // Spawn background tasks
    spawner
        .spawn(button_task(p.PC13.degrade(), p.EXTI13.degrade()))
        .unwrap();
    spawner.spawn(heartbeat_task(p.PA5.degrade())).unwrap();

    // Main event loop
    loop {
        match BUTTON_SIGNAL.wait().await {
            ButtonEvent::ShortPress => {
                info!("Short Press Event");
            }
            ButtonEvent::LongPress => {
                info!("Long Press Event");
            }
        };
    }
}

/// Heartbeat task that toggles an LED every 1/2 second
#[embassy_executor::task]
async fn heartbeat_task(pin: AnyPin) {
    let mut led = Output::new(pin, Level::High, Speed::Low);
    // Use a ticker instead of a timer because it guarantees that
    // it will fire every interval instead of every interval plus the time it takes
    // to toggle the led.
    // https://github.com/embassy-rs/embassy/blob/main/embassy-time/src/timer.rs#L157
    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        led.toggle();
        ticker.next().await;
    }
}

/// Task that just looks for long or short button presses
///
/// Long press is hard-coded as >= 2 seconds
///
/// Only one event signal will be sent after the long press no matter how long
/// the button is held down.
#[embassy_executor::task]
async fn button_task(pin: AnyPin, exti_chan: AnyChannel) {
    let mut button = ExtiInput::new(Input::new(pin, Pull::Down), exti_chan);
    // Do we need to debounce or is that done by Embassy?
    loop {
        button.wait_for_falling_edge().await;
        // info!("FALLING EDGE");
        match with_timeout(Duration::from_millis(2000), button.wait_for_rising_edge()).await {
            Ok(_) => {
                BUTTON_SIGNAL.signal(ButtonEvent::ShortPress);
            }
            Err(_) => {
                BUTTON_SIGNAL.signal(ButtonEvent::LongPress);
            }
        };
    }
}