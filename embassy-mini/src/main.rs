#![no_std]
#![no_main]
#![allow(unused)]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{
    Board, ChopConf, Gconf, TCoolThrs, TStep, TpwmThrs, components::steppers::Direction,
};
use embassy_executor::Spawner;
use embassy_futures::select::select;
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

#[derive(Debug)]
pub enum CartesianDirection {
    ZPositive,
    ZNegative,
    XPositive,
    XNegative,
}

impl Into<Direction> for CartesianDirection {
    fn into(self) -> Direction {
        match self {
            Self::ZPositive => Direction::CounterClockwise,
            Self::ZNegative => Direction::Clockwise,
            Self::XPositive => Direction::CounterClockwise,
            Self::XNegative => Direction::Clockwise,
        }
    }
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
            x_steps_per_mm: 100, // Is this including microsteps?
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

        Timer::after_secs(2).await;

        mini.configure_steppers().await;

        //mini.home().await.unwrap();

        info!("Error State: {}", mini.board.steppers.x.has_errored().await);
        // /mini.home_x().await.unwrap();

        mini
    }

    async fn configure_steppers(&self) {
        // X
        info!("[CONFIGURING X STEPPER]");
        let mut gconf = Gconf::default();
        self.board
            .steppers
            .x
            .read_register(&mut gconf)
            .await
            .unwrap();
        gconf.mstep_reg_select = true;
        self.board
            .steppers
            .x
            .write_register(&mut gconf)
            .await
            .unwrap();
        let mut chopconf = ChopConf::default();
        self.board
            .steppers
            .x
            .read_register(&mut chopconf)
            .await
            .unwrap();
        chopconf.mres = 4.into();
        self.board
            .steppers
            .x
            .write_register(&mut chopconf)
            .await
            .unwrap();

        let mut tcoolthrs = TCoolThrs::default();
        self.board
            .steppers
            .x
            .read_register(&mut tcoolthrs)
            .await
            .unwrap();
        let mut tstep = TStep::default();
        self.board
            .steppers
            .x
            .read_register(&mut tstep)
            .await
            .unwrap();

        let mut tpwmthrs = TpwmThrs::default();
        self.board
            .steppers
            .x
            .read_register(&mut tpwmthrs)
            .await
            .unwrap();

        let t0: u32 = tcoolthrs.tcoolthrs.into();
        let t1: u32 = tstep.tstep.into();
        let t2: u32 = tpwmthrs.tpwm_thrs.into();
        info!("{} >= {} > {}", t0, t1, t2);

        tcoolthrs.tcoolthrs = t1.into();
        self.board
            .steppers
            .x
            .write_register(&mut tcoolthrs)
            .await
            .unwrap();

        // Y
        /*
        info!("[CONFIGURING Y STEPPER]");
        let mut gconf = Gconf::default();
        self.board
            .steppers
            .y
            .read_register(&mut gconf)
            .await
            .unwrap();
        gconf.mstep_reg_select = true;
        self.board
            .steppers
            .y
            .write_register(&mut gconf)
            .await
            .unwrap();
        let mut chopconf = ChopConf::default();
        self.board
            .steppers
            .y
            .read_register(&mut chopconf)
            .await
            .unwrap();
        chopconf.mres = 4.into();
        self.board
            .steppers
            .y
            .write_register(&mut chopconf)
            .await
            .unwrap();
        // Z
        info!("[CONFIGURING Z STEPPER]");
        let mut gconf = Gconf::default();
        self.board
            .steppers
            .z
            .read_register(&mut gconf)
            .await
            .unwrap();
        gconf.mstep_reg_select = true;
        self.board
            .steppers
            .z
            .write_register(&mut gconf)
            .await
            .unwrap();
        let mut chopconf = ChopConf::default();
        self.board
            .steppers
            .z
            .read_register(&mut chopconf)
            .await
            .unwrap();
        chopconf.mres = 4.into();
        self.board
            .steppers
            .z
            .write_register(&mut chopconf)
            .await
            .unwrap();
        // E0
        info!("[CONFIGURING E0 STEPPER]");
        let mut gconf = Gconf::default();
        self.board
            .steppers
            .e
            .read_register(&mut gconf)
            .await
            .unwrap();
        gconf.mstep_reg_select = true;
        self.board
            .steppers
            .e
            .write_register(&mut gconf)
            .await
            .unwrap();
        let mut chopconf = ChopConf::default();
        self.board
            .steppers
            .e
            .read_register(&mut chopconf)
            .await
            .unwrap();
        chopconf.mres = 4.into();
        self.board
            .steppers
            .e
            .write_register(&mut chopconf)
            .await
            .unwrap();
            */
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
        todo!()
    }

    async fn home_x(&self) -> Result<(), HomeError> {
        info!("[HOME X]");
        self.board.steppers.x.enable().await;
        self.board
            .steppers
            .x
            .set_direction(CartesianDirection::XNegative.into())
            .await;

        let fut_01 = async {
            self.board.steppers.x.on_error().await;
        };
        let fut_02 = async {
            let mm_per_second = 3;
            let timestep_us = 1_000_000 / (self.z_steps_per_mm * mm_per_second);
            loop {
                if self.board.steppers.x.try_step().is_err() {
                    self.board.steppers.x.disable().await;
                }
                Timer::after_micros(timestep_us).await;
            }
        };
        info!("[HOME X] Started");
        let homed = select(fut_01, fut_02).await;
        info!("[HOME X] End Detected");

        self.board
            .steppers
            .x
            .set_direction(CartesianDirection::XPositive.into())
            .await;

        // Move back 10mm??
        let mm_per_second = 4;
        for _ in 0..10 * self.z_steps_per_mm {
            if self.board.steppers.z.try_step().is_err() {
                self.board.steppers.z.disable().await;
                return Err(HomeError::TryLockError);
            };
            Timer::after_micros(mm_per_second).await
        }

        Ok(())
    }

    async fn home_y(&self) -> Result<(), HomeError> {
        todo!()
    }

    async fn home_z(&self) -> Result<(), HomeError> {
        self.board.steppers.z.enable().await;
        self.board
            .steppers
            .z
            .set_direction(CartesianDirection::ZPositive.into())
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
            .set_direction(CartesianDirection::ZNegative.into())
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
