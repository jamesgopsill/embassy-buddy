#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
};
use embassy_time::Timer;
use embassy_tmc::{direction::Direction, tmc2209::TMC2209Minimal};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());

    let en = Output::new(p.PD3, Level::High, Speed::VeryHigh);
    let step = Output::new(p.PD1, Level::High, Speed::VeryHigh);
    let dir = Output::new(p.PD0, Level::High, Speed::VeryHigh);
    let dia = ExtiInput::new(p.PE2, p.EXTI2, Pull::None);

    let mut stepper = TMC2209Minimal::new(en, step, dir, dia);
    stepper.enable();
    info!("Stepper Enabled");
    let mut n = 0;
    let microstep = 64;
    loop {
        n += 1;
        if n > 3 {
            break;
        }
        info!("Forward");
        stepper.set_direction(Direction::Clockwise);
        for _ in 0..100 * microstep {
            stepper.step();
            Timer::after_ticks(1).await
        }
        info!("Backward");
        stepper.set_direction(Direction::CounterClockwise);
        for _ in 0..100 * microstep {
            stepper.step();
            Timer::after_ticks(1).await
        }
    }
    info!("Stepper Disabled");
    stepper.disable();
}
