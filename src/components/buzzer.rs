use embedded_hal::pwm::SetDutyCycle;

pub struct Buzzer<T> {
    ch: T,
}

impl<T: SetDutyCycle> Buzzer<T> {
    pub fn new(ch: T) -> Self {
        Self { ch }
    }

    pub async fn set_duty_cycle_fully_off(&mut self) {
        self.ch.set_duty_cycle_fully_off().unwrap();
    }

    pub async fn set_duty_cycle_fully_on(&mut self) {
        self.ch.set_duty_cycle_fully_on().unwrap();
    }

    pub async fn set_duty_cycle_fraction(
        &mut self,
        num: u16,
        denom: u16,
    ) {
        self.ch.set_duty_cycle_fraction(num, denom).unwrap();
    }

    pub async fn set_duty_cycle_percent(
        &mut self,
        percent: u8,
    ) {
        self.ch.set_duty_cycle_percent(percent).unwrap();
    }
}
