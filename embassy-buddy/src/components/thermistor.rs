use core::ops::DerefMut;

use embassy_stm32::{adc::AnyAdcChannel, peripherals::ADC1};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::Mutex,
};
use libm::log;

use crate::components::adc::BuddyADC1;

pub type BuddyThermistor<'a> = Thermistor<'a, ThreadModeRawMutex>;

pub struct Thermistor<'a, M: RawMutex> {
    adc: &'a BuddyADC1,
    ch: Mutex<M, AnyAdcChannel<ADC1>>,
    pull_up_resistor: f64,
    max_adc_sample: u16,
    beta: f64,
    r_ref: f64,
    t_ref: f64,
}

impl<'a, M: RawMutex> Thermistor<'a, M> {
    pub fn new(
        adc: &'a BuddyADC1,
        ch: AnyAdcChannel<ADC1>,
        pull_up_resistor: f64,
        max_adc_sample: u16,
        beta: f64,
        r_ref: f64,
        t_ref: f64,
    ) -> Self {
        Self {
            adc,
            ch: Mutex::new(ch),
            pull_up_resistor,
            max_adc_sample,
            beta,
            r_ref,
            t_ref,
        }
    }

    /// Read the thermistor temperature. This will return the
    /// temperature and cache it for
    pub async fn read_temperature(&self) -> f64 {
        let sample: u16;
        {
            // Hold the mutex for as little time as possible.
            // May not need it for single-threaded applications but felt it is
            // good practice to be explicit.
            // Hold the guard.
            let mut guard = self.adc.lock().await;
            let adc = guard.as_mut().unwrap();
            let mut ch = self.ch.lock().await;
            // Take a sample from the channel the thermistor is on.
            sample = adc.blocking_read(ch.deref_mut());
        }
        let v_ratio = self.max_adc_sample as f64 / sample as f64;
        let resistance = self.pull_up_resistor / (v_ratio - 1.0);
        let ln = log(resistance / self.r_ref);
        let one_over_beta = 1.0 / self.beta;
        let one_over_t0 = 1.0 / (273.15 + self.t_ref);
        let denom = (one_over_beta * ln) + one_over_t0;
        1.0 / denom
    }
}
