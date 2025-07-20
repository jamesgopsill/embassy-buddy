use embassy_stm32::{
    adc::{Adc, AdcChannel, Instance},
    peripherals::ADC1,
};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    mutex::{Mutex, TryLockError},
    once_lock::OnceLock,
};

pub(crate) static BUDDY_ADC1: OnceLock<BuddyAdc<ADC1>> = OnceLock::new();

pub(crate) const ADC_SAMPLE_MAX: u16 = (1 << 12) - 1;

pub struct BuddyAdc<T: Instance> {
    adc: Mutex<ThreadModeRawMutex, Adc<'static, T>>,
}

impl<T: Instance> BuddyAdc<T> {
    pub fn new(peripheral: T) -> Self {
        Self {
            adc: Mutex::new(Adc::new(peripheral)),
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
}

pub(crate) fn init_adc1(peripheral: ADC1) {
    let adc = BuddyAdc::new(peripheral);
    if BUDDY_ADC1.init(adc).is_err() {
        panic!("ADC already initiliased");
    }
}
