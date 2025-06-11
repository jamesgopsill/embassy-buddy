#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::rotary_button::BuddyRotaryButton};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let btn = Board::init_rotary_button(p.PE12, p.EXTI12);
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
