#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{BoardBuilder, components::flash::Instruction};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");

    let board = BoardBuilder::default().flash(true).build().await;
    let flash = board.flash.unwrap();

    let data = flash
        .send_instruction(Instruction::ReadStatusRegister1)
        .await;
    info!("[Register1 Response] {}", data);

    let data = flash
        .send_instruction(Instruction::ReadStatusRegister2)
        .await;
    info!("[Register2 Response] {}", data);

    let data = flash
        .send_instruction(Instruction::ReadStatusRegister3)
        .await;
    info!("[Register3 Response] {}", data);

    let register = flash.read_status_register_one().await;
    info!("Busy: {}", register.busy);
}
