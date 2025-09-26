#![doc = include_str!("../../docs/lcd.md")]
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    mode::Blocking,
    peripherals::{PB10, PC2, PC3, PC8, PC9, PD11, SPI2},
    spi::{Config, MODE_3, Spi},
    time::Hertz,
};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::Mutex,
};
use embassy_time::Delay;
use embedded_hal::{
    delay::DelayNs,
    digital::OutputPin,
    spi::{ErrorKind, ErrorType, Operation, SpiBus, SpiDevice},
};
use mipidsi::{Builder, Display, interface::SpiInterface, models::ST7789, options::Orientation};
use static_cell::StaticCell;

pub type BuddyLcd<'a> = LcdDevice<ThreadModeRawMutex, Spi<'a, Blocking>, Output<'a>>;
pub type BuddyDisplay<'a> = Mutex<
    ThreadModeRawMutex,
    Display<SpiInterface<'a, BuddyLcd<'a>, Output<'a>>, ST7789, Output<'a>>,
>;

pub fn build_display<'a>(
    peri: SPI2,
    sck: PB10,
    mosi: PC3,
    miso: PC2,
    cs: PC9,
    dc: PD11,
    rst: PC8,
) -> BuddyDisplay<'a> {
    // Initialise the SPI
    let mut config = Config::default();
    config.mode = MODE_3;
    config.frequency = Hertz(16_000_000);
    let spi = Spi::new_blocking(peri, sck, mosi, miso, config);

    // Create the device.
    let cs = Output::new(cs, Level::High, Speed::VeryHigh);
    let lcd_device: BuddyLcd = LcdDevice::new(spi, cs);

    // Create an interface to the device
    static BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();
    let buf = BUFFER.init([0u8; 1024]);
    let dc = Output::new(dc, Level::Low, Speed::VeryHigh);
    let di = SpiInterface::new(lcd_device, dc, buf);

    // Create the display
    let rst = Output::new(rst, Level::Low, Speed::VeryHigh);
    let mut delay = embassy_time::Delay;
    let display = Builder::new(ST7789, di)
        .display_size(240, 320)
        .reset_pin(rst)
        .orientation(Orientation::new().flip_vertical().flip_horizontal())
        .init(&mut delay)
        .unwrap();
    Mutex::new(display)
}

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
