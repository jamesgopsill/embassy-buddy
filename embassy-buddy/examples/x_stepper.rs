#![no_std]
#![no_main]

use cortex_m::asm::nop;
use defmt::info;
use defmt_rtt as _;
use embassy_buddy::Board;
use embassy_executor::Spawner;
use embassy_stm32::{exti::ExtiInput, gpio::Output, usart::BufferedUart};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_time::Timer;
use embassy_tmc::{direction::Direction, tmc2209::TMC2209AsyncUart};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());

    // !important - must initialise usart first otherwise stepper init will error.
    Board::init_stepper_usart(p.USART2, p.PD5);
    let stepper = Board::init_x_stepper(p.PD3, p.PD1, p.PD0, p.PE2, p.EXTI2)
        .await
        .unwrap();

    let ioin = stepper.read_ioin().await.unwrap();
    info!("Ioin enn: {}", ioin.enn);

    let fut = back_and_forth(&stepper);
    fut.await;
}

async fn back_and_forth(
    stepper: &TMC2209AsyncUart<'_, Output<'_>, ExtiInput<'_>, ThreadModeRawMutex, BufferedUart<'_>>,
) -> ! {
    stepper.enable().await;
    info!("Stepper Enabled");
    let mut n = 0;
    let microstep = 64;
    loop {
        n += 1;
        if n > 3 {
            break;
        }
        info!("Forward");
        stepper.set_direction(Direction::Clockwise).await;
        for _ in 0..100 * microstep {
            stepper.try_step().unwrap();
            Timer::after_micros(100).await
        }
        info!("Backward");
        stepper.set_direction(Direction::CounterClockwise).await;
        for _ in 0..100 * microstep {
            stepper.try_step().unwrap();
            Timer::after_micros(100).await
        }
    }
    info!("Stepper Disabled");
    stepper.disable().await;
    loop {
        nop();
    }
}
