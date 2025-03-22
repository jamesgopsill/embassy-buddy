#![no_std]

use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};

pub type BuddyMutex<T> = Mutex<ThreadModeRawMutex, Option<T>>;

pub mod board;
pub use board::Board;
pub mod config;
pub mod fan;
pub mod filament_sensor;
pub mod heater;
pub mod pinda;
pub mod rotary_button;
pub mod rotary_encoder;
pub mod thermistor;
pub mod tmc2209;
