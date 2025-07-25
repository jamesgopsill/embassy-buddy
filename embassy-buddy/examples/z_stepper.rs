#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::steppers::BuddyStepperExti};
use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_tmc::direction::Direction;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let uart = Board::init_stepper_usart(p.USART2, p.PD5);
    let stepper = Board::init_z_stepper(uart, p.PD2, p.PD4, p.PD15, p.PE5, p.EXTI5);
    let fut = back_and_forth(&stepper);
    fut.await;
}

async fn back_and_forth(stepper: &BuddyStepperExti<'_>) {
    stepper.enable().await;
    info!("Stepper Enabled");
    let mut n = 0;
    let microstep = 64;
    loop {
        n += 1;
        if n > 3 {
            break;
        }
        info!("Clockwise");
        stepper.set_direction(Direction::Clockwise).await;
        for _ in 0..100 * microstep {
            stepper.try_step().unwrap();
            Timer::after_micros(100).await
        }
        info!("CounterClockwise");
        stepper.set_direction(Direction::CounterClockwise).await;
        for _ in 0..100 * microstep {
            stepper.try_step().unwrap();
            Timer::after_micros(100).await
        }
    }
    info!("Stepper Disabled");
    stepper.disable().await;
}
