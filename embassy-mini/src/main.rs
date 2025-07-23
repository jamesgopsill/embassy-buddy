#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::steppers::Direction};
use embassy_executor::Spawner;

use embassy_printer::Printer;
use embassy_time::Timer;
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
    z: f64,
}

impl<'a> Mini<'a> {
    async fn new(board: Board<'a>) -> Self {
        let mini = Self { board, z: 0.0 };
        mini.home().await.unwrap();
        mini
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
        let microstep = 64;
        // Step up just in case we're near the board
        // TODO:
        // - check we do not hit the ceiling on retraction
        // - x home (using stall guard)
        // - y home (using stall guard)
        // - move retract from the home position by a certain amout
        // - turn steps into distance (get from prusa firmware.)
        for _ in 0..200 * microstep {
            if self.board.steppers.z.try_step().is_err() {
                self.board.steppers.z.disable().await;
                return Err(HomeError::TryLockError);
            };
            Timer::after_micros(100).await
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
            Timer::after_micros(200).await;
        }
    }
}
