#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::pinda::BuddyPinda};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let pinda = Board::init_pinda(p.PA8, p.EXTI8);

    let fut = pinda_interrupt(pinda);
    fut.await;
}

async fn pinda_interrupt(sensor: BuddyPinda<'_>) -> ! {
    loop {
        if let Ok(change) = sensor.try_on_change().await {
            info!("[PINDA] {}", change);
        } else {
            info!("[PINDA] Error");
        }
    }
}
