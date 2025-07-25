#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{
    Board,
    components::{display::BuddyDisplay, thermistors::BuddyThermistor},
};
use embassy_executor::Spawner;
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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let adc = Board::init_adc1(p.ADC1);
    let probe = Board::init_default_bed_thermistor(adc, p.PA4);
    let display = Board::init_display(p.SPI2, p.PB10, p.PC3, p.PC2, p.PC9, p.PD11, p.PC8);
    render_ferris(&display);

    let fut = temp(probe, &display);
    fut.await;
}

async fn temp(sensor: BuddyThermistor<'_>, display: &BuddyDisplay<'_>) -> ! {
    let mut text: String<14> = String::new();
    let point = Point::new(50, 0);
    let size = Size::new(240, 30);
    let rect = Rectangle::new(point, size);

    loop {
        Timer::after_secs(2).await;
        let temp = sensor.read().await;
        info!("Bed Temp: {}", temp);

        let mut display = display.try_lock().unwrap();

        display.fill_solid(&rect, Rgb565::BLACK).unwrap();
        text.clear();
        write!(&mut text, "Bed: {temp:.2}").unwrap();
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        Text::with_alignment(
            text.as_str(),
            Point::new(0, 20),
            text_style,
            Alignment::Left,
        )
        .draw(&mut *display)
        .unwrap();
    }
}

fn render_ferris(display: &BuddyDisplay) {
    let mut display = display.try_lock().unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    info!("FERRIS!");

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

    info!("Rendering Done");
}
