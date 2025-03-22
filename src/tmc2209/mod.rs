use datagram::{TMC2209RegisterDatagram, TMC2209RequestDatagram};
use defmt::*;
use embassy_stm32::{
	gpio::{AnyPin, Input, Level, Output, Pull, Speed},
	mode::Blocking,
	usart::{Error, Uart},
};
use embassy_time::Timer;
use packed_struct::*;
use registers::{ChopConf, Gconf, IfCnt, Ioin, PwmConf, TMC2209Register};

use crate::BuddyMutex;

pub mod datagram;
pub mod registers;

pub struct TMC2209Config {
	pub address: u8,
	pub en_pin: AnyPin,
	pub step_pin: AnyPin,
	pub dir_pin: AnyPin,
	pub dia_pin: AnyPin,
	pub uart: &'static BuddyMutex<Uart<'static, Blocking>>,
}

#[derive(Debug, Format, Clone, Copy)]
pub enum Direction {
	CounterClockwise,
	Clockwise,
}

#[derive(Debug)]
pub enum TMC2209Error {
	InvalidCRC,
	NoStaticsRemaining,
	Uart(Error),
	WriteError,
	TestUartError,
}

/// Manages a uart connected TMC2209 driven stepper motor.
pub struct TMC2209 {
	address: u8,
	en_pin: Output<'static>,
	step_pin: Output<'static>,
	dir_pin: Output<'static>,
	pub dia_interrupt: Input<'static>,
	uart: &'static BuddyMutex<Uart<'static, Blocking>>,
	direction: Direction,
}

impl TMC2209 {
	/// Initialise a uart connected TMC2209 motor.
	pub(crate) async fn init(c: TMC2209Config) -> Result<Self, TMC2209Error> {
		let en_pin = Output::new(c.en_pin, Level::High, Speed::VeryHigh);
		let step_pin = Output::new(c.step_pin, Level::Low, Speed::VeryHigh);
		let dir_pin = Output::new(c.dir_pin, Level::Low, Speed::VeryHigh);
		let dia_interrupt = Input::new(c.dia_pin, Pull::None);

		let mut tmc = Self {
			address: c.address,
			en_pin,
			step_pin,
			dir_pin,
			dia_interrupt,
			uart: c.uart,
			direction: Direction::Clockwise,
		};

		tmc.test_uart().await?;

		//
		let gconf = tmc.read_gconf().await?;
		// CW: dir_pin = 0 if shaft_reg = 0
		// otherwise CW dir_pin = 1, shaft_reg = 1
		if gconf.shaft == tmc.dir_pin.is_set_high() {
			tmc.direction = Direction::Clockwise;
		} else {
			tmc.direction = Direction::CounterClockwise;
		}

		Ok(tmc)
	}

	/// Tests the uart connection
	async fn test_uart(&mut self) -> Result<(), TMC2209Error> {
		let ioin = self.read_ioin().await?;
		if ioin.zero1 != 0 || ioin.zero2 != 0 {
			return Err(TMC2209Error::TestUartError);
		}
		Ok(())
	}

	/// Read one of the TMCs registers.
	async fn read_from_register(
		&mut self,
		register: TMC2209Register,
	) -> Result<TMC2209RegisterDatagram, TMC2209Error> {
		let request_datagram = TMC2209RequestDatagram::new(self.address, register);

		let mut guard = self.uart.lock().await;
		let uart = guard.as_mut().unwrap();

		if let Err(e) = uart.blocking_flush() {
			return Err(TMC2209Error::Uart(e));
		};

		if let Err(e) = uart.blocking_write(request_datagram.as_slice()) {
			return Err(TMC2209Error::Uart(e));
		};

		let mut read_buf: [u8; 8] = [0x0; 8];
		if let Err(e) = uart.blocking_read(&mut read_buf) {
			return Err(TMC2209Error::Uart(e));
		};

		let register_datagram = TMC2209RegisterDatagram::from_reply(read_buf);
		register_datagram.is_valid()?;

		Ok(register_datagram)
	}

	/// Write to one of the TMC registers.
	pub async fn write_to_register(
		&mut self,
		register: TMC2209Register,
		msg: [u8; 4],
	) -> Result<(), TMC2209Error> {
		// Step 1. Check the ifcnt
		let ifcnt_before = self.read_ifcnt().await?;

		Timer::after_micros(1).await;

		// Step 2.

		let datagram = TMC2209RegisterDatagram::new(self.address, register, msg);

		{
			let mut guard = self.uart.lock().await;
			let uart = guard.as_mut().unwrap();
			if let Err(e) = uart.blocking_write(datagram.as_slice()) {
				return Err(TMC2209Error::Uart(e));
			};
		}

		Timer::after_micros(1).await;

		// Step 3. Check the ifcnt again (TODO: index wraparound)
		let ifcnt_after = self.read_ifcnt().await?;
		info!("After: {}", ifcnt_after.cnt);

		if ifcnt_after.cnt > ifcnt_before.cnt {
			Ok(())
		} else {
			Err(TMC2209Error::WriteError)
		}
	}

	pub async fn read_ioin(&mut self) -> Result<Ioin, TMC2209Error> {
		let datagram = self.read_from_register(TMC2209Register::Ioin).await?;
		Ok(Ioin::unpack_from_slice(&datagram.payload()).unwrap())
	}

	pub async fn read_gconf(&mut self) -> Result<Gconf, TMC2209Error> {
		let datagram = self.read_from_register(TMC2209Register::GConf).await?;
		Ok(Gconf::unpack_from_slice(&datagram.payload()).unwrap())
	}

	pub async fn read_ifcnt(&mut self) -> Result<IfCnt, TMC2209Error> {
		let datagram = self.read_from_register(TMC2209Register::IfCnt).await?;
		Ok(IfCnt::unpack_from_slice(&datagram.payload()).unwrap())
	}

	pub async fn read_chop_conf(&mut self) -> Result<ChopConf, TMC2209Error> {
		let datagram = self.read_from_register(TMC2209Register::IfCnt).await?;
		Ok(ChopConf::unpack_from_slice(&datagram.payload()).unwrap())
	}

	pub async fn read_pwm_conf(&mut self) -> Result<PwmConf, TMC2209Error> {
		let datagram = self.read_from_register(TMC2209Register::PwmConf).await?;
		Ok(PwmConf::unpack_from_slice(&datagram.payload()).unwrap())
	}

	/// (From the docs p.9) Enable not input. The power stage becomes
	/// switched off (all motor outputs floating) when this pin becomes
	/// driven to a high level.
	pub fn enable(&mut self) {
		self.en_pin.set_low();
	}

	pub fn disable(&mut self) {
		self.en_pin.set_high();
	}

	pub async fn toggle_step(&mut self) {
		self.step_pin.toggle();
	}

	/// Controls motor direction by changing the state of the dir GPIO pin.
	pub async fn set_direction(
		&mut self,
		direction: Direction,
	) -> Result<(), TMC2209Error> {
		let gconf = self.read_gconf().await?;

		self.direction = direction;

		match (gconf.shaft, direction) {
			(true, Direction::Clockwise) => self.dir_pin.set_high(),
			(true, Direction::CounterClockwise) => self.dir_pin.set_low(),
			(false, Direction::Clockwise) => self.dir_pin.set_low(),
			(false, Direction::CounterClockwise) => self.dir_pin.set_high(),
		}

		Ok(())
	}

	/// Returns the Direction of the motor shaft.
	pub fn get_direction(&mut self) -> Direction {
		self.direction
	}
}
