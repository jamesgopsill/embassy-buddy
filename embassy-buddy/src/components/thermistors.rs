use core::ops::DerefMut;

use embassy_stm32::{adc::AnyAdcChannel, peripherals::ADC1};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};
use libm::log;

use crate::components::adc::BuddyAdc;

pub type BuddyThermistor<'a> = Thermistor<'a, ThreadModeRawMutex>;

pub struct BuddyThermistors<'a> {
    pub bed: BuddyThermistor<'a>,
    pub board: BuddyThermistor<'a>,
    pub hotend: BuddyThermistor<'a>,
}

pub struct Thermistor<'a, M: RawMutex> {
    adc: &'a BuddyAdc<ADC1>,
    // Wrapping a mutex so the thermistor
    // can be a shared reference without needed to be mutable.
    ch: Mutex<M, AnyAdcChannel<ADC1>>,
    pull_up_resistor: f64,
    beta: f64,
    r_ref: f64,
    t_ref: f64,
}

impl<'a, M: RawMutex> Thermistor<'a, M> {
    pub fn new(
        adc: &'a BuddyAdc<ADC1>,
        ch: AnyAdcChannel<ADC1>,
        pull_up_resistor: f64,
        beta: f64,
        r_ref: f64,
        t_ref: f64,
    ) -> Self {
        Self {
            adc,
            ch: Mutex::new(ch),
            pull_up_resistor,
            beta,
            r_ref,
            t_ref,
        }
    }

    /// Read the thermistor temperature. This will return the
    /// temperature and cache it for
    pub async fn read(&self) -> f64 {
        let mut ch = self.ch.lock().await;
        let ch = ch.deref_mut();
        let sample = self.adc.blocking_read(ch).await;
        let v_ratio = self.adc.max_value() as f64 / sample as f64;
        let resistance = self.pull_up_resistor / (v_ratio - 1.0);
        let ln = log(resistance / self.r_ref);
        let one_over_beta = 1.0 / self.beta;
        let one_over_t0 = 1.0 / (273.15 + self.t_ref);
        let denom = (one_over_beta * ln) + one_over_t0;
        1.0 / denom
    }

    pub fn try_read(&self) -> Result<f64, TryLockError> {
        let mut ch = self.ch.try_lock()?;
        let sample = self.adc.try_blocking_read(ch.deref_mut())?;
        let v_ratio = self.adc.max_value() as f64 / sample as f64;
        let resistance = self.pull_up_resistor / (v_ratio - 1.0);
        let ln = log(resistance / self.r_ref);
        let one_over_beta = 1.0 / self.beta;
        let one_over_t0 = 1.0 / (273.15 + self.t_ref);
        let denom = (one_over_beta * ln) + one_over_t0;
        Ok(1.0 / denom)
    }
}
