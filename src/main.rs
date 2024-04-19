#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
//use embassy_stm32::dma::NoDma;
use embassy_stm32::exti::{AnyChannel, Channel, ExtiInput};
use embassy_stm32::gpio::{AnyPin, Input, Level, Output, Pin, Pull, Speed};
//use embassy_stm32::time::Hertz;
//use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::{with_timeout, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello Pomodoro-AF!");
    let p = embassy_stm32::init(Default::default());

    //    let mut button = ExtiInput::new(Input::new(p.PC13, Pull::Down), p.EXTI13);
    let mut led1 = Output::new(p.PA5, Level::High, Speed::Low);
    led1.set_high();

    let interval: Duration = embassy_time::Duration::from_millis(500);

    // TODO -- Get previous state of the system from Flash

    // Spawn background tasks
    spawner
        .spawn(button_task(p.PC13.degrade(), p.EXTI13.degrade()))
        .unwrap();

    // Main event loop
    loop {
        // Heartbeat
        led1.toggle();
        Timer::after(interval).await;
    }
}

// Task that just looks for long or short button presses
#[embassy_executor::task]
async fn button_task(pin: AnyPin, ext_pin: AnyChannel) {
    let mut button = ExtiInput::new(Input::new(pin, Pull::Down), ext_pin);
    // Do we need to debounce or is that done by Embassy?
    loop {
        button.wait_for_falling_edge().await;
        info!("FALLING EDGE");
        match with_timeout(Duration::from_millis(2000), button.wait_for_rising_edge()).await {
            Ok(_) => {
                info!("Short Press");
            }
            Err(_) => {
                info!("Long Press");
            }
        };
    }
}
