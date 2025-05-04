use embassy_stm32::{
    adc::{Adc, AnyAdcChannel},
    peripherals::ADC1,
};
use libm::log;

use crate::BuddyMutex;

pub struct Thermistor<'a, 'b> {
    adc: &'a BuddyMutex<Adc<'b, ADC1>>,
    channel: AnyAdcChannel<ADC1>,
    pull_up_resistor: f64,
    max_adc_sample: u16,
    beta: f64,
    r_ref: f64,
    t_ref: f64,
    last_temp: f64,
}

impl<'a, 'b> Thermistor<'a, 'b> {
    pub fn new(
        adc: &'a BuddyMutex<Adc<'b, ADC1>>,
        channel: AnyAdcChannel<ADC1>,
        pull_up_resistor: f64,
        max_adc_sample: u16,
        beta: f64,
        r_ref: f64,
        t_ref: f64,
    ) -> Self {
        Self {
            adc,
            channel,
            pull_up_resistor,
            max_adc_sample,
            beta,
            r_ref,
            t_ref,
            last_temp: 0.0,
        }
    }

    /// Returns the last recorded temperature
    pub fn last_recorded_temperature(&self) -> f64 {
        self.last_temp
    }

    /// Read the thermistor temperature. This will return the
    /// temperature and cache it for
    pub async fn read_temperature(&mut self) -> f64 {
        let sample: u16;
        {
            // Hold the mutex for as little time as possible.
            // May not need it for single-threaded applications but felt it is
            // good practice to be explicit.
            // Hold the guard.
            let mut adc = self.adc.lock().await;
            // Take a sample from the channel the thermistor is on.
            sample = adc.blocking_read(&mut self.channel);
        }
        let v_ratio = self.max_adc_sample as f64 / sample as f64;
        let resistance = self.pull_up_resistor / (v_ratio - 1.0);
        let ln = log(resistance / self.r_ref);
        let one_over_beta = 1.0 / self.beta;
        let one_over_t0 = 1.0 / (273.15 + self.t_ref);
        let denom = (one_over_beta * ln) + one_over_t0;
        self.last_temp = 1.0 / denom;
        self.last_temp
    }
}
