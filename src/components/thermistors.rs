#![doc = include_str!("../../docs/thermistors.md")]
use core::ops::DerefMut;

use embassy_stm32::{adc::AnyAdcChannel, peripherals::ADC1};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};
use libm::log;

use crate::components::adc::BuddyAdc;

/// A convenience type to simplify the typing.
pub type BuddyThermistor<'a> = Thermistor<'a, ThreadModeRawMutex>;

/// A struct handling the interactions with a pull-up thermistor.
pub struct Thermistor<'a, M: RawMutex> {
    /// Requires a reference to the ADC so we can sample the voltage.
    adc: &'a BuddyAdc<ADC1>,
    // Wrapping a mutex so the thermistor can be a shared reference without needed to be mutable.
    /// The channel associated with the thermistor pin wrapped in a Mutex for interior mutability enable the Thermistor to be shared as a immutable reference.
    ch: Mutex<M, AnyAdcChannel<ADC1>>,
    /// The pull-up resistor value in the circuit.
    pull_up_resistor: f64,
    /// The beta value of the thermistor.
    beta: f64,
    /// The reference resistance of the thermistor.
    r_ref: f64,
    /// The reference temperature (usually 25C).
    t_ref: f64,
}

impl<'a, M: RawMutex> Thermistor<'a, M> {
    /// Create a new Thermistor instance.
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

    /// Read the thermistor temperature waiting for the channel to lock. `embassy-time`'s `WithTimeout` functionality is useful to catch times where code has been waiting too long.
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

    /// Try and immediatly read the Thermistor value.
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
