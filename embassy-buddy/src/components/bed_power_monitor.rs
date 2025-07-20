use core::ops::DerefMut;

use embassy_stm32::{adc::AnyAdcChannel, peripherals::ADC1};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};

use crate::components::adc::BuddyAdc;

pub type BuddyPowerMonitor<'a> = BedPowerMonitor<'a, ThreadModeRawMutex>;

pub struct BedPowerMonitor<'a, M: RawMutex> {
    adc: &'a BuddyAdc<ADC1>,
    ch: Mutex<M, AnyAdcChannel<ADC1>>,
    max_adc_sample: u16,
    max_voltage: f64,
}

impl<'a, M: RawMutex> BedPowerMonitor<'a, M> {
    pub fn new(
        adc: &'a BuddyAdc<ADC1>,
        ch: AnyAdcChannel<ADC1>,
        max_adc_sample: u16,
        max_voltage: f64,
    ) -> Self {
        let ch = Mutex::new(ch);
        Self {
            adc,
            ch,
            max_adc_sample,
            max_voltage,
        }
    }

    pub async fn read(&self) -> f64 {
        let sample: u16;
        {
            // Hold the mutex for as little time as possible.
            // May not need it for single-threaded applications but felt it is
            // good practice to be explicit.
            // Hold the guard.
            //let mut adc = self.adc.lock().await;
            let mut ch = self.ch.lock().await;
            // Take a sample from the channel the thermistor is on.
            sample = self.adc.blocking_read(ch.deref_mut()).await;
        }
        let v_ratio = self.max_adc_sample as f64 / sample as f64;
        v_ratio * self.max_voltage
    }

    pub fn try_read(&self) -> Result<f64, TryLockError> {
        let mut ch = self.ch.try_lock()?;
        let sample = self.adc.try_blocking_read(ch.deref_mut())?;
        let v_ratio = self.max_adc_sample as f64 / sample as f64;
        Ok(v_ratio * self.max_voltage)
    }
}
