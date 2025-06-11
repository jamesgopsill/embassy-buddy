#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::buzzer::BuddyBuzzer};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let buzzer = Board::init_buzzer(p.PA0, p.TIM2);
    let fut = buzz(&buzzer);
    fut.await;
}

async fn buzz(buzzer: &BuddyBuzzer<'_>) {
    let mut n = 0;
    loop {
        info!("Buzz");
        buzzer.try_set_duty_cycle_fraction(1, 2).await.unwrap();
        Timer::after_millis(100).await;
        buzzer.try_set_duty_cycle_fully_off().await.unwrap();
        Timer::after_secs(1).await;
        n += 1;
        if n > 5 {
            break;
        }
    }
    info!("Buzz Finished");
}
