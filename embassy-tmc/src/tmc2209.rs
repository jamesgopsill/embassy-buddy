use core::{convert::Infallible, ops::DerefMut};

use defmt::Format;
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
    I: InputPin<Error = Infallible>,
    U,
> TMC2209<'a, R, O, I, U>
{
    pub fn new_no_usart_no_interrupt(en: O, step: O, dir: O, dia: I) -> Self {
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
    U: Read + Write,
> TMC2209<'a, R, O, I, U>
{
    pub fn new_usart_interruptable(
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
    U: Read + Write,
> TMC2209<'a, R, O, I, U>
{
    pub fn new_usart_no_interrupt(
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
    /// Enables (powers stage on) the stepper motor by setting Pin 2 low ([TMC2209 Datasheet Page 9][https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.09.pdf]).
    pub async fn enable(&self) {
        let mut en = self.en.lock().await;
        en.set_low().unwrap();
    }

    pub fn try_enable(&self) -> Result<(), TryLockError> {
        let mut en = self.en.try_lock()?;
        en.set_low().unwrap();
        Ok(())
    }

    /// Disables (power stage off) the stepper motor by setting Pin 2 high ([TMC2209 Datasheet Page 9][https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.09.pdf]).
    pub async fn disable(&self) {
        let mut en = self.en.lock().await;
        en.set_high().unwrap();
    }

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

    pub fn try_has_errored(&self) -> Result<bool, TryLockError> {
        let mut dia = self.dia.try_lock()?;
        Ok(dia.is_high().unwrap())
    }
}

impl<'a, R: RawMutex, O, I: Wait<Error = Infallible>, U> TMC2209<'a, R, O, I, U> {
    pub async fn on_error(&self) {
        let mut dia = self.dia.lock().await;
        dia.wait_for_rising_edge().await.unwrap();
    }
}

impl<'a, R: RawMutex, O, I, U: Read + Write> TMC2209<'a, R, O, I, U> {
    pub async fn write(&self, register: &mut impl Datagram) -> Result<(), TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        register.write(usart.deref_mut(), self.addr).await
    }

    pub async fn read_ifcnt(&self) -> Result<IfCnt, TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        IfCnt::read(usart.deref_mut(), self.addr).await
    }

    pub async fn read_ioin(&self) -> Result<Ioin, TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        Ioin::read(usart.deref_mut(), self.addr).await
    }

    pub async fn read_gconf(&self) -> Result<Gconf, TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        Gconf::read(usart.deref_mut(), self.addr).await
    }

    pub async fn read_gstat(&self) -> Result<GStat, TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        GStat::read(usart.deref_mut(), self.addr).await
    }

    pub async fn read_nodeconf(&self) -> Result<NodeConf, TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        NodeConf::read(usart.deref_mut(), self.addr).await
    }

    pub async fn read_iholdirun(&self) -> Result<IHoldIRun, TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        IHoldIRun::read(usart.deref_mut(), self.addr).await
    }

    pub async fn read_tpowerdown(&self) -> Result<TPowerDown, TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        TPowerDown::read(usart.deref_mut(), self.addr).await
    }

    pub async fn read_tstep(&self) -> Result<TStep, TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        TStep::read(usart.deref_mut(), self.addr).await
    }

    pub async fn read_tpwmthrs(&self) -> Result<TpwmThrs, TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        TpwmThrs::read(usart.deref_mut(), self.addr).await
    }

    pub async fn read_vactual(&self) -> Result<VActual, TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        VActual::read(usart.deref_mut(), self.addr).await
    }

    pub async fn read_chopconf(&self) -> Result<ChopConf, TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        ChopConf::read(usart.deref_mut(), self.addr).await
    }

    pub async fn read_pwmconf(&self) -> Result<PwmConf, TMCError> {
        let usart = self.usart.unwrap();
        let mut usart = usart.lock().await;
        PwmConf::read(usart.deref_mut(), self.addr).await
    }
}

#[derive(PackedStruct, Default, Format)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct IfCnt {
    #[packed_field(bytes = "0")]
    pub cnt: u8,
}

impl Datagram for IfCnt {
    fn read_reg_addr() -> u8 {
        0x02
    }
}

#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct Ioin {
    /// (From the docs p.9) Enable not input. The power stage becomes
    /// switched off (all motor outputs floating) when this pin is
    /// driven high.
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
    fn read_reg_addr() -> u8 {
        0x06
    }
}

#[derive(PackedStruct, Default, Format)]
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
    fn read_reg_addr() -> u8 {
        0x00
    }
}

#[derive(PackedStruct, Default, Format)]
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
    fn read_reg_addr() -> u8 {
        0x01
    }
}

#[derive(PackedStruct, Default, Format)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct NodeConf {
    #[packed_field(bits = "8..=11")]
    pub send_delay: u8,
}

impl Datagram for NodeConf {
    fn read_reg_addr() -> u8 {
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
    fn read_reg_addr() -> u8 {
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
    fn read_reg_addr() -> u8 {
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
    fn read_reg_addr() -> u8 {
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
    fn read_reg_addr() -> u8 {
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
    fn read_reg_addr() -> u8 {
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
    fn read_reg_addr() -> u8 {
        0x6C
    }
}

#[derive(PackedStruct, Default, Format)]
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
    fn read_reg_addr() -> u8 {
        0x70
    }
}
