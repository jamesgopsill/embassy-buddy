#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, BuddyStepperInterrupt, components::buzzer::Buzzer};
use embassy_executor::Spawner;
use embassy_stm32::{peripherals::TIM2, timer::simple_pwm::SimplePwmChannel};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let mut buzzer = Board::init_buzzer(p.PA0, p.TIM2);
    let fut = buzz(&mut buzzer);
    fut.await;
}

async fn buzz(buzzer: &mut Buzzer<SimplePwmChannel<'_, TIM2>>) {
    let mut n = 0;
    loop {
        info!("Buzz");
        buzzer.set_duty_cycle_fraction(1, 2).await;
        Timer::after_millis(100).await;
        buzzer.set_duty_cycle_fully_off().await;
        Timer::after_secs(1).await;
        n += 1;
        if n > 5 {
            break;
        }
    }
    info!("Buzz Finished");
}

async fn back_and_forth(stepper: &mut BuddyStepperInterrupt<'_, '_>) -> ! {
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
            Timer::after_ticks(1).await
        }
        info!("Backward");
        stepper.set_direction(Direction::CounterClockwise);
        for _ in 0..100 * microstep {
            stepper.step();
            Timer::after_ticks(1).await
        }
    }
    info!("Stepper Disabled");
    stepper.disable();
    loop {
        nop();
    }
}
