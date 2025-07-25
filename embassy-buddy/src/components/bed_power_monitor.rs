use core::ops::DerefMut;

use embassy_stm32::{adc::AnyAdcChannel, peripherals::ADC1};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};

use crate::components::adc::BuddyAdc;

pub type BuddyBedPowerMonitor = BedPowerMonitor<ThreadModeRawMutex>;

pub fn init_buddy_bed_power_monitor(
    adc: &'static BuddyAdc<ADC1>,
    ch: AnyAdcChannel<ADC1>,
) -> BuddyBedPowerMonitor {
    BedPowerMonitor::new(adc, ch, 24.0)
}

pub struct BedPowerMonitor<M: RawMutex> {
    adc: &'static BuddyAdc<ADC1>,
    ch: Mutex<M, AnyAdcChannel<ADC1>>,
    max_voltage: f64,
}

impl<M: RawMutex> BedPowerMonitor<M> {
    pub fn new(adc: &'static BuddyAdc<ADC1>, ch: AnyAdcChannel<ADC1>, max_voltage: f64) -> Self {
        let ch = Mutex::new(ch);
        Self {
            adc,
            ch,
            max_voltage,
        }
    }

    pub async fn read(&self) -> f64 {
        let mut ch = self.ch.lock().await;
        let ch = ch.deref_mut();
        let sample = self.adc.blocking_read(ch).await;
        let v_ratio = self.adc.max_value() as f64 / sample as f64;
        v_ratio * self.max_voltage
    }

    pub fn try_read(&self) -> Result<f64, TryLockError> {
        let mut ch = self.ch.try_lock()?;
        let sample = self.adc.try_blocking_read(ch.deref_mut())?;
        let v_ratio = self.adc.max_value() as f64 / sample as f64;
        Ok(v_ratio * self.max_voltage)
    }
}
