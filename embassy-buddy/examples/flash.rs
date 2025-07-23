#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::flash::Instruction};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());

    let flash = Board::init_flash(
        p.SPI3, p.PC10, p.PC12, p.PC11, p.DMA1_CH5, p.DMA1_CH2, p.PD7,
    );

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
