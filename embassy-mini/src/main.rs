#![no_std]
#![no_main]

use defmt::{info, println};
use defmt_rtt as _;
use embassy_buddy::{
    Board,
    components::{
        rotary_button::BuddyRotaryButton,
        rotary_encoder::{BuddyRotaryEncoder, Direction},
        steppers::{self, BuddyStepperInterruptDia},
    },
};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_time::Timer;
use panic_probe as _;

//pub mod api;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let peripherals = embassy_stm32::init(Default::default());
    let board = Board::new(peripherals).await;

    let fut_01 = click(&board.rotary_button, &board.steppers.x);
    let fut_02 = spun(&board.rotary_encoder, &board.steppers.x);
    let fut = join(fut_01, fut_02);
    fut.await;
}

async fn click(btn: &BuddyRotaryButton<'_>, stepper: &BuddyStepperInterruptDia<'_>) -> ! {
    info!("[Button] init");
    stepper.disable().await;
    let mut enabled = false;
    loop {
        btn.try_on_click().await.unwrap();
        if enabled {
            info!("[Button] Toggling off");
            stepper.disable().await;
            enabled = false;
        } else {
            info!("[Button] Toggling on");
            stepper.enable().await;
            enabled = true;
        }
    }
}

async fn spun(rotary: &BuddyRotaryEncoder<'_>, stepper: &BuddyStepperInterruptDia<'_>) -> ! {
    info!("[SPUN] Init");
    loop {
        let dir = rotary.try_spun().await.unwrap();
        println!("[SPUN] {}", dir);
        match dir {
            Direction::Clockwise => {
                println!("[SPUN] Moving");
                stepper
                    .try_set_direction(steppers::Direction::Clockwise)
                    .unwrap();
                for _ in 0..64 {
                    stepper.try_step().unwrap();
                    Timer::after_micros(100).await
                }
            }
            Direction::CounterClockwise => {
                println!("[SPUN] Moving");
                stepper
                    .try_set_direction(steppers::Direction::CounterClockwise)
                    .unwrap();
                for _ in 0..64 {
                    stepper.try_step().unwrap();
                    Timer::after_micros(100).await
                }
            }
        }
    }
}
