use embassy_stm32::{
    exti::ExtiInput,
    timer::{simple_pwm::SimplePwmChannel, GeneralInstance4Channel},
};
use embassy_time::{Duration, Instant, WithTimeout};

pub struct Fan<T: GeneralInstance4Channel> {
    ch: SimplePwmChannel<'static, T>,
    inp: ExtiInput<'static>,
}

impl<T: GeneralInstance4Channel> Fan<T> {
    pub async fn init(ch: SimplePwmChannel<'static, T>, inp: ExtiInput<'static>) -> Fan<T> {
        Self { ch, inp }
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

    pub async fn rpm(&mut self) -> Option<f64> {
        let minute_in_millis: f64 = 1_000_000.0 * 60.0;
        let ok = self
            .inp
            .wait_for_any_edge()
            .with_timeout(Duration::from_millis(10))
            .await
            .is_ok();
        if ok {
            let tick_one = Instant::now().as_micros() as f64;
            let ok = self
                .inp
                .wait_for_any_edge()
                .with_timeout(Duration::from_millis(10))
                .await
                .is_ok();
            if ok {
                let tick_two = Instant::now().as_micros() as f64;
                let delta = tick_two - tick_one;
                let rpm = minute_in_millis / (2.0 * delta);
                Some(rpm)
            } else {
                None
            }
        } else {
            None
        }
    }
}
