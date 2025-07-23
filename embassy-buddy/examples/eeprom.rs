#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::eeprom::Memory};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());

    let eeprom = Board::init_eeprom(p.I2C1, p.PB8, p.PB9, p.DMA1_CH6, p.DMA1_CH0);

    let byte = eeprom.current_address_read(Memory::User).await.unwrap();
    info!("User Memory Byte: {}", byte);

    let byte = eeprom.current_address_read(Memory::System).await.unwrap();
    info!("System Memory Byte: {}", byte);

    let byte = eeprom.random_address_read(Memory::User, 0).await.unwrap();
    info!("User Memory Byte: {}", byte);

    let mut bytes: [u8; 8] = [0; 8];
    eeprom
        .sequential_random_read(Memory::User, 0, &mut bytes)
        .await
        .unwrap();
    info!("User Memory Bytes: {}", bytes);

    eeprom
        .sequential_random_read(Memory::System, 0, &mut bytes)
        .await
        .unwrap();
    info!("System Memory Bytes: {}", bytes);
}
