#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    mode::Blocking,
    spi::{Config, MODE_3, Spi},
    time::Hertz,
};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::Mutex,
};
use embassy_time::Delay;
use embedded_graphics::{
    image::{Image, ImageRawLE},
    pixelcolor::Rgb565,
    prelude::*,
};
use embedded_hal::{
    delay::DelayNs,
    digital::OutputPin,
    spi::{ErrorKind, ErrorType, Operation, SpiBus, SpiDevice},
};
use mipidsi::{Builder, interface::SpiInterface, models::ST7789, options::Orientation};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());

    let peri = p.SPI2;
    let sck = p.PB10; // Clock Signal
    let mosi = p.PC3;
    let miso = p.PC2;

    // Chip Signal (Note. Mode 3 SPI)
    let cs = p.PC9;
    let cs = Output::new(cs, Level::High, Speed::VeryHigh);

    // Data/Command
    // lcd_rs register select a.k.a data/commannd pin (PD11)
    let dc = Output::new(p.PD11, Level::Low, Speed::VeryHigh);

    let mut config = Config::default();
    config.mode = MODE_3;
    let spi = Spi::new_blocking(peri, sck, mosi, miso, config);
    info!("SPI created...");

    let lcd_device: BuddyLcd = LcdDevice::new(spi, cs);
    info!("LCD Device created");

    let mut buffer = [0_u8; 1024];
    let di = SpiInterface::new(lcd_device, dc, &mut buffer);

    let rst = p.PC8;
    let rst = Output::new(rst, Level::Low, Speed::VeryHigh);

    let mut delay = embassy_time::Delay;

    let mut display = Builder::new(ST7789, di)
        .display_size(240, 320)
        .reset_pin(rst)
        .orientation(Orientation::new().flip_vertical().flip_horizontal())
        .init(&mut delay)
        .unwrap();
    info!("Display created");

    display.clear(Rgb565::BLACK).unwrap();

    info!("FERRIS!");

    let raw_image_data = ImageRawLE::new(include_bytes!("../assets/ferris.raw"), 86);
    const X: i32 = (240 / 2) - (86 / 2);
    const Y: i32 = (320 / 2) - 50;
    info!("{} {}", X, Y);
    let ferris = Image::new(&raw_image_data, Point::new(X, Y));
    display.clear(Rgb565::BLACK).unwrap();
    ferris.draw(&mut display).unwrap();

    info!("Booted");
}

pub type BuddyLcd<'a> = LcdDevice<ThreadModeRawMutex, Spi<'a, Blocking>, Output<'a>>;

pub struct LcdDevice<M: RawMutex, T: SpiBus, O: OutputPin> {
    spi: Mutex<M, T>,
    cs: Mutex<M, O>,
}

impl<M: RawMutex, T: SpiBus, O: OutputPin> LcdDevice<M, T, O> {
    pub fn new(spi: T, cs: O) -> Self {
        Self {
            spi: Mutex::new(spi),
            cs: Mutex::new(cs),
        }
    }
}

impl<M: RawMutex, T: SpiBus, O: OutputPin> ErrorType for LcdDevice<M, T, O> {
    // Use their in-built ErrorKind.
    // Could abstract away in the future if needed.
    type Error = ErrorKind;
}

impl<M: RawMutex, T: SpiBus, O: OutputPin> SpiDevice for LcdDevice<M, T, O> {
    fn transaction(
        &mut self,
        operations: &mut [embedded_hal::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        match (self.cs.try_lock(), self.spi.try_lock()) {
            (Ok(mut cs), Ok(mut spi)) => {
                cs.set_low().unwrap();
                for op in operations {
                    match op {
                        Operation::Read(read) => {
                            if spi.read(read).is_err() {
                                cs.set_high().unwrap();
                                return Err(ErrorKind::Other);
                            }
                        }
                        Operation::Write(write) => {
                            if spi.write(write).is_err() {
                                cs.set_high().unwrap();
                                return Err(ErrorKind::Other);
                            }
                        }
                        Operation::DelayNs(ns) => {
                            let mut d = Delay;
                            d.delay_ns(*ns);
                        }
                        Operation::Transfer(read, write) => {
                            if spi.transfer(read, write).is_err() {
                                cs.set_high().unwrap();
                                return Err(ErrorKind::Other);
                            }
                        }
                        Operation::TransferInPlace(rw) => {
                            if spi.transfer_in_place(rw).is_err() {
                                cs.set_high().unwrap();
                                return Err(ErrorKind::Other);
                            }
                        }
                    }
                }
                cs.set_high().unwrap();
                Ok(())
            }
            (Err(_), _) => Err(ErrorKind::Other),
            (_, Err(_)) => Err(ErrorKind::Other),
        }
    }
}
