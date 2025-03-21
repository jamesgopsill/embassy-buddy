use embassy_stm32::timer::{simple_pwm::SimplePwmChannel, GeneralInstance4Channel};

pub struct Heater<T: GeneralInstance4Channel> {
    ch: SimplePwmChannel<'static, T>,
}

impl<T: GeneralInstance4Channel> Heater<T> {
    pub fn init(mut ch: SimplePwmChannel<'static, T>) -> Heater<T> {
        ch.disable();
        Self { ch }
    }

    pub async fn enable(&mut self) {
        self.ch.enable();
    }

    pub async fn disable(&mut self) {
        self.ch.disable();
    }

    pub async fn set_duty_cycle_fully_off(&mut self) {
        self.ch.set_duty_cycle_fully_off();
    }

    pub async fn set_duty_cycle_fully_on(&mut self) {
        self.ch.set_duty_cycle_fully_on();
    }
}
