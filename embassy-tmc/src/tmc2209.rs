use core::convert::Infallible;

use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    mutex::{Mutex, TryLockError},
};
use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
use embedded_hal_async::digital::Wait;
use embedded_io_async::{Read, Write};
use packed_struct::{
    derive::PackedStruct,
    types::{
        Integer,
        bits::{self, Bits},
    },
};

use crate::{datagram::Datagram, direction::Direction, errors::TMCError};

/// A TMC2209 driver struct with no uart connection. Simply driven by setting the pins.
pub struct TMC2209Minimal<
    O: OutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible>,
    I: InputPin<Error = Infallible> + Wait<Error = Infallible>,
    R: RawMutex,
> {
    en: Mutex<R, O>,
    step: Mutex<R, O>,
    dir: Mutex<R, O>,
    dia: Mutex<R, I>,
}

impl<
    O: OutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible>,
    I: InputPin<Error = Infallible> + Wait<Error = Infallible>,
    R: RawMutex,
> TMC2209Minimal<O, I, R>
{
    /// Create a new instance of the TMC2209Minimal driver.
    pub fn new(
        en: O,
        step: O,
        dir: O,
        dia: I,
    ) -> Self {
        Self {
            en: Mutex::new(en),
            step: Mutex::new(step),
            dir: Mutex::new(dir),
            dia: Mutex::new(dia),
        }
    }

    /// Enables (powers stage on) the stepper motor by setting Pin 2 low ([TMC2209 Datasheet Page 9][https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.09.pdf]).
    pub async fn enable(&self) {
        let mut en = self.en.lock().await;
        en.set_low().unwrap();
    }

    /// Disables (power stage off) the stepper motor by setting Pin 2 high ([TMC2209 Datasheet Page 9][https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.09.pdf]).
    pub async fn disable(&self) {
        let mut en = self.en.lock().await;
        en.set_high().unwrap();
    }

    /// Toggles the step pin (Pin 16) to initiate a step.
    pub async fn step(&self) {
        let mut step = self.step.lock().await;
        step.toggle().unwrap();
    }

    /// Sets the direction of the motor spindle by driving Pin 19 high or low.
    pub async fn set_direction(
        &self,
        dir: Direction,
    ) {
        let mut d = self.dir.lock().await;
        match dir {
            Direction::Clockwise => d.set_high().unwrap(),
            Direction::CounterClockwise => d.set_low().unwrap(),
        }
    }

    /// Returns the current direction setting of the motor.
    pub async fn get_direction(&mut self) -> Direction {
        let mut dir = self.dir.lock().await;
        match dir.is_set_high().unwrap() {
            true => Direction::Clockwise,
            false => Direction::CounterClockwise,
        }
    }

    /// Report if the stepper motor has errored immediately retruning if the lock cannot be attained.
    pub async fn try_has_errored(&self) -> Result<bool, TryLockError> {
        let mut dia = self.dia.try_lock()?;
        Ok(dia.is_high().unwrap())
    }

    /// Report is the stepper motor has errored waiting for the lock to be attained.
    pub async fn has_errored(&self) -> bool {
        let mut dia = self.dia.lock().await;
        dia.is_high().unwrap()
    }

    /// This function will try and hold the lock for the dia pin immediately erroring if unable to attain it. Use with a embassy-futures select statement and another future that will exit on the successful completion of an event that is using the motor. If that select finishes then this future will drop relinquishing the lock.
    pub async fn on_error(&self) -> Result<(), TryLockError> {
        let mut dia = self.dia.try_lock()?;
        dia.wait_for_high().await.unwrap();
        Ok(())
    }
}

/// A TMC2209 driver struct with an async uart connection
/// to set-up and manage the driver. The connection is wrapped in
/// a Mutex as there could be up to four drivers on the same
/// connection and you may want to have four instances of TMC2209
/// to manage each one. This has been tested alongside the embassy-buddy
/// create where I needed to connect to the four motors controlling the
/// 3D printer axes and extruder.
///
/// During testing, I had to use the stm32 BufferedUart instance set into
/// `HalfDuplexReadback::Readback` mode so the `read_register` function expects
/// this to be set.
pub struct TMC2209AsyncUart<
    'a,
    // Output Pin
    O: OutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible>,
    // Input Interruptable Pin
    I, // InputPin<Error = Infallible>| Wait<Error = Infallible>,
    // RawMutex
    R: RawMutex,
    // Uart
    U: Read + Write,
> {
    en: O,
    step: O,
    dir: O,
    dia: I,
    addr: u8,
    usart: &'a Mutex<R, U>,
}

impl<
    O: OutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible>,
    I,
    R: RawMutex,
    U: Read + Write,
> TMC2209AsyncUart<'_, O, I, R, U>
{
    /// Enables (powers stage on) the stepper motor by setting Pin 2 low ([TMC2209 Datasheet Page 9][https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.09.pdf]).
    pub fn enable(&mut self) {
        self.en.set_low().unwrap();
    }

    /// Disables (power stage off) the stepper motor by setting Pin 2 high ([TMC2209 Datasheet Page 9][https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.09.pdf]).
    pub fn disable(&mut self) {
        self.en.set_high().unwrap();
    }

    /// Toggles the step pin (Pin 16) to initiate a step.
    pub fn step(&mut self) {
        self.step.toggle().unwrap();
    }

    /// Sets the direction of the motor spindle by driving Pin 19 high or low.
    pub fn set_direction(
        &mut self,
        dir: Direction,
    ) {
        match dir {
            Direction::Clockwise => self.dir.set_high().unwrap(),
            Direction::CounterClockwise => self.dir.set_low().unwrap(),
        }
    }

    /// Returns the current direction setting of the motor.
    pub fn get_direction(&mut self) -> Direction {
        match self.dir.is_set_high().unwrap() {
            true => Direction::Clockwise,
            false => Direction::CounterClockwise,
        }
    }
}

impl<
    'a,
    O: OutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible>,
    I: InputPin<Error = Infallible>,
    R: RawMutex,
    U: Read + Write,
> TMC2209AsyncUart<'a, O, I, R, U>
{
    /// Create new instance of TMC2209AsyncUart.
    pub fn new_with_input(
        en: O,
        step: O,
        dir: O,
        dia: I,
        addr: u8,
        usart: &'a Mutex<R, U>,
    ) -> Result<Self, TMCError> {
        if addr > 3 {
            return Err(TMCError::InvalidMotorAddress(addr));
        }
        Ok(Self {
            en,
            step,
            dir,
            dia,
            addr,
            usart,
        })
    }

    /// Check the diagnostic pin (Pin 11) to see if an error has occurred.
    pub fn has_errored(&mut self) -> bool {
        self.dia.is_high().unwrap()
    }
}

impl<
    'a,
    O: OutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible>,
    I: Wait<Error = Infallible>,
    R: RawMutex,
    U: Read + Write,
> TMC2209AsyncUart<'a, O, I, R, U>
{
    pub fn new_with_interrupt(
        en: O,
        step: O,
        dir: O,
        dia: I,
        addr: u8,
        usart: &'a Mutex<R, U>,
    ) -> Result<Self, TMCError> {
        if addr > 3 {
            return Err(TMCError::InvalidMotorAddress(addr));
        }
        Ok(Self {
            en,
            step,
            dir,
            dia,
            addr,
            usart,
        })
    }

    /// An async interrupt waiting if the diagnostic pin goes high.
    pub async fn wait_for_error(&mut self) {
        self.dia.wait_for_high().await.unwrap()
    }
}

impl<
    O: OutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible>,
    I,
    R: RawMutex,
    U: Read + Write,
> TMC2209AsyncUart<'_, O, I, R, U>
{
    /// Reads a register from the driver. Pass an instance of the register
    /// that you wish to populate with the actual settings from the
    /// driver.
    pub async fn read_register(
        &mut self,
        register: &mut impl Datagram,
    ) -> Result<(), TMCError> {
        // Generate the read request from the register
        // that implements the Datagram trait.
        let datagram = register.read_request(self.addr)?;
        info!("Sending: {}", datagram);

        info!("Locking usart");
        // Get exclusive access to the uart.
        let mut usart = self.usart.lock().await;

        info!("Writing data");
        if usart.write(datagram.as_slice()).await.is_err() {
            return Err(TMCError::UsartError);
        }

        info!("Write Complete");
        let mut msg = [0u8; 16];
        // Expects the uart to be set into readback mode so it will
        // return both the request and response. Thus, we need to
        // know the length of what we sent so we can pull out the response
        // from the array.
        let start = datagram.len();
        let end: usize = match start {
            4 => 12, // Read Requests
            8 => 16, // Write Requests
            // Should not occur as we should have generated a
            // valid datagram from the impl.
            _ => return Err(TMCError::DatagramLength(start)),
        };

        // Async read bytes as they are returned.
        usart.read_exact(&mut msg[..end]).await.unwrap();
        info!("Bytes Received: {}", msg[..end]);

        info!("Updating Register");
        register.update(&msg[start..end])?;

        Ok(())
    }

    /// Write to a register on the driver with some updated
    /// config params.
    pub async fn write_register(
        &mut self,
        register: &mut impl Datagram,
    ) -> Result<(), TMCError> {
        // Check what the request count is.
        let ifcnt_before = self.read_ifcnt().await?;

        // Create the datagram
        let datagram = register.write_request(self.addr)?;

        // Perform the write and make sure release the lock
        // here so we can access the uart again when we
        // want to read the new ifcnt.
        {
            let mut uart = self.usart.lock().await;
            if uart.write(datagram.as_slice()).await.is_err() {
                return Err(TMCError::UsartError);
            }
        }

        let ifcnt_after = self.read_ifcnt().await?;

        // The ifcnt wraps if it goes over `u8::MAX` so we need to
        // check if it is either greater than the previous value
        // and if the previous value is `u8::MAX` then check if the
        // count has wrapped.
        if ifcnt_after.cnt > ifcnt_before.cnt
            || (ifcnt_before.cnt == u8::MAX && ifcnt_after.cnt == 0)
        {
            Ok(())
        } else {
            Err(TMCError::UsartError)
        }
    }

    pub async fn read_ifcnt(&mut self) -> Result<IfCnt, TMCError> {
        let mut reg = IfCnt::default();
        self.read_register(&mut reg).await?;
        Ok(reg)
    }

    pub async fn read_ioin(&mut self) -> Result<Ioin, TMCError> {
        let mut reg = Ioin::default();
        self.read_register(&mut reg).await?;
        Ok(reg)
    }

    pub async fn read_gconf(&mut self) -> Result<Gconf, TMCError> {
        let mut reg = Gconf::default();
        self.read_register(&mut reg).await?;
        Ok(reg)
    }

    pub async fn read_gstat(&mut self) -> Result<GStat, TMCError> {
        let mut reg = GStat::default();
        self.read_register(&mut reg).await?;
        Ok(reg)
    }

    pub async fn read_nodeconf(&mut self) -> Result<NodeConf, TMCError> {
        let mut reg = NodeConf::default();
        self.read_register(&mut reg).await?;
        Ok(reg)
    }

    pub async fn read_iholdirun(&mut self) -> Result<IHoldIRun, TMCError> {
        let mut reg = IHoldIRun::default();
        self.read_register(&mut reg).await?;
        Ok(reg)
    }

    pub async fn read_tpowerdown(&mut self) -> Result<TPowerDown, TMCError> {
        let mut reg = TPowerDown::default();
        self.read_register(&mut reg).await?;
        Ok(reg)
    }

    pub async fn read_tstep(&mut self) -> Result<TStep, TMCError> {
        let mut reg = TStep::default();
        self.read_register(&mut reg).await?;
        Ok(reg)
    }

    pub async fn read_tpwmthrs(&mut self) -> Result<TpwmThrs, TMCError> {
        let mut reg = TpwmThrs::default();
        self.read_register(&mut reg).await?;
        Ok(reg)
    }

    pub async fn read_vactual(&mut self) -> Result<VActual, TMCError> {
        let mut reg = VActual::default();
        self.read_register(&mut reg).await?;
        Ok(reg)
    }

    pub async fn read_chopconf(&mut self) -> Result<ChopConf, TMCError> {
        let mut reg = ChopConf::default();
        self.read_register(&mut reg).await?;
        Ok(reg)
    }

    pub async fn read_pwmconf(&mut self) -> Result<PwmConf, TMCError> {
        let mut reg = PwmConf::default();
        self.read_register(&mut reg).await?;
        Ok(reg)
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct IfCnt {
    #[packed_field(bytes = "0")]
    pub cnt: u8,
}

impl Datagram for IfCnt {
    fn read_register(&self) -> u8 {
        0x02
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct Ioin {
    /// (From the docs p.9) Enable not input. The power stage becomes
    /// switched off (all motor outputs floating) when this pin becomes
    /// driven to a high level.
    #[packed_field(bits = "0")]
    pub enn: bool,
    #[packed_field(bits = "1")]
    pub zero1: Integer<u8, bits::Bits<1>>,
    #[packed_field(bits = "2")]
    pub ms1: bool,
    #[packed_field(bits = "3")]
    pub ms2: bool,
    #[packed_field(bits = "4")]
    pub diag: bool,
    #[packed_field(bits = "5")]
    pub zero2: Integer<u8, bits::Bits<1>>,
    #[packed_field(bits = "6")]
    pub pdn_uart: bool,
    #[packed_field(bits = "7")]
    pub step: bool,
    #[packed_field(bits = "8")]
    pub spread_en: bool,
    #[packed_field(bits = "9")]
    pub dir: bool,
    #[packed_field(bytes = "3")]
    pub version: u8,
}

impl Datagram for Ioin {
    fn read_register(&self) -> u8 {
        0x06
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct Gconf {
    #[packed_field(bits = "0")]
    pub i_scale_analog: bool,
    #[packed_field(bits = "1")]
    pub internal_rsense: bool,
    #[packed_field(bits = "2")]
    pub en_spreadcycle: bool,
    #[packed_field(bits = "3")]
    pub shaft: bool,
    #[packed_field(bits = "4")]
    pub index_otpw: bool,
    #[packed_field(bits = "5")]
    pub index_step: bool,
    #[packed_field(bits = "6")]
    pub pdn_disable: bool,
    #[packed_field(bits = "7")]
    pub mstep_reg_select: bool,
    #[packed_field(bits = "8")]
    pub multistep_filt: bool,
    #[packed_field(bits = "9")]
    pub test_mode: bool,
}

impl Datagram for Gconf {
    fn read_register(&self) -> u8 {
        0x00
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct GStat {
    #[packed_field(bits = "0")]
    pub reset: bool,
    #[packed_field(bits = "1")]
    pub drv_err: bool,
    #[packed_field(bits = "2")]
    pub uv_cp: bool,
}

impl Datagram for GStat {
    fn read_register(&self) -> u8 {
        0x01
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct NodeConf {
    #[packed_field(bits = "8..=11")]
    pub send_delay: u8,
}

impl Datagram for NodeConf {
    fn read_register(&self) -> u8 {
        0x03
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct IHoldIRun {
    #[packed_field(bits = "0..=4")]
    pub ihold: Integer<u8, Bits<5>>,
    #[packed_field(bits = "8..=12")]
    pub irun: Integer<u8, Bits<5>>,
    #[packed_field(bits = "16..=19")]
    pub ihold_delay: Integer<u8, Bits<4>>,
}

impl Datagram for IHoldIRun {
    fn read_register(&self) -> u8 {
        0x10
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct TPowerDown {
    #[packed_field(bytes = "0")]
    pub tpower_down: Integer<u8, Bits<8>>,
}

impl Datagram for TPowerDown {
    fn read_register(&self) -> u8 {
        0x11
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct TStep {
    #[packed_field(bits = "0..=19")]
    pub tstep: Integer<u32, Bits<20>>,
}

impl Datagram for TStep {
    fn read_register(&self) -> u8 {
        0x12
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct TpwmThrs {
    #[packed_field(bits = "0..=19")]
    pub tpwm_thrs: Integer<u32, Bits<20>>,
}

impl Datagram for TpwmThrs {
    fn read_register(&self) -> u8 {
        0x13
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct VActual {
    #[packed_field(bits = "0..=23")]
    pub vactual: Integer<i32, Bits<24>>,
}

impl VActual {
    pub fn new(v: i32) -> Self {
        VActual { vactual: v.into() }
    }
}

impl Datagram for VActual {
    fn read_register(&self) -> u8 {
        0x22
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct ChopConf {
    #[packed_field(bits = "0..=3")]
    pub toff: Integer<u8, Bits<4>>,
    #[packed_field(bits = "4..=6")]
    pub hstrt: Integer<u8, Bits<3>>,
    #[packed_field(bits = "8..=10")]
    pub hend: u8,
    #[packed_field(bits = "15..=16")]
    pub tbl: u8,
    #[packed_field(bits = "17")]
    pub vsense: bool,
    #[packed_field(bits = "24..=27")]
    pub mres: Integer<u8, Bits<4>>,
    #[packed_field(bits = "28")]
    pub intpol: bool,
    #[packed_field(bits = "29")]
    pub dedge: bool,
    #[packed_field(bits = "30")]
    pub diss2g: bool,
    #[packed_field(bits = "31")]
    pub diss2vs: bool,
}

impl Datagram for ChopConf {
    fn read_register(&self) -> u8 {
        0x6C
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct PwmConf {
    #[packed_field(bytes = "0")]
    pub pwm_ofs: u8,
    #[packed_field(bytes = "1")]
    pub pwm_grad: u8,
    #[packed_field(bits = "16..17")]
    pub pwm_freq: u8,
    #[packed_field(bits = "18")]
    pub pwm_autoscale: bool,
    #[packed_field(bits = "19")]
    pub pwm_autograd: bool,
    #[packed_field(bits = "20")]
    pub freewheel0: bool,
    #[packed_field(bits = "21")]
    pub freewheel1: bool,
    #[packed_field(bits = "24..=27")]
    pub pwm_reg: u8,
    #[packed_field(bits = "28..=31")]
    pub pwm_ilm: u8,
}

impl Datagram for PwmConf {
    fn read_register(&self) -> u8 {
        0x70
    }
}
