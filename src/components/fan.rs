use embassy_time::{Duration, Instant, WithTimeout};
use embedded_hal::pwm::SetDutyCycle;
use embedded_hal_async::digital::Wait;

pub struct Fan<T, I> {
    ch: T,
    exti: I,
}

impl<T: SetDutyCycle, I: Wait> Fan<T, I> {
    pub fn new(
        ch: T,
        exti: I,
    ) -> Self {
        Self { ch, exti }
    }
}

impl<T: SetDutyCycle, I> Fan<T, I> {
    pub async fn set_duty_cycle_fully_off(&mut self) {
        self.ch.set_duty_cycle_fully_off().unwrap();
    }

    pub async fn set_duty_cycle_fully_on(&mut self) {
        self.ch.set_duty_cycle_fully_on().unwrap();
    }
}

impl<T, I: Wait> Fan<T, I> {
    /// Calculate the current speed of the fan.
    pub async fn rpm(&mut self) -> Option<f64> {
        let minute_in_millis: f64 = 1_000.0 * 60.0;
        let ok = self
            .exti
            .wait_for_any_edge()
            .with_timeout(Duration::from_millis(10))
            .await
            .is_ok();
        if ok {
            let tick_one = Instant::now().as_micros() as f64;
            let ok = self
                .exti
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
