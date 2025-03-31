use embassy_stm32::{
	adc::Adc,
	mode::Blocking,
	peripherals::{ADC1, TIM1, TIM2, TIM3},
	usart::{Config as UartConfig, HalfDuplexConfig, HalfDuplexReadback, Uart},
	Peripherals,
};
use embassy_sync::mutex::Mutex;
use packed_struct::PackedStructSlice;

use crate::{
	buzzer::Buzzer,
	config::Config,
	fan::Fan,
	filament_sensor::FilamentSensor,
	heater::Heater,
	pinda::Pinda,
	rotary_button::RotaryButton,
	rotary_encoder::RotaryEncoder,
	thermistor::Thermistor,
	tmc2209::{registers::TMC2209Register, TMC2209Error, TMC2209},
	BuddyMutex,
};

// ######################
// A set of statics that represent the different
// features of the board.
// ######################

static STEPPERS: Steppers = Steppers {
	x: Mutex::new(None),
	y: Mutex::new(None),
	z: Mutex::new(None),
	e: Mutex::new(None),
};

static HEATERS: Heaters = Heaters {
	hotend: Mutex::new(None),
	bed: Mutex::new(None),
};

static THERMISTORS: Thermistors = Thermistors {
	hotend: Mutex::new(None),
	bed: Mutex::new(None),
	board: Mutex::new(None),
};

static ROTARY: Rotary = Rotary {
	button: Mutex::new(None),
	encoder: Mutex::new(None),
};

pub(crate) static BOARD: Board = Board {
	adc1: Mutex::new(None),
	filament_sensor: Mutex::new(None),
	pinda: Mutex::new(None),
	fan_0: Mutex::new(None),
	fan_1: Mutex::new(None),
	stepper_uart: Mutex::new(None),
	heaters: &HEATERS,
	steppers: &STEPPERS,
	thermistors: &THERMISTORS,
	rotary: &ROTARY,
	buzzer: Mutex::new(None),
};

pub struct Steppers {
	pub x: BuddyMutex<TMC2209>,
	pub y: BuddyMutex<TMC2209>,
	pub z: BuddyMutex<TMC2209>,
	pub e: BuddyMutex<TMC2209>,
}

pub struct Thermistors {
	pub hotend: BuddyMutex<Thermistor>,
	pub bed: BuddyMutex<Thermistor>,
	pub board: BuddyMutex<Thermistor>,
}

pub struct Heaters {
	pub hotend: BuddyMutex<Heater<TIM3>>,
	pub bed: BuddyMutex<Heater<TIM3>>,
}

pub struct Rotary {
	pub encoder: BuddyMutex<RotaryEncoder>,
	pub button: BuddyMutex<RotaryButton>,
}

/// The representation of the board and the features it contains.
/// The struct instantiates the objects into statics that can
/// later be accessed in embassy tasks.
pub struct Board {
	pub adc1: BuddyMutex<Adc<'static, ADC1>>,
	pub filament_sensor: BuddyMutex<FilamentSensor>,
	pub pinda: BuddyMutex<Pinda>,
	pub fan_0: BuddyMutex<Fan<TIM1>>,
	pub fan_1: BuddyMutex<Fan<TIM1>>,
	pub stepper_uart: BuddyMutex<Uart<'static, Blocking>>,
	pub heaters: &'static Heaters,
	pub steppers: &'static Steppers,
	pub thermistors: &'static Thermistors,
	pub rotary: &'static Rotary,
	pub buzzer: BuddyMutex<Buzzer<TIM2>>,
}

impl Board {
	/// Initialising the board with the Prusa MINI default
	/// configuration.
	pub async fn default_mini(p: Peripherals) -> &'static Self {
		let c = Config::mini(p);
		Board::init(c).await
	}

	pub async fn init(c: Config) -> &'static Self {
		*(BOARD.buzzer.lock().await) = Some(Buzzer::init(c.buzzer));
		*(BOARD.adc1.lock().await) = Some(Adc::new(c.adc1));
		*(BOARD.thermistors.bed.lock().await) = Some(Thermistor::init(c.thermistors.bed));
		*(BOARD.thermistors.board.lock().await) = Some(Thermistor::init(c.thermistors.board));
		*(BOARD.thermistors.hotend.lock().await) = Some(Thermistor::init(c.thermistors.hotend));
		*(BOARD.filament_sensor.lock().await) = Some(FilamentSensor::init(c.filament_sensor));
		*(BOARD.pinda.lock().await) = Some(Pinda::init(c.pinda));
		*(BOARD.rotary.button.lock().await) = Some(RotaryButton::init(c.rotary.button));
		*(BOARD.rotary.encoder.lock().await) = Some(RotaryEncoder::init(c.rotary.encoder));
		*(BOARD.fan_0.lock().await) = Some(Fan::init(c.fans.fan_0));
		*(BOARD.fan_1.lock().await) = Some(Fan::init(c.fans.fan_1));
		*(BOARD.heaters.bed.lock().await) = Some(Heater::init(c.heaters.bed));
		*(BOARD.heaters.hotend.lock().await) = Some(Heater::init(c.heaters.hotend));

		// Stepper Uart
		let uart_config = UartConfig::default();
		let uart = Uart::new_blocking_half_duplex(
			c.stepper_uart.uart,
			c.stepper_uart.tx,
			uart_config,
			HalfDuplexReadback::NoReadback,
			HalfDuplexConfig::PushPull,
		)
		.unwrap();
		uart.set_baudrate(115_200).unwrap();
		*(BOARD.stepper_uart.lock().await) = Some(uart);

		// Steppers
		let mut x = TMC2209::init(c.steppers.x).await.unwrap();
		Board::configure_stepper(&mut x).await.unwrap();
		*(BOARD.steppers.x.lock().await) = Some(x);

		let mut y = TMC2209::init(c.steppers.y).await.unwrap();
		Board::configure_stepper(&mut y).await.unwrap();
		*(BOARD.steppers.y.lock().await) = Some(y);

		let mut z = TMC2209::init(c.steppers.z).await.unwrap();
		Board::configure_stepper(&mut z).await.unwrap();
		*(BOARD.steppers.z.lock().await) = Some(z);

		let mut e = TMC2209::init(c.steppers.e).await.unwrap();
		Board::configure_stepper(&mut e).await.unwrap();
		*(BOARD.steppers.e.lock().await) = Some(e);

		&BOARD
	}

	async fn configure_stepper(stepper: &mut TMC2209) -> Result<(), TMC2209Error> {
		// Configuration copied from Prusa Buddy Marlin Firmware
		// TODO: Finish the configuration porting across from their config.
		// Update the GCONF register
		let mut gconf = stepper.read_gconf().await?;
		gconf.mstep_reg_select = true; // Select microsteps with UART
		gconf.pdn_disable = true; // Use UART
		gconf.i_scale_analog = false;
		gconf.en_spreadcycle = false;
		let mut payload: [u8; 4] = [0u8; 4];
		gconf.pack_to_slice(&mut payload).unwrap();
		stepper
			.write_to_register(TMC2209Register::GConf, payload)
			.await?;
		// Update the ChopConf
		let mut chop_conf = stepper.read_chop_conf().await?;
		chop_conf.tbl = 0b01; // blank_time = 24
		chop_conf.intpol = true;
		chop_conf.toff = 3;
		chop_conf.hend = 1; // -2 + 3;
		chop_conf.hstrt = 5; // 6 - 1;
		chop_conf.dedge = true;
		chop_conf.mres = 2;
		let mut payload: [u8; 4] = [0u8; 4];
		chop_conf.pack_to_slice(&mut payload).unwrap();
		stepper
			.write_to_register(TMC2209Register::ChopConf, payload)
			.await?;
		// Update the PwmConf
		let mut pwm_conf = stepper.read_pwm_conf().await.unwrap();
		pwm_conf.pwm_ilm = 12;
		pwm_conf.pwm_reg = 8;
		pwm_conf.pwm_autograd = true;
		pwm_conf.pwm_autoscale = true;
		pwm_conf.pwm_freq = 0b01;
		pwm_conf.pwm_grad = 14;
		pwm_conf.pwm_ofs = 36;
		let mut payload: [u8; 4] = [0u8; 4];
		pwm_conf.pack_to_slice(&mut payload).unwrap();
		stepper
			.write_to_register(TMC2209Register::PwmConf, payload)
			.await?;
		Ok(())
	}
}

/*
use embassy_stm32::{
	adc::{Adc, AdcChannel, AnyAdcChannel},
	exti::{AnyChannel, Channel, ExtiInput},
	gpio::{AnyPin, OutputType, Pin},
	mode::Blocking,
	peripherals::{ADC1, EXTI10, EXTI14, PB0, PB1, PD5, PE10, PE11, PE14, PE9, TIM1, TIM3, USART2},
	time::khz,
	timer::simple_pwm::{PwmPin, SimplePwm},
	usart::{Config, HalfDuplexConfig, HalfDuplexReadback, Uart},
	Peripherals,
};
use embassy_sync::mutex::Mutex;
use packed_struct::PackedStructSlice;
*/

/*
impl Board {
	pub async fn init(p: Peripherals) -> &'static Self {
		Board::init_adc1(p.ADC1).await;
		Board::init_thermistors(
			p.PA4.degrade_adc(),
			p.PA5.degrade_adc(),
			p.PC0.degrade_adc(),
		)
		.await;
		Board::init_filament_sensor(p.PB4.degrade(), p.EXTI4.degrade()).await;
		Board::init_rotary_button(p.PE12.degrade(), p.EXTI12.degrade()).await;
		Board::init_pinda(p.PA8.degrade(), p.EXTI8.degrade()).await;
		Board::init_rotary_encoder(
			p.PE13.degrade(),
			p.EXTI13.degrade(),
			p.PE15.degrade(),
			p.EXTI15.degrade(),
		)
		.await;
		Board::init_fans(p.TIM1, p.PE11, p.PE9, p.PE10, p.EXTI10, p.PE14, p.EXTI14).await;
		Board::init_heaters(p.TIM3, p.PB0, p.PB1).await;
		Board::init_uart(p.USART2, p.PD5).await;

		// More needed for the motors in terms of config.

		let mut z = TMC2209::init(
			0,
			p.PD2.degrade(),
			p.PD4.degrade(),
			p.PD15.degrade(),
			p.PE5.degrade(),
			p.EXTI3.degrade(),
			&BOARD.stepper_uart,
		)
		.await
		.unwrap();
		Board::configure_stepper(&mut z).await.unwrap();
		{
			*(BOARD.steppers.z.lock().await) = Some(z);
		}

		let mut x = TMC2209::init(
			1,
			p.PD3.degrade(),
			p.PD1.degrade(),
			p.PD0.degrade(),
			p.PE2.degrade(),
			p.EXTI2.degrade(),
			&BOARD.stepper_uart,
		)
		.await
		.unwrap();
		Board::configure_stepper(&mut x).await.unwrap();
		{
			*(BOARD.steppers.x.lock().await) = Some(x);
		}

		/*
		TODO: resolve the diag pin interrupt clash with the rotary encoder.
		let e = TMC2209::init(
			2,
			p.PD10.degrade(),
			p.PD9.degrade(),
			p.PD8.degrade(),
			p.PA15.degrade(),
			// Need to check if a different interrupt is ok.
			// p.EXTI15.degrade(),
			&BOARD.stepper_uart,
		)
		.await
		.unwrap();
		{
			*(BOARD.steppers.e.lock().await) = Some(e);
		}
		*/

		let mut y = TMC2209::init(
			3,
			p.PD14.degrade(),
			p.PD13.degrade(),
			p.PD12.degrade(),
			p.PE1.degrade(),
			p.EXTI1.degrade(),
			&BOARD.stepper_uart,
		)
		.await
		.unwrap();
		Board::configure_stepper(&mut y).await.unwrap();
		{
			*(BOARD.steppers.y.lock().await) = Some(y);
		}

		&BOARD
	}

	async fn configure_stepper(stepper: &mut TMC2209) -> Result<(), TMC2209Error> {
		// Configuration copied from Prusa Buddy Marlin Firmware
		// TODO: Finish the configuration porting across from their config.
		// Update the GCONF register
		let mut gconf = stepper.read_gconf().await?;
		gconf.mstep_reg_select = true; // Select microsteps with UART
		gconf.pdn_disable = true; // Use UART
		gconf.i_scale_analog = false;
		gconf.en_spreadcycle = false;
		let mut payload: [u8; 4] = [0u8; 4];
		gconf.pack_to_slice(&mut payload).unwrap();
		stepper
			.write_to_register(TMC2209Register::GConf, payload)
			.await?;
		// Update the ChopConf
		let mut chop_conf = stepper.read_chop_conf().await?;
		chop_conf.tbl = 0b01; // blank_time = 24
		chop_conf.intpol = true;
		chop_conf.toff = 3;
		chop_conf.hend = 1; // -2 + 3;
		chop_conf.hstrt = 5; // 6 - 1;
		chop_conf.dedge = true;
		chop_conf.mres = 2;
		let mut payload: [u8; 4] = [0u8; 4];
		chop_conf.pack_to_slice(&mut payload).unwrap();
		stepper
			.write_to_register(TMC2209Register::ChopConf, payload)
			.await?;
		// Update the PwmConf
		let mut pwm_conf = stepper.read_pwm_conf().await.unwrap();
		pwm_conf.pwm_ilm = 12;
		pwm_conf.pwm_reg = 8;
		pwm_conf.pwm_autograd = true;
		pwm_conf.pwm_autoscale = true;
		pwm_conf.pwm_freq = 0b01;
		pwm_conf.pwm_grad = 14;
		pwm_conf.pwm_ofs = 36;
		let mut payload: [u8; 4] = [0u8; 4];
		pwm_conf.pack_to_slice(&mut payload).unwrap();
		stepper
			.write_to_register(TMC2209Register::PwmConf, payload)
			.await?;
		Ok(())
	}

	async fn init_adc1(adc: ADC1) {
		let adc = Adc::new(adc);
		{
			*(BOARD.adc1.lock().await) = Some(adc);
		}
	}

	async fn init_uart(
		uart: USART2,
		tx: PD5,
	) {
		let uart_config = Config::default();
		let uart = Uart::new_blocking_half_duplex(
			uart,
			tx,
			uart_config,
			HalfDuplexReadback::NoReadback,
			HalfDuplexConfig::PushPull,
		)
		.unwrap();
		uart.set_baudrate(115_200).unwrap();
		{
			*(BOARD.stepper_uart.lock().await) = Some(uart);
		}
	}

	async fn init_filament_sensor(
		pin: AnyPin,
		exti: AnyChannel,
	) {
		let filament_sensor = FilamentSensor::init(pin, exti);
		{
			*(BOARD.filament_sensor.lock().await) = Some(filament_sensor);
		}
	}

	async fn init_fans(
		tim: TIM1,
		pwm_pin_0: PE11,
		pwm_pin_1: PE9,
		pin_0: PE10,
		exti_0: EXTI10,
		pin_1: PE14,
		exti_1: EXTI14,
	) {
		let fan_0_pwm_pin = PwmPin::new_ch2(pwm_pin_0, OutputType::PushPull);
		let fan_1_pwm_pin = PwmPin::new_ch1(pwm_pin_1, OutputType::PushPull);
		let pwm = SimplePwm::new(
			tim,
			Some(fan_1_pwm_pin),
			Some(fan_0_pwm_pin),
			None,
			None,
			khz(21),
			Default::default(),
		);
		// EXTI Inputs
		let fan_0_inp = ExtiInput::new(pin_0, exti_0, embassy_stm32::gpio::Pull::None);
		let fan_1_inp = ExtiInput::new(pin_1, exti_1, embassy_stm32::gpio::Pull::None);

		let channels = pwm.split();
		let fan_0 = Fan::init(channels.ch2, fan_0_inp).await;
		let fan_1 = Fan::init(channels.ch1, fan_1_inp).await;
		{
			*(BOARD.fan_0.lock().await) = Some(fan_0);
			*(BOARD.fan_1.lock().await) = Some(fan_1);
		}
	}

	async fn init_heaters(
		tim: TIM3,
		bed_pin: PB0,
		hotend_pin: PB1,
	) {
		let bed = PwmPin::new_ch3(bed_pin, OutputType::PushPull);
		let hotend = PwmPin::new_ch4(hotend_pin, OutputType::PushPull);
		let pwm = SimplePwm::new(
			tim,
			None,
			None,
			Some(bed),
			Some(hotend),
			khz(21),
			Default::default(),
		);

		let channels = pwm.split();
		let bed = Heater::init(channels.ch3);
		let hotend = Heater::init(channels.ch4);
		{
			*(BOARD.heaters.bed.lock().await) = Some(bed);
			*(BOARD.heaters.hotend.lock().await) = Some(hotend);
		}
	}

	async fn init_rotary_button(
		pin: AnyPin,
		exti: AnyChannel,
	) {
		let rotary_button = RotaryButton::init(pin, exti).await;
		{
			*(BOARD.rotary.button.lock().await) = Some(rotary_button);
		}
	}

	async fn init_pinda(
		pin: AnyPin,
		exti: AnyChannel,
	) {
		let pinda = Pinda::init(pin, exti).await;
		{
			*(BOARD.pinda.lock().await) = Some(pinda);
		}
	}

	async fn init_rotary_encoder(
		pin_a: AnyPin,
		exti_a: AnyChannel,
		pin_b: AnyPin,
		exti_b: AnyChannel,
	) {
		let encoder = RotaryEncoder::init(pin_a, exti_a, pin_b, exti_b).await;
		{
			*(BOARD.rotary.encoder.lock().await) = Some(encoder);
		}
	}

	async fn init_thermistors(
		bed: AnyAdcChannel<ADC1>,
		board: AnyAdcChannel<ADC1>,
		hotend: AnyAdcChannel<ADC1>,
	) {
		// Bed thermistor
		let max_adc_sample = (1 << 12) - 1;
		let bed_thermistor = Thermistor::init(
			&BOARD.adc1,
			bed,
			4_700.0,
			max_adc_sample,
			4_092.0,
			100_000.0,
		);
		{
			*(BOARD.thermistors.bed.lock().await) = Some(bed_thermistor);
		}
		let board_thermistor = Thermistor::init(
			&BOARD.adc1,
			board,
			4_700.0,
			max_adc_sample,
			4_550.0,
			100_000.0,
		);
		{
			*(BOARD.thermistors.board.lock().await) = Some(board_thermistor);
		}
		let hotend_thermistor = Thermistor::init(
			&BOARD.adc1,
			hotend,
			4_700.0,
			max_adc_sample,
			4_267.0,
			100_000.0,
		);
		{
			*(BOARD.thermistors.hotend.lock().await) = Some(hotend_thermistor);
		}
	}
}

*/
