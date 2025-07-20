use embassy_stm32::{
    gpio::OutputType,
    peripherals::{PB0, PB1, TIM3},
    time::khz,
    timer::simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel},
};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};
use embedded_hal::pwm::SetDutyCycle;

pub type BuddyHeater<'a> = Heater<ThreadModeRawMutex, SimplePwmChannel<'a, TIM3>>;

pub struct BuddyHeaters<'a> {
    pub bed: BuddyHeater<'a>,
    pub hotend: BuddyHeater<'a>,
}

pub(crate) fn init_heaters<'a>(bed: PB0, hotend: PB1, tim: TIM3) -> BuddyHeaters<'a> {
    let bed_pin = PwmPin::new_ch3(bed, OutputType::PushPull);
    let hotend_pin = PwmPin::new_ch4(hotend, OutputType::PushPull);
    let pwm = SimplePwm::new(
        tim,
        None,
        None,
        Some(bed_pin),
        Some(hotend_pin),
        khz(21),
        Default::default(),
    );
    let channels = pwm.split();
    let bed = Heater::new(channels.ch3);
    let hotend = Heater::new(channels.ch4);
    BuddyHeaters { bed, hotend }
}

pub struct Heater<M: RawMutex, T> {
    ch: Mutex<M, T>,
}

impl<M: RawMutex, T: SetDutyCycle> Heater<M, T> {
    pub fn new(ch: T) -> Self {
        Self { ch: Mutex::new(ch) }
    }

    pub fn try_set_duty_cycle_fully_off(&self) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_fully_off().unwrap();
        Ok(())
    }

    pub async fn set_duty_cycle_fully_off(&self) {
        let mut ch = self.ch.lock().await;
        ch.set_duty_cycle_fully_off().unwrap();
    }

    pub fn try_set_duty_cycle_fully_on(&self) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_fully_on().unwrap();
        Ok(())
    }

    pub async fn set_duty_cycle_fully_on(&self) {
        let mut ch = self.ch.lock().await;
        ch.set_duty_cycle_fully_on().unwrap();
    }

    pub async fn set_duty_cycle_fraction(&self, num: u16, denom: u16) {
        let mut ch = self.ch.lock().await;
        ch.set_duty_cycle_fraction(num, denom).unwrap();
    }

    pub fn try_set_duty_cycle_fraction(&self, num: u16, denom: u16) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_fraction(num, denom).unwrap();
        Ok(())
    }

    pub async fn set_duty_cycle_percent(&self, percent: u8) {
        let mut ch = self.ch.lock().await;
        ch.set_duty_cycle_percent(percent).unwrap();
    }

    pub fn try_set_duty_cycle_percent(&self, percent: u8) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_percent(percent).unwrap();
        Ok(())
    }
}
