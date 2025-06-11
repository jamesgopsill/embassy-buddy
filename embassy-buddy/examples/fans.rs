#![no_std]
#![no_main]

use cortex_m::asm::nop;
use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::fan::BuddyFan};
use embassy_executor::Spawner;
use embassy_futures::join::join4;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let f = Board::init_fans(p.PE11, p.PE10, p.EXTI10, p.PE9, p.PE14, p.EXTI14, p.TIM1);

    let fut_1 = cycle_fan(&f.fan_0, "Fan 0");
    let fut_2 = cycle_fan(&f.fan_1, "Fan 1");
    let fut_3 = speed_camera(&f.fan_0, "Fan 0");
    let fut_4 = speed_camera(&f.fan_1, "Fan 1");
    let fut = join4(fut_1, fut_2, fut_3, fut_4);
    fut.await;
}

async fn cycle_fan(
    fan: &BuddyFan<'_>,
    label: &str,
) -> ! {
    let mut n = 0;
    loop {
        {
            info!("[{}] On", label);
            fan.try_set_duty_cycle_fully_on().await.unwrap();
        }
        Timer::after_secs(4).await;
        {
            info!("[{}] Off", label);
            fan.try_set_duty_cycle_fully_off().await.unwrap();
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

async fn speed_camera(
    fan: &BuddyFan<'_>,
    label: &str,
) -> ! {
    loop {
        {
            if let Ok(Some(rpm)) = fan.try_rpm().await {
                info!("[{}] RPM: {}", label, rpm);
            }
        }
        Timer::after_millis(250).await;
    }
}
