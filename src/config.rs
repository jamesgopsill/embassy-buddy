use embassy_stm32::exti::Channel;
use embassy_stm32::gpio::{OutputType, Pin};
use embassy_stm32::peripherals::{TIM1, TIM3};
use embassy_stm32::time::khz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{adc::AdcChannel, peripherals::ADC1, Peripherals};

use crate::board::BOARD;
use crate::fan::FanConfig;
use crate::filament_sensor::FilamentSensorConfig;
use crate::heater::HeaterConfig;
use crate::pinda::PindaConfig;
use crate::rotary_button::RotaryButtonConfig;
use crate::rotary_encoder::RotaryEncoderConfig;
use crate::thermistor::ThermistorConfig;
use crate::tmc2209::TMC2209Config;

pub struct Config {
	pub adc1: ADC1,
	pub thermistors: ThermistorsConfig,
	pub pinda: PindaConfig,
	pub filament_sensor: FilamentSensorConfig,
	pub rotary: RotaryConfig,
	pub fans: FansConfig,
	pub heaters: HeatersConfig,
	pub steppers: SteppersConfig,
}

impl Config {
	pub fn mini(p: Peripherals) -> Self {
		// ### Thermistors ###
		let max_adc_sample = (1 << 12) - 1;
		let bed_thermistor_config = ThermistorConfig {
			adc: &BOARD.adc1,
			channel: p.PA4.degrade_adc(),
			pull_up_resistor: 4_700.0,
			max_adc_sample,
			beta: 4_092.0,
			r_ref: 100_000.0,
			t_ref: 25.0,
		};
		let board_thermistor_config = ThermistorConfig {
			adc: &BOARD.adc1,
			channel: p.PA5.degrade_adc(),
			pull_up_resistor: 4_700.0,
			max_adc_sample,
			beta: 4_550.0,
			r_ref: 100_000.0,
			t_ref: 25.0,
		};
		let hotend_thermistor_config = ThermistorConfig {
			adc: &BOARD.adc1,
			channel: p.PC0.degrade_adc(),
			pull_up_resistor: 4_700.0,
			max_adc_sample,
			beta: 4_267.0,
			r_ref: 100_000.0,
			t_ref: 25.0,
		};
		let thermistors = ThermistorsConfig {
			bed: bed_thermistor_config,
			board: board_thermistor_config,
			hotend: hotend_thermistor_config,
		};
		// ## PINDA ##
		let pinda = PindaConfig {
			pin: p.PA8.degrade(),
			exti: p.EXTI8.degrade(),
		};
		// ## FILAMENT SENSOR ##
		let filament_sensor = FilamentSensorConfig {
			pin: p.PB4.degrade(),
			exti: p.EXTI4.degrade(),
		};
		// ## ROTARY ##
		let rotary_button_config = RotaryButtonConfig {
			pin: p.PE12.degrade(),
			exti: p.EXTI12.degrade(),
		};
		let rotary_encoder_config = RotaryEncoderConfig {
			pin_a: p.PE13.degrade(),
			exti_a: p.EXTI13.degrade(),
			pin_b: p.PE15.degrade(),
			exti_b: p.EXTI15.degrade(),
		};
		let rotary = RotaryConfig {
			button: rotary_button_config,
			encoder: rotary_encoder_config,
		};
		// ## FANS ##
		let fan_0_pwm_pin = PwmPin::new_ch2(p.PE11, OutputType::PushPull);
		let fan_1_pwm_pin = PwmPin::new_ch1(p.PE9, OutputType::PushPull);
		let pwm = SimplePwm::new(
			p.TIM1,
			Some(fan_1_pwm_pin),
			Some(fan_0_pwm_pin),
			None,
			None,
			khz(21),
			Default::default(),
		);
		let channels = pwm.split();
		let fan_0_config = FanConfig {
			ch: channels.ch2,
			pin: p.PE10.degrade(),
			exti: p.EXTI10.degrade(),
		};
		let fan_1_config = FanConfig {
			ch: channels.ch1,
			pin: p.PE14.degrade(),
			exti: p.EXTI14.degrade(),
		};
		let fans = FansConfig {
			fan_0: fan_0_config,
			fan_1: fan_1_config,
		};
		// ## Heaters ##
		let bed_heater = PwmPin::new_ch3(p.PB0, OutputType::PushPull);
		let hotend_heater = PwmPin::new_ch4(p.PB1, OutputType::PushPull);
		let pwm = SimplePwm::new(
			p.TIM3,
			None,
			None,
			Some(bed_heater),
			Some(hotend_heater),
			khz(21),
			Default::default(),
		);
		let channels = pwm.split();
		let bed_heater_config = HeaterConfig { ch: channels.ch3 };
		let hotend_heater_config = HeaterConfig { ch: channels.ch4 };
		let heaters = HeatersConfig {
			bed: bed_heater_config,
			hotend: hotend_heater_config,
		};
		// ## Stepper Motors ##
		let stepper_x = TMC2209Config {
			address: 1,
			en_pin: p.PD3.degrade(),
			step_pin: p.PD1.degrade(),
			dir_pin: p.PD0.degrade(),
			dia_pin: p.PE2.degrade(),
			uart: &BOARD.stepper_uart,
		};
		let stepper_y = TMC2209Config {
			address: 3,
			en_pin: p.PD14.degrade(),
			step_pin: p.PD13.degrade(),
			dir_pin: p.PD12.degrade(),
			dia_pin: p.PE1.degrade(),
			uart: &BOARD.stepper_uart,
		};
		let stepper_z = TMC2209Config {
			address: 0,
			en_pin: p.PD2.degrade(),
			step_pin: p.PD4.degrade(),
			dir_pin: p.PD15.degrade(),
			dia_pin: p.PE5.degrade(),
			uart: &BOARD.stepper_uart,
		};
		let stepper_e = TMC2209Config {
			address: 2,
			en_pin: p.PD10.degrade(),
			step_pin: p.PD9.degrade(),
			dir_pin: p.PD8.degrade(),
			dia_pin: p.PA15.degrade(),
			uart: &BOARD.stepper_uart,
		};

		let steppers = SteppersConfig {
			x: stepper_x,
			y: stepper_y,
			z: stepper_z,
			e: stepper_e,
		};

		Self {
			adc1: p.ADC1,
			thermistors,
			pinda,
			filament_sensor,
			rotary,
			fans,
			heaters,
			steppers,
		}
	}
}

pub struct ThermistorsConfig {
	pub bed: ThermistorConfig,
	pub board: ThermistorConfig,
	pub hotend: ThermistorConfig,
}

pub struct RotaryConfig {
	pub button: RotaryButtonConfig,
	pub encoder: RotaryEncoderConfig,
}

pub struct FansConfig {
	pub fan_0: FanConfig<TIM1>,
	pub fan_1: FanConfig<TIM1>,
}

pub struct HeatersConfig {
	pub bed: HeaterConfig<TIM3>,
	pub hotend: HeaterConfig<TIM3>,
}

pub struct SteppersConfig {
	pub x: TMC2209Config,
	pub y: TMC2209Config,
	pub z: TMC2209Config,
	pub e: TMC2209Config,
}
