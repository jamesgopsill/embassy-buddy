#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{BoardBuilder, BuddyBuzzer};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let board = BoardBuilder::default().buzzer(true).build().await;
    // We can unwrap it as we know we have built the board with it.
    let buzzer = board.buzzer.unwrap();
    let fut = buzz(&buzzer);
    fut.await;
}

async fn buzz(buzzer: &BuddyBuzzer<'_>) {
    let mut n = 0;
    loop {
        info!("[BUZZER] buzz");
        buzzer.try_set_duty_cycle_fraction(1, 2).unwrap();
        Timer::after_millis(100).await;
        buzzer.try_set_duty_cycle_fully_off().unwrap();
        Timer::after_secs(1).await;
        n += 1;
        if n > 5 {
            break;
        }
    }
    info!("[BUZZER] finished");
}
