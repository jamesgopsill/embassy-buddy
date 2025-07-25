#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::steppers::Direction};
use embassy_executor::Spawner;
use embassy_printer::Printer;
use embassy_time::Timer;
use embedded_graphics::{
    image::{Image, ImageRaw, ImageRawLE},
    mono_font::{MonoTextStyle, iso_8859_16::FONT_10X20},
    pixelcolor::{Rgb565, raw::LittleEndian},
    prelude::*,
    text::{Alignment, Text},
};
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Booting...");
    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];
    let board = Board::new(&spawner, mac_addr).await;
    let _mini = Mini::new(board).await;
    info!("Mini Loaded")
}

#[derive(Debug)]
pub enum MiniError {
    MoveFailed,
    HomeFailed,
}

#[derive(Debug)]
pub enum HomeError {
    TryLockError,
}

pub struct Mini<'a> {
    board: Board<'a>,
    x_microstep: u64,
    y_microstep: u64,
    z_microstep: u64,
    e_microstep: u64,
    x_steps_per_mm: u64,
    y_steps_per_mm: u64,
    z_steps_per_mm: u64,
    e_steps_per_mm: u64,
}

impl<'a> Mini<'a> {
    async fn new(board: Board<'a>) -> Self {
        let mini = Self {
            board,
            x_microstep: 16,
            y_microstep: 16,
            z_microstep: 16,
            e_microstep: 16,
            x_steps_per_mm: 100,
            y_steps_per_mm: 100,
            z_steps_per_mm: 400,
            e_steps_per_mm: 325,
        };

        let display = &mini.board.display;
        let mut display = display.try_lock().unwrap();

        display.clear(Rgb565::BLACK).unwrap();

        info!("FERRIS!");

        let raw_image_data: ImageRaw<'_, Rgb565, LittleEndian> =
            ImageRawLE::new(include_bytes!("../assets/ferris.raw"), 86);
        // centre ferris
        const X: i32 = (240 / 2) - (86 / 2);
        const Y: i32 = (320 / 2) - 50;
        let ferris = Image::new(&raw_image_data, Point::new(X, Y));
        display.clear(Rgb565::BLACK).unwrap();
        ferris.draw(&mut *display).unwrap();

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        let text = "Embassy Mini! ^_^";
        Text::with_alignment(
            text,
            Point::new(240 / 2, 195),
            text_style,
            Alignment::Center,
        )
        .draw(&mut *display)
        .unwrap();

        drop(display);

        mini.configure_steppers().await;

        //mini.home().await.unwrap();

        mini
    }

    async fn configure_steppers(&self) {
        // X
        info!("[CONFIGURING X STEPPER]");
        let mut gconf = self.board.steppers.x.read_gconf().await.unwrap();
        gconf.mstep_reg_select = true;
        self.board.steppers.x.write(&mut gconf).await.unwrap();
        let mut chopconf = self.board.steppers.x.read_chopconf().await.unwrap();
        chopconf.mres = 4.into();
        self.board.steppers.x.write(&mut chopconf).await.unwrap();
        // Y
        info!("[CONFIGURING Y STEPPER]");
        let mut gconf = self.board.steppers.y.read_gconf().await.unwrap();
        gconf.mstep_reg_select = true;
        self.board.steppers.y.write(&mut gconf).await.unwrap();
        let mut chopconf = self.board.steppers.y.read_chopconf().await.unwrap();
        chopconf.mres = 4.into();
        self.board.steppers.y.write(&mut chopconf).await.unwrap();
        // Z
        info!("[CONFIGURING Z STEPPER]");
        let mut gconf = self.board.steppers.z.read_gconf().await.unwrap();
        gconf.mstep_reg_select = true;
        self.board.steppers.z.write(&mut gconf).await.unwrap();
        let mut chopconf = self.board.steppers.z.read_chopconf().await.unwrap();
        chopconf.mres = 4.into();
        self.board.steppers.z.write(&mut chopconf).await.unwrap();
        // E0
        info!("[CONFIGURING E0 STEPPER]");
        let mut gconf = self.board.steppers.e.read_gconf().await.unwrap();
        gconf.mstep_reg_select = true;
        self.board.steppers.e.write(&mut gconf).await.unwrap();
        let mut chopconf = self.board.steppers.e.read_chopconf().await.unwrap();
        chopconf.mres = 4.into();
        self.board.steppers.e.write(&mut chopconf).await.unwrap();
    }
}

impl<'a> Printer for Mini<'a> {
    type PrinterError = MiniError;
    type HomeError = HomeError;

    async fn relative_linear_move(
        &self,
        _x: f64,
        _y: f64,
        _z: f64,
    ) -> Result<(), Self::PrinterError> {
        Ok(())
    }

    async fn home(&self) -> Result<(), HomeError> {
        self.board.steppers.z.enable().await;
        self.board
            .steppers
            .z
            .set_direction(Direction::CounterClockwise)
            .await;

        let down_mm_per_sec: u64 = 5;
        let up_mm_per_sec: u64 = 10;

        let down_timestep_us = 1_000_000 / (self.z_steps_per_mm * down_mm_per_sec);
        info!("Down ms: {}", down_timestep_us);

        let up_timestep_us = 1_000_000 / (self.z_steps_per_mm * up_mm_per_sec);
        info!("Up ms: {}", up_timestep_us);

        // Move 5mm up?
        for _ in 0..10 * self.z_steps_per_mm {
            if self.board.steppers.z.try_step().is_err() {
                self.board.steppers.z.disable().await;
                return Err(HomeError::TryLockError);
            };
            Timer::after_micros(up_timestep_us).await
        }

        self.board
            .steppers
            .z
            .set_direction(Direction::Clockwise)
            .await;

        loop {
            if self.board.steppers.z.try_step().is_err() {
                self.board.steppers.z.disable().await;
                return Err(HomeError::TryLockError);
            }
            let c = self.board.pinda_sensor.in_contact().await;
            if c {
                self.board.steppers.z.disable().await;
                return Ok(());
            }
            Timer::after_micros(down_timestep_us).await;
        }
    }
}
