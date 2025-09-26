#![allow(unused)]
use core::convert::Infallible;

use defmt::error;
use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    mutex::{Mutex, TryLockError},
};
use embassy_time::{Duration, WithTimeout};
use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
use embedded_hal_async::digital::Wait;
use embedded_io_async::{Read, ReadReady, Write};

use crate::fmt::info;
use crate::{
    IfCnt,
    components::tmc::{datagram::Datagram, direction::Direction, error::TMCError},
};

/// A struct that provides the API to interact with the TMC2209 driver.
pub struct TMC2209<'a, R: RawMutex, O, I, U> {
    en: Mutex<R, O>,
    step: Mutex<R, O>,
    dir: Mutex<R, O>,
    dia: Mutex<R, I>,
    addr: u8,
    usart: Option<&'a Mutex<R, U>>,
}

// New instances
impl<
    'a,
    R: RawMutex,
    O: OutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible>,
    I: InputPin<Error = Infallible> + Wait<Error = Infallible>,
    U,
> TMC2209<'a, R, O, I, U>
{
    /// Create a new driver instance with no usart and an interruptable dia pin.
    pub fn new_no_usart_interruptable(en: O, step: O, dir: O, dia: I) -> Self {
        Self {
            en: Mutex::new(en),
            step: Mutex::new(step),
            dir: Mutex::new(dir),
            dia: Mutex::new(dia),
            addr: 0,
            usart: None,
        }
    }
}

impl<
    'a,
    R: RawMutex,
    O: OutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible>,
    I: InputPin<Error = Infallible> + Wait<Error = Infallible>,
    U: Read + Write + ReadReady,
> TMC2209<'a, R, O, I, U>
{
    /// Create a new driver instance with an async usart connection and an interruptable dia pin.
    pub fn new_async_usart_interruptable(
        en: O,
        step: O,
        dir: O,
        dia: I,
        addr: u8,
        usart: &'a Mutex<R, U>,
    ) -> Result<Self, TMCError> {
        if addr > 3 {
            return Err(TMCError::InvalidDriverAddress(addr));
        }
        Ok(Self {
            en: Mutex::new(en),
            step: Mutex::new(step),
            dir: Mutex::new(dir),
            dia: Mutex::new(dia),
            addr,
            usart: Some(usart),
        })
    }
}

impl<
    'a,
    R: RawMutex,
    O: OutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible>,
    I: InputPin<Error = Infallible>,
    U: Read + Write + ReadReady,
> TMC2209<'a, R, O, I, U>
{
    /// Create a new driver instance with an async usart connection and no interruptable dia pin.
    pub fn new_async_usart_no_interrupt(
        en: O,
        step: O,
        dir: O,
        dia: I,
        addr: u8,
        usart: &'a Mutex<R, U>,
    ) -> Result<Self, TMCError> {
        if addr > 3 {
            return Err(TMCError::InvalidDriverAddress(addr));
        }
        Ok(Self {
            en: Mutex::new(en),
            step: Mutex::new(step),
            dir: Mutex::new(dir),
            dia: Mutex::new(dia),
            addr,
            usart: Some(usart),
        })
    }
}

// ###################

impl<'a, R: RawMutex, O: OutputPin<Error = Infallible>, I, U> TMC2209<'a, R, O, I, U> {
    /// Enables (powers stage on) the stepper motor by setting Pin 2 low ([TMC2209 Datasheet Page 9](https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.09.pdf)).
    pub async fn enable(&self) {
        let mut en = self.en.lock().await;
        en.set_low().unwrap();
    }

    /// Tries immediate locking of the enable pin mutex and powering on of the (powers stage on) the stepper motor by setting Pin 2 low ([TMC2209 Datasheet Page 9](https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.09.pdf)).
    pub fn try_enable(&self) -> Result<(), TryLockError> {
        let mut en = self.en.try_lock()?;
        en.set_low().unwrap();
        Ok(())
    }

    /// Disables (power stage off) the stepper motor by setting Pin 2 high ([TMC2209 Datasheet Page 9](https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.09.pdf)).
    pub async fn disable(&self) {
        let mut en = self.en.lock().await;
        en.set_high().unwrap();
    }

    ///  Tries immediate locking of the enable pin mutex and disabling (power stage off) the stepper motor by setting Pin 2 high ([TMC2209 Datasheet Page 9](https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.09.pdf)).
    pub fn try_disable(&self) -> Result<(), TryLockError> {
        let mut en = self.en.try_lock()?;
        en.set_high().unwrap();
        Ok(())
    }

    /// Sets the direction of the motor spindle by driving Pin 19 high or low.
    pub async fn set_direction(&self, dir: Direction) {
        let mut d = self.dir.lock().await;
        match dir {
            Direction::Clockwise => d.set_high().unwrap(),
            Direction::CounterClockwise => d.set_low().unwrap(),
        }
    }

    /// Tries immediate locking of the dir mutex so a direction change can be made.
    pub fn try_set_direction(&self, dir: Direction) -> Result<(), TryLockError> {
        let mut d = self.dir.try_lock()?;
        match dir {
            Direction::Clockwise => d.set_high().unwrap(),
            Direction::CounterClockwise => d.set_low().unwrap(),
        }
        Ok(())
    }
}

impl<'a, R: RawMutex, O: StatefulOutputPin<Error = Infallible>, I, U> TMC2209<'a, R, O, I, U> {
    /// Toggles the step pin (Pin 16) to initiate a step.
    pub async fn step(&self) {
        let mut step = self.step.lock().await;
        step.toggle().unwrap();
    }

    /// Tries an immediate step.
    pub fn try_step(&self) -> Result<(), TryLockError> {
        let mut step = self.step.try_lock()?;
        step.toggle().unwrap();
        Ok(())
    }

    /// Returns the current direction setting of the motor.
    pub async fn get_direction(&self) -> Direction {
        let mut dir = self.dir.lock().await;
        match dir.is_set_high().unwrap() {
            true => Direction::Clockwise,
            false => Direction::CounterClockwise,
        }
    }

    /// Tries to get the direction by expecting to lock the mutex immediately.
    pub fn try_get_direction(&self) -> Result<Direction, TryLockError> {
        let mut dir = self.dir.try_lock()?;
        match dir.is_set_high().unwrap() {
            true => Ok(Direction::Clockwise),
            false => Ok(Direction::CounterClockwise),
        }
    }
}

impl<'a, R: RawMutex, O, I: InputPin<Error = Infallible>, U> TMC2209<'a, R, O, I, U> {
    /// Check the diagnostic pin (Pin 11) to see if an error has occurred.
    pub async fn has_errored(&self) -> bool {
        let mut dia = self.dia.lock().await;
        dia.is_high().unwrap()
    }

    /// Tries to lock and read the dia pin.
    pub fn try_has_errored(&self) -> Result<bool, TryLockError> {
        let mut dia = self.dia.try_lock()?;
        Ok(dia.is_high().unwrap())
    }
}

impl<'a, R: RawMutex, O, I: Wait<Error = Infallible>, U> TMC2209<'a, R, O, I, U> {
    /// Enables an aync wait for an error on the dia pin.
    pub async fn on_error(&self) {
        let mut dia = self.dia.lock().await;
        dia.wait_for_rising_edge().await.unwrap();
    }
}

impl<'a, R: RawMutex, O, I, U: Read + Write + ReadReady> TMC2209<'a, R, O, I, U> {
    /// Performs a read request on the given driver usart and address. Expects ReadBack mode on the usart.
    pub async fn read_register(&self, register: &mut impl Datagram) -> Result<(), TMCError> {
        let datagram = register.read_request(self.addr)?;
        info!("[TMC2209] Read Request: {}", datagram);
        let mut usart = self.usart.unwrap().lock().await;
        if usart.write_all(datagram.as_slice()).await.is_err() {
            return Err(TMCError::UsartError);
        }

        // Expect readback mode so 12 bytes (4 read request + 8 response).
        let mut buf: [u8; 12] = [0u8; 12];
        if usart
            .read_exact(&mut buf)
            .with_timeout(Duration::from_secs(1))
            .await
            .is_err()
        {
            error!("[TMC2209] Reading Timed Out: {}", buf);
            return Err(TMCError::Timeout);
        };
        info!("[TMC2209] Read Request + Response: {}", buf);
        let msg = &buf[4..];
        register.update(msg)?;
        Ok(())
    }

    /// Writes a register to the TMC2209
    pub async fn write_register(&self, register: &mut impl Datagram) -> Result<(), TMCError> {
        let mut ifcnt_before = IfCnt::default();
        self.read_register(&mut ifcnt_before).await?;

        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;

        let datagram_1 = register.as_write_request(self.addr)?;
        let datagram_2 = IfCnt::default().read_request(self.addr)?;
        info!("[TMC2209] Write Request: {:?}", datagram_1);

        if usart.write_all(datagram_1.as_slice()).await.is_err() {
            return Err(TMCError::UsartError);
        }

        // Check it was successful
        if usart.write_all(datagram_2.as_slice()).await.is_err() {
            return Err(TMCError::UsartError);
        }

        // Buffer is 8 write request + 4 read request + 8 response.
        let mut buf: [u8; 20] = [0u8; 20];
        if usart
            .read_exact(&mut buf)
            .with_timeout(Duration::from_secs(1))
            .await
            .is_err()
        {
            error!("[TMC2209] Reading Timed Out: {}", buf);
            return Err(TMCError::Timeout);
        };
        info!("[TMC2209] Write + IfCnt + Response: {}", buf);
        let msg = &buf[12..];
        let ifcnt_after = IfCnt::from_datagram(msg)?;

        // The ifcnt wraps if it goes over `u8::MAX` so we need to
        // check if it is either greater than the previous value
        // and if the previous value is `u8::MAX` then check if the
        // count has wrapped.
        if ifcnt_after.cnt > ifcnt_before.cnt
            || (ifcnt_before.cnt == u8::MAX && ifcnt_after.cnt == 0)
        {
            Ok(())
        } else {
            Err(TMCError::WriteError(ifcnt_before.cnt, ifcnt_after.cnt))
        }
    }
}
