#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{
    BoardBuilder, BuddyDisplay, BuddyFilamentSensor, BuddyPinda, BuddyRotaryButton,
    BuddyRotaryEncoder, BuddyStepperExti, BuddyThermistor, Direction,
};
use embassy_executor::Spawner;
use embassy_futures::join::{join3, join5};
use embassy_time::Timer;
use embedded_graphics::{
    image::{Image, ImageRawLE},
    mono_font::{MonoTextStyle, iso_8859_16::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Rectangle,
    text::{Alignment, Text},
};
use heapless::String;
use panic_probe as _;

pub enum Temp {
    Bed,
    Board,
    Hotend,
}

impl core::fmt::Display for Temp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::Bed => "[BED]",
            Self::Board => "[BOARD]",
            Self::Hotend => "[HOTEND]",
        };
        write!(f, "{s}")
    }
}

impl defmt::Format for Temp {
    fn format(&self, f: defmt::Formatter) {
        let s = match self {
            Self::Bed => "[BED]",
            Self::Board => "[BOARD]",
            Self::Hotend => "[HOTEND]",
        };
        defmt::write!(f, "{}", s)
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Booting...");
    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];
    let board = BoardBuilder::new()
        .ethernet(&spawner, mac_addr)
        .build()
        .await;

    let pinda = &board.pinda.unwrap();
    let filament_sensor = &board.filament_sensor.unwrap();
    let rotary_button = &board.rotary_button.unwrap();
    let rotaty_encoder = &board.rotary_encoder.unwrap();
    let bed_thermistor = &board.bed_thermistor.unwrap();
    let board_thermistor = &board.board_thermistor.unwrap();
    let hotend_thermistor = &board.hotend_thermistor.unwrap();
    let display = &board.display.unwrap();
    let x_stepper = &board.x_stepper.unwrap();
    let y_stepper = &board.y_stepper.unwrap();
    let z_stepper = &board.z_stepper.unwrap();

    let fut_01 = pinda_interrupt(pinda);
    let fut_02 = filament_interrupt(filament_sensor);
    let fut_03 = click(rotary_button);
    let fut_04 = spun(rotaty_encoder);
    let fut_05 = report_temp(bed_thermistor, Temp::Bed, display);
    let fut_06 = report_temp(board_thermistor, Temp::Board, display);
    let fut_07 = report_temp(hotend_thermistor, Temp::Hotend, display);
    let fut_08 = back_and_forth(x_stepper, "[STEPPER_X]");
    let fut_09 = back_and_forth(y_stepper, "[STEPPER_Y]");
    let fut_10 = back_and_forth(z_stepper, "[STEPPER_Z]");
    let fut_11 = display_ferris(display);

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

async fn report_temp(sensor: &BuddyThermistor<'_>, label: Temp, display: &BuddyDisplay<'_>) -> ! {
    let mut text: String<20> = String::new();
    let point = match label {
        Temp::Bed => Point::new(0, 0),
        Temp::Board => Point::new(0, 22),
        Temp::Hotend => Point::new(0, 44),
    };
    let size = Size::new(240, 22);
    let rect = Rectangle::new(point, size);

    loop {
        Timer::after_secs(2).await;
        let temp = sensor.read().await;
        info!("{} Temp: {}", label, temp);

        let mut display = display.lock().await;

        display.fill_solid(&rect, Rgb565::BLACK).unwrap();
        text.clear();
        write!(&mut text, "{label}: {temp:.2}").unwrap();
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        let point = match label {
            Temp::Bed => Point::new(0, 20),
            Temp::Board => Point::new(0, 40),
            Temp::Hotend => Point::new(0, 60),
        };
        Text::with_alignment(text.as_str(), point, text_style, Alignment::Left)
            .draw(&mut *display)
            .unwrap();
    }
}

async fn back_and_forth(stepper: &BuddyStepperExti<'_>, label: &str) {
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
