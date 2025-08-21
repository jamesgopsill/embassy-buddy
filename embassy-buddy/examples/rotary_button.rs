#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{BoardBuilder, BuddyRotaryButton};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let board = BoardBuilder::default().rotary_button(true).build().await;
    let btn = board.rotary_button.unwrap();
    let fut = click(btn);
    fut.await;
}

async fn click(btn: BuddyRotaryButton<'_>) -> ! {
    info!("Initialising Click");
    loop {
        btn.try_on_click().await.unwrap();
        info!("click");
    }
}
