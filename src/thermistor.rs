use embassy_stm32::{
	adc::{Adc, AnyAdcChannel},
	peripherals::ADC1,
};
use libm::log;

use crate::BuddyMutex;

pub struct ThermistorConfig {
	pub adc: &'static BuddyMutex<Adc<'static, ADC1>>,
	pub channel: AnyAdcChannel<ADC1>,
	pub pull_up_resistor: f64,
	pub max_adc_sample: u16,
	pub beta: f64,
	pub r_ref: f64,
	pub t_ref: f64,
}

pub struct Thermistor {
	adc: &'static BuddyMutex<Adc<'static, ADC1>>,
	channel: AnyAdcChannel<ADC1>,
	pull_up_resistor: f64,
	max_adc_sample: u16,
	beta: f64,
	r_ref: f64,
	t_ref: f64,
	temperature: f64,
}

impl Thermistor {
	/// Initialise a Thermistor
	pub(crate) fn init(c: ThermistorConfig) -> Self {
		Self {
			adc: c.adc,
			channel: c.channel,
			pull_up_resistor: c.pull_up_resistor,
			max_adc_sample: c.max_adc_sample,
			beta: c.beta,
			r_ref: c.r_ref,
			t_ref: c.t_ref,
			temperature: 0.0,
		}
	}

	/// Returns the last recorded temperature
	pub fn last_recorded_temperature(&self) -> f64 {
		self.temperature
	}

	/// Read the thermistor temperature. This will return the
	/// temperature and cache it for
	pub async fn read_temperature(&mut self) -> f64 {
		let sample: u16;
		{
			// Hold the mutex for only the time we need it.
			// May not need it for single-threaded applications but felt it is
			// good practice to be explicit.
			// Hold the guard.
			let mut adc_guard = self.adc.lock().await;
			// Get a mutable reference to the ADC
			let adc = adc_guard.as_mut().unwrap();
			// Take a sample from the channel the thermistor is on.
			sample = adc.blocking_read(&mut self.channel);
		}
		let v_ratio = self.max_adc_sample as f64 / sample as f64;
		let resistance = self.pull_up_resistor / (v_ratio - 1.0);
		let ln = log(resistance / self.r_ref);
		let one_over_beta = 1.0 / self.beta;
		let one_over_t0 = 1.0 / (273.15 + self.t_ref);
		let denom = (one_over_beta * ln) + one_over_t0;
		self.temperature = 1.0 / denom;
		self.temperature
	}
}
