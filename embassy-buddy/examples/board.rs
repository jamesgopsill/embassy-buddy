#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{
    Board,
    components::{
        filament_sensor::BuddyFilamentSensor, pinda::BuddyPinda, rotary_button::BuddyRotaryButton,
        rotary_encoder::BuddyRotaryEncoder, thermistor::BuddyThermistor,
    },
};
use embassy_executor::Spawner;
use embassy_futures::join::{join3, join5};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());

    let board = Board::new(p).await;

    let fut_01 = pinda_interrupt(&board.pinda_sensor);
    let fut_02 = filament_interrupt(&board.filament_sensor);
    let fut_03 = click(&board.rotary_button);
    let fut_04 = spun(&board.rotary_encoder);
    let fut_05 = report_temp(&board.thermistors.bed, "bed");
    let fut_06 = report_temp(&board.thermistors.board, "board");
    let fut_07 = report_temp(&board.thermistors.hotend, "hotend");
    //let fut_08 = back_and_forth(&board.steppers.x);
    //let fut_09 = back_and_forth(&board.steppers.y);

    let fut = join5(fut_01, fut_02, fut_03, fut_04, fut_05);
    let fut = join3(fut, fut_06, fut_07);
    fut.await;
}

async fn pinda_interrupt(p: &BuddyPinda<'_>) -> ! {
    loop {
        let change = p.try_on_change().await;
        info!("Pinda: {}", change);
    }
}

async fn filament_interrupt(p: &BuddyFilamentSensor<'_>) -> ! {
    loop {
        let change = p.try_on_change().await;
        info!("Filament: {}", change);
    }
}

async fn click(btn: &BuddyRotaryButton<'_>) -> ! {
    loop {
        let click = btn.try_on_click().await;
        info!("Click: {}", click);
    }
}

async fn spun(rot: &BuddyRotaryEncoder<'_>) -> ! {
    loop {
        let dir = rot.try_spun().await;
        info!("Spun: {}", dir);
    }
}

async fn report_temp(
    sensor: &BuddyThermistor<'_>,
    label: &str,
) -> ! {
    loop {
        Timer::after_secs(2).await;
        let temp = sensor.read_temperature().await;
        info!("{} Temp: {}", label, temp);
    }
}

/*
async fn back_and_forth(stepper: BuddyStepperInterruptDia<'_>) -> ! {
    let mut stepper = stepper.lock().await;
    stepper.enable();
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
            Timer::after_micros(200).await
        }
        info!("Backward");
        stepper.set_direction(Direction::CounterClockwise);
        for _ in 0..100 * microstep {
            stepper.step();
            Timer::after_micros(200).await
        }
    }
    info!("Stepper Disabled");
    stepper.disable();
    loop {
        Timer::after_secs(1).await;
    }
}
*/
