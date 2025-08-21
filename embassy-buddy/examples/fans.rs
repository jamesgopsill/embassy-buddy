#![no_std]
#![no_main]

use cortex_m::asm::nop;
use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{BoardBuilder, BuddyFan};
use embassy_executor::Spawner;
use embassy_futures::join::join4;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let board = BoardBuilder::default()
        .fan_0(true)
        .fan_1(true)
        .build()
        .await;
    let fan_0 = board.fan_0.unwrap();
    let fan_1 = board.fan_1.unwrap();

    let fut_1 = cycle_fan(&fan_0, "Fan 0");
    let fut_2 = cycle_fan(&fan_1, "Fan 1");
    let fut_3 = speed_camera(&fan_0, "Fan 0");
    let fut_4 = speed_camera(&fan_1, "Fan 1");
    let fut = join4(fut_1, fut_2, fut_3, fut_4);
    fut.await;
}

async fn cycle_fan(fan: &BuddyFan<'_>, label: &str) -> ! {
    let mut n = 0;
    loop {
        {
            info!("[{}] On", label);
            fan.try_set_duty_cycle_fully_on().unwrap();
        }
        Timer::after_secs(4).await;
        {
            info!("[{}] Off", label);
            fan.try_set_duty_cycle_fully_off().unwrap();
        }
        Timer::after_secs(4).await;

        n += 1;
        if n > 3 {
            break;
        }
    }
    loop {
        nop();
    }
}

async fn speed_camera(fan: &BuddyFan<'_>, label: &str) -> ! {
    loop {
        {
            if let Ok(Some(rpm)) = fan.try_rpm().await {
                info!("[{}] RPM: {}", label, rpm);
            }
        }
        Timer::after_millis(250).await;
    }
}
