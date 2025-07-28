use core::ops::DerefMut;

use embassy_stm32::{adc::AnyAdcChannel, peripherals::ADC1};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};

use crate::components::adc::BuddyAdc;

pub type BuddyBedPowerMonitor = BedPowerMonitor<ThreadModeRawMutex>;

/// A convenience function for initialising the bed power monitor for the board. This is re-published through the Board struct for public use.
pub(crate) fn init_buddy_bed_power_monitor(
    adc: &'static BuddyAdc<ADC1>,
    ch: AnyAdcChannel<ADC1>,
) -> BuddyBedPowerMonitor {
    BedPowerMonitor::new(adc, ch, 24.0)
}

/// Provides access to the boards bed power monitor peripheral.
pub struct BedPowerMonitor<M: RawMutex> {
    adc: &'static BuddyAdc<ADC1>,
    ch: Mutex<M, AnyAdcChannel<ADC1>>,
    max_voltage: f64,
}

impl<M: RawMutex> BedPowerMonitor<M> {
    /// Create a new instance of the Bed Power Monitor. The bed power monitor on the buddy board shares `ADC1` with the board thermistors.
    pub fn new(adc: &'static BuddyAdc<ADC1>, ch: AnyAdcChannel<ADC1>, max_voltage: f64) -> Self {
        let ch = Mutex::new(ch);
        Self {
            adc,
            ch,
            max_voltage,
        }
    }

    /// Asynchronously (waits for `ADC1` to become available) reads the bed power monitor channel and computes the voltage being delivered to the bed.
    pub async fn read(&self) -> f64 {
        let mut ch = self.ch.lock().await;
        let ch = ch.deref_mut();
        let sample = self.adc.blocking_read(ch).await;
        let v_ratio = self.adc.max_value() as f64 / sample as f64;
        v_ratio * self.max_voltage
    }

    /// Try an immediate read of the bed power channel. This functions errors if it is unable to immediately lock the adc.
    pub fn try_read(&self) -> Result<f64, TryLockError> {
        let mut ch = self.ch.try_lock()?;
        let sample = self.adc.try_blocking_read(ch.deref_mut())?;
        let v_ratio = self.adc.max_value() as f64 / sample as f64;
        Ok(v_ratio * self.max_voltage)
    }
}
