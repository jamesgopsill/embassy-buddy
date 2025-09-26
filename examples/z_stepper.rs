#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{BoardBuilder, BuddyStepperExti, Direction, Gconf, Ioin};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let board = BoardBuilder::default().z_stepper(true).build().await;
    let stepper = board.z_stepper.unwrap();

    let mut ioin = Ioin::default();
    stepper.read_register(&mut ioin).await.unwrap();
    info!("IOIN enn: {}", ioin.enn);

    let mut gconf = Gconf::default();
    stepper.read_register(&mut gconf).await.unwrap();
    info!("GCONF MSTEP: {:?}", gconf.mstep_reg_select);
    gconf.mstep_reg_select = false;

    info!("Writing Update");
    stepper.write_register(&mut gconf).await.unwrap();

    info!("Reading Updated GCONF");
    stepper.read_register(&mut gconf).await.unwrap();
    info!("GCONF MSTEP: {:?}", gconf.mstep_reg_select);

    let fut = back_and_forth(&stepper);
    fut.await;
}

async fn back_and_forth(stepper: &BuddyStepperExti<'_>) {
    stepper.enable().await;
    info!("Stepper Enabled");
    let mut n = 0;
    let microstep = 64;
    loop {
        n += 1;
        if n > 3 {
            break;
        }
        info!("CounterClockwise");
        stepper.set_direction(Direction::CounterClockwise).await;
        for _ in 0..200 * microstep {
            stepper.try_step().unwrap();
            Timer::after_micros(100).await
        }
        /*
        info!("Clockwise");
        stepper.set_direction(Direction::Clockwise).await;
        for _ in 0..100 * microstep {
            stepper.try_step().unwrap();
            Timer::after_micros(100).await
        }
        */
    }
    info!("Stepper Disabled");
    stepper.disable().await;
}
