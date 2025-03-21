#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_buddy::{pinda::Pinda, Board, BuddyMutex};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let board = Board::init(p).await;

    spawner.must_spawn(contact(&board.pinda));
}

#[embassy_executor::task()]
pub async fn contact(sensor: &'static BuddyMutex<Pinda>) -> ! {
    let mut guard = sensor.lock().await;
    let pinda = guard.as_mut().unwrap();
    loop {
        let change = pinda.on_change().await;
        info!("{}", change);
    }
}
