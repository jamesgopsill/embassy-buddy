#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{
    Board,
    components::{
        display::BuddyDisplay, filament_sensor::BuddyFilamentSensor, pinda::BuddyPinda,
        rotary_button::BuddyRotaryButton, rotary_encoder::BuddyRotaryEncoder,
        steppers::BuddyStepperInterruptDia, thermistors::BuddyThermistor,
    },
};
use embassy_executor::Spawner;
use embassy_futures::join::{join3, join5};
use embassy_time::Timer;
use embassy_tmc::direction::Direction;
use embedded_graphics::{
    image::{Image, ImageRawLE},
    mono_font::{MonoTextStyle, iso_8859_16::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, Text},
};
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Booting...");
    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];
    let board = Board::new(&spawner, mac_addr).await;

    let fut_01 = pinda_interrupt(&board.pinda_sensor);
    let fut_02 = filament_interrupt(&board.filament_sensor);
    let fut_03 = click(&board.rotary_button);
    let fut_04 = spun(&board.rotary_encoder);
    let fut_05 = report_temp(&board.thermistors.bed, "[BED]");
    let fut_06 = report_temp(&board.thermistors.board, "[BOARD]");
    let fut_07 = report_temp(&board.thermistors.hotend, "[HOTEND]");
    let fut_08 = back_and_forth(&board.steppers.x, "[STEPPER_X]");
    let fut_09 = back_and_forth(&board.steppers.y, "[STEPPER_Y]");
    let fut_10 = back_and_forth(&board.steppers.z, "[STEPPER_Z]");
    let fut_11 = display_ferris(&board.display);

    let fut = join5(fut_01, fut_02, fut_03, fut_04, fut_05);
    let fut = join5(fut, fut_06, fut_07, fut_08, fut_09);
    let fut = join3(fut, fut_10, fut_11);
    fut.await;
}

async fn display_ferris(p: &BuddyDisplay<'_>) {
    let mut display = p.lock().await;
    display.clear(Rgb565::BLACK).unwrap();

    info!("[DISPLAY] FERRIS!");

    let raw_image_data = ImageRawLE::new(include_bytes!("../assets/ferris.raw"), 86);
    const X: i32 = (240 / 2) - (86 / 2);
    const Y: i32 = (320 / 2) - 50;
    info!("{} {}", X, Y);
    let ferris = Image::new(&raw_image_data, Point::new(X, Y));
    display.clear(Rgb565::BLACK).unwrap();
    ferris.draw(&mut *display).unwrap();

    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    let text = "Embassy Buddy! ^_^";
    Text::with_alignment(
        text,
        Point::new(240 / 2, 195),
        text_style,
        Alignment::Center,
    )
    .draw(&mut *display)
    .unwrap();

    info!("[DISPLAY] Rendering Complete");
}

async fn pinda_interrupt(p: &BuddyPinda<'_>) -> ! {
    loop {
        if let Ok(change) = p.try_on_change().await {
            info!("[PINDA]: {}", change);
        } else {
            info!("[PINDA]: Error");
        }
    }
}

async fn filament_interrupt(p: &BuddyFilamentSensor<'_>) -> ! {
    loop {
        if let Ok(change) = p.try_on_change().await {
            info!("[FILAMENT]: {}", change);
        } else {
            info!("[FILAMENT]: Error");
        }
    }
}

async fn click(btn: &BuddyRotaryButton<'_>) -> ! {
    loop {
        if btn.try_on_click().await.is_ok() {
            info!("[BTN] Click");
        } else {
            info!("[BTN] Error");
        }
    }
}

async fn spun(rot: &BuddyRotaryEncoder<'_>) -> ! {
    loop {
        if let Ok(dir) = rot.try_spun().await {
            info!("[ROTARY]: {}", dir);
        } else {
            info!("[ROTARY]: Error");
        }
    }
}

async fn report_temp(sensor: &BuddyThermistor<'_>, label: &str) -> ! {
    loop {
        Timer::after_secs(2).await;
        let temp = sensor.read().await;
        info!("{} Temp: {}", label, temp);
    }
}

async fn back_and_forth(stepper: &BuddyStepperInterruptDia<'_>, label: &str) {
    stepper.enable().await;
    let mut n = 0;
    let microstep = 64;
    loop {
        n += 1;
        if n > 3 {
            break;
        }
        info!("{} Forward", label);
        stepper.set_direction(Direction::Clockwise).await;
        for _ in 0..100 * microstep {
            stepper.try_step().unwrap();
            Timer::after_micros(200).await
        }
        info!("{} Backward", label);
        stepper.set_direction(Direction::CounterClockwise).await;
        for _ in 0..100 * microstep {
            stepper.try_step().unwrap();
            Timer::after_micros(200).await
        }
    }
    info!("{} Disabled", label);
    stepper.disable().await;
}
