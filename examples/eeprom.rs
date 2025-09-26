#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{BoardBuilder, components::eeprom::Memory};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let board = BoardBuilder::default().eeprom(true).build().await;
    let eeprom = board.eeprom.unwrap();

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
