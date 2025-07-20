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
    // Wrapping a mutex so the thermistor can be a shared reference without needed to be mutable.
    ch: Mutex<M, AnyAdcChannel<ADC1>>,
    pull_up_resistor: f64,
    max_adc_sample: u16,
    beta: f64,
    r_ref: f64,
    t_ref: f64,
}

impl<'a, M: RawMutex> Thermistor<'a, M> {
    pub fn new(
        adc: &'a BuddyAdc<ADC1>,
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
    pub async fn read(&self) -> f64 {
        let sample: u16;
        {
            // Hold the mutex for as little time as possible.
            // May not need it for single-threaded applications but felt it is
            // good practice to be explicit.
            let mut ch = self.ch.lock().await;
            let ch = ch.deref_mut();
            sample = self.adc.blocking_read(ch).await;
        }
        let v_ratio = self.max_adc_sample as f64 / sample as f64;
        let resistance = self.pull_up_resistor / (v_ratio - 1.0);
        let ln = log(resistance / self.r_ref);
        let one_over_beta = 1.0 / self.beta;
        let one_over_t0 = 1.0 / (273.15 + self.t_ref);
        let denom = (one_over_beta * ln) + one_over_t0;
        1.0 / denom
    }

    pub fn try_read(&self) -> Result<f64, TryLockError> {
        //let mut adc = self.adc.try_lock()?;
        let mut ch = self.ch.try_lock()?;
        let sample = self.adc.try_blocking_read(ch.deref_mut())?;
        let v_ratio = self.max_adc_sample as f64 / sample as f64;
        let resistance = self.pull_up_resistor / (v_ratio - 1.0);
        let ln = log(resistance / self.r_ref);
        let one_over_beta = 1.0 / self.beta;
        let one_over_t0 = 1.0 / (273.15 + self.t_ref);
        let denom = (one_over_beta * ln) + one_over_t0;
        Ok(1.0 / denom)
    }
}

/*
pub(crate) async init_thermistor<'a>(pin: PA4,
    beta: f64,
    r_ref: f64,
    t_ref: f64,
) -> BuddyThermistor<'a> {

}

pub(crate) async fn init_bed_thermistor<'a>(
    pin: PA4,
    beta: f64,
    r_ref: f64,
    t_ref: f64,
) -> BuddyThermistor<'a> {
    let adc = BUDDY_ADC1.get().await;
    Thermistor::new(
        adc,
        pin.degrade_adc(),
        4_700.0,
        MAX_ADC_SAMPLE,
        4_092.0,
        100_000.0,
        25.0,
    )
}

pub(crate) async fn init_board_thermistor<'a>(pin: PA5) -> BuddyThermistor<'a> {
    const MAX_ADC_SAMPLE: u16 = (1 << 12) - 1;
    let adc = BUDDY_ADC1.get().await;
    Thermistor::new(
        adc,
        pin.degrade_adc(),
        4_700.0,
        MAX_ADC_SAMPLE,
        4_550.0,
        100_000.0,
        25.0,
    )
}

pub(crate) async fn init_hotend_thermistor<'a>(pin: PC0) -> BuddyThermistor<'a> {
    const MAX_ADC_SAMPLE: u16 = (1 << 12) - 1;
    let adc = BUDDY_ADC1.get().await;
    Thermistor::new(
        adc,
        pin.degrade_adc(),
        4_700.0,
        MAX_ADC_SAMPLE,
        4_267.0,
        100_000.0,
        25.0,
    )
}
*/
