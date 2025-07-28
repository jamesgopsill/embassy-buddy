#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::steppers::BuddySteppers};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Booting...");
    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];
    let board = Board::new(&spawner, mac_addr).await;
    configure(&board.steppers).await;
    info!("Done")
}

async fn configure(steppers: &BuddySteppers<'_>) {
    info!("Stepper X");
    let mut gconf = steppers.x.read_gconf().await.unwrap();
    gconf.mstep_reg_select = true;
    steppers.x.write(&mut gconf).await.unwrap();
    let mut chopconf = steppers.x.read_chopconf().await.unwrap();
    chopconf.mres = 4.into();
    steppers.x.write(&mut chopconf).await.unwrap();
    info!("Stepper Done");

    info!("Stepper Y");
    let mut gconf = steppers.y.read_gconf().await.unwrap();
    gconf.mstep_reg_select = true;
    steppers.y.write(&mut gconf).await.unwrap();
    let mut chopconf = steppers.y.read_chopconf().await.unwrap();
    chopconf.mres = 4.into();
    steppers.y.write(&mut chopconf).await.unwrap();

    info!("Stepper Z");
    let mut gconf = steppers.z.read_gconf().await.unwrap();
    gconf.mstep_reg_select = true;
    steppers.z.write(&mut gconf).await.unwrap();
    let mut chopconf = steppers.z.read_chopconf().await.unwrap();
    chopconf.mres = 4.into();
    steppers.z.write(&mut chopconf).await.unwrap();

    info!("Stepper E");
    let mut gconf = steppers.e.read_gconf().await.unwrap();
    gconf.mstep_reg_select = true;
    steppers.e.write(&mut gconf).await.unwrap();
    let mut chopconf = steppers.e.read_chopconf().await.unwrap();
    chopconf.mres = 4.into();
    steppers.e.write(&mut chopconf).await.unwrap();
}
