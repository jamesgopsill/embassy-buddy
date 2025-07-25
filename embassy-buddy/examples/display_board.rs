#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::Board;
use embassy_executor::Spawner;
use embedded_graphics::{
    image::{Image, ImageRawLE},
    mono_font::{MonoTextStyle, iso_8859_16::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, Text},
};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());

    let display = Board::init_display(p.SPI2, p.PB10, p.PC3, p.PC2, p.PC9, p.PD11, p.PC8);
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
