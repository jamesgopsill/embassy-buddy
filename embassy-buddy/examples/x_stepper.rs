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

    // !important - must initialise usart first.
    // TODO add errors to catch if trying to initialise the stepper without first initialising the usart.
    Board::init_stepper_usart(p.USART2, p.PD5);
    let mut stepper = Board::init_x_stepper(p.PD3, p.PD1, p.PD0, p.PE2, p.EXTI2).await;

    let ioin = stepper.read_ioin().await.unwrap();
    info!("Ioin enn: {}", ioin.enn);

    let fut = back_and_forth(&mut stepper);
    fut.await;
}

async fn back_and_forth(
    stepper: &mut TMC2209AsyncUart<
        '_,
        Output<'_>,
        ExtiInput<'_>,
        ThreadModeRawMutex,
        BufferedUart<'_>,
    >
) -> ! {
    stepper.enable();
    info!("Stepper Enabled");
    let mut n = 0;
    let microstep = 64;
    loop {
        n += 1;
        if n > 3 {
            break;
        }
        info!("Forward");
        stepper.set_direction(Direction::Clockwise);
        for _ in 0..100 * microstep {
            stepper.step();
            Timer::after_micros(100).await
        }
        info!("Backward");
        stepper.set_direction(Direction::CounterClockwise);
        for _ in 0..100 * microstep {
            stepper.step();
            Timer::after_micros(100).await
        }
    }
    info!("Stepper Disabled");
    stepper.disable();
    loop {
        nop();
    }
}
