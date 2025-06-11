use embassy_stm32::{adc::Adc, peripherals::ADC1};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};

pub type BuddyADC1 = Mutex<ThreadModeRawMutex, Option<Adc<'static, ADC1>>>;

pub(crate) static BUDDY_ADC1: BuddyADC1 = Mutex::new(None);
