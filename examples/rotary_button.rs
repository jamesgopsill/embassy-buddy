#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{components::rotary_button::RotaryButton, Board};
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let pinda = Board::init_rotary_button(p.PE12, p.EXTI12);
    let fut = click(pinda);
    fut.await;
}

async fn click(mut btn: RotaryButton<ExtiInput<'_>>) -> ! {
    info!("Initialising Click");
    loop {
        btn.on_click().await;
        info!("click");
    }
}
