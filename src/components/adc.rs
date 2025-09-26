#![doc = include_str!("../../docs/adc.md")]
use embassy_stm32::{
    adc::{Adc, AdcChannel, Instance},
    peripherals::{ADC1, ADC2, ADC3},
};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    mutex::{Mutex, TryLockError},
};
use static_cell::StaticCell;

pub struct BuddyAdc<T: Instance> {
    adc: Mutex<ThreadModeRawMutex, Adc<'static, T>>,
}

impl<T: Instance> BuddyAdc<T> {
    pub fn new(peri: T) -> Self {
        Self {
            adc: Mutex::new(Adc::new(peri)),
        }
    }

    pub fn try_blocking_read(&self, ch: &mut impl AdcChannel<T>) -> Result<u16, TryLockError> {
        let mut adc = self.adc.try_lock()?;
        let sample = adc.blocking_read(ch);
        Ok(sample)
    }

    pub async fn blocking_read(&self, ch: &mut impl AdcChannel<T>) -> u16 {
        let mut adc = self.adc.lock().await;
        adc.blocking_read(ch)
    }

    pub fn max_value(&self) -> u16 {
        (1 << 12) - 1
    }
}

impl BuddyAdc<ADC1> {
    pub fn new_static_adc1(peri: ADC1) -> &'static Self {
        let adc = Self::new(peri);
        static ADC: StaticCell<BuddyAdc<ADC1>> = StaticCell::new();
        ADC.init(adc)
    }
}

impl BuddyAdc<ADC2> {
    pub fn new_static_adc2(peri: ADC2) -> &'static Self {
        let adc = Self::new(peri);
        static ADC: StaticCell<BuddyAdc<ADC2>> = StaticCell::new();
        ADC.init(adc)
    }
}

impl BuddyAdc<ADC3> {
    pub fn new_static_adc3(peri: ADC3) -> &'static Self {
        let adc = Self::new(peri);
        static ADC: StaticCell<BuddyAdc<ADC3>> = StaticCell::new();
        ADC.init(adc)
    }
}
