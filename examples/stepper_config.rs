#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{BoardBuilder, BuddyStepperExti, ChopConf, Gconf};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let board = BoardBuilder::default()
        .x_stepper(true)
        .y_stepper(true)
        .z_stepper(true)
        .e_stepper(true)
        .build()
        .await;
    let x_stepper = board.x_stepper.unwrap();
    let y_stepper = board.y_stepper.unwrap();
    let z_stepper = board.z_stepper.unwrap();
    configure(&x_stepper, "X").await;
    configure(&y_stepper, "Y").await;
    configure(&z_stepper, "Z").await;
    info!("Done")
}

async fn configure(stepper: &BuddyStepperExti<'_>, label: &str) {
    info!("Stepper {}", label);
    let mut gconf = Gconf::default();
    stepper.read_register(&mut gconf).await.unwrap();
    gconf.mstep_reg_select = true;
    stepper.write_register(&mut gconf).await.unwrap();
    let mut chopconf = ChopConf::default();
    stepper.read_register(&mut chopconf).await.unwrap();
    chopconf.mres = 4.into();
    stepper.write_register(&mut chopconf).await.unwrap();
    info!("Stepper Done");
}
