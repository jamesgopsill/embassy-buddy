use defmt::Format;
use packed_struct::{
    PackedStructSlice,
    derive::PackedStruct,
    types::{Integer, bits::Bits},
};

use crate::components::tmc::error::TMCError;

/// A byte the synchronises the communications between host and driver.
const SYNC_BYTE: u8 = 0x05;

/// The byte offset between the read and write register commands.
const WRITE_OFFSET: u8 = 0x80;

/// A trait representing the datagrams that can be written and read from the TMC2209 driver.
pub trait Datagram: PackedStructSlice + Default {
    /// Return the address of the read register for the datagram.
    fn read_reg_addr() -> u8;

    /// Return the address of the write register for the datagram.
    fn write_reg_addr() -> u8 {
        Self::read_reg_addr() + WRITE_OFFSET
    }

    /// Create a read register request.
    fn read_request(&self, addr: u8) -> Result<[u8; 4], TMCError> {
        if addr > 3 {
            return Err(TMCError::InvalidDriverAddress(addr));
        }
        let crc = crc8_atm(&[SYNC_BYTE, addr, Self::read_reg_addr()]);
        Ok([SYNC_BYTE, addr, Self::read_reg_addr(), crc])
    }

    /// Transforms a Datagram into a write request.
    fn as_write_request(&self, uart_addr: u8) -> Result<[u8; 8], TMCError> {
        if uart_addr > 3 {
            return Err(TMCError::InvalidDriverAddress(uart_addr));
        }
        let mut payload: [u8; 4] = [0u8; 4];
        if self.pack_to_slice(&mut payload).is_err() {
            return Err(TMCError::PackingError);
        };
        let crc = crc8_atm(&[
            SYNC_BYTE,
            uart_addr,
            Self::write_reg_addr(),
            payload[0],
            payload[1],
            payload[2],
            payload[3],
        ]);
        Ok([
            SYNC_BYTE,
            uart_addr,
            Self::write_reg_addr(),
            payload[0],
            payload[1],
            payload[2],
            payload[3],
            crc,
        ])
    }

    fn update(&mut self, datagram: &[u8]) -> Result<(), TMCError> {
        let new = Self::from_datagram(datagram)?;
        *self = new;
        Ok(())
    }

    /// Update this instance of the datagram by reading a &[u8]. For example, data received back from the uart.
    fn from_datagram(datagram: &[u8]) -> Result<Self, TMCError> {
        if datagram.len() != 8 {
            return Err(TMCError::DatagramLength(datagram.len()));
        }
        let crc = crc8_atm(&datagram[0..7]);
        if datagram[7] != crc {
            return Err(TMCError::CrcDoesNotMatch);
        }
        if datagram[0] != SYNC_BYTE {
            return Err(TMCError::InvalidSyncByte(datagram[0]));
        }
        // Page 19. returns OxFF. Not a motor address
        if datagram[1] != 255 {
            return Err(TMCError::InvalidMasterAddress(datagram[0]));
        }
        if datagram[2] != Self::read_reg_addr() {
            return Err(TMCError::RegisterAddrDoesNotMatch(
                Self::read_reg_addr(),
                datagram[2],
            ));
        }
        if let Ok(s) = Self::unpack_from_slice(&datagram[3..7]) {
            Ok(s)
        } else {
            Err(TMCError::UnpackingError)
        }
    }
}

/// CRC8-ATM polynomial calculation following the datasheet c-code reference.
/// https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.09.pdf
fn crc8_atm(bytes: &[u8]) -> u8 {
    let mut crc = 0u8;
    for b in bytes {
        let mut b = *b;
        for _ in 0..8 {
            if (crc >> 7) ^ (b & 0x01) != 0 {
                crc = (crc << 1) ^ 0x07;
            } else {
                crc <<= 1;
            }
            b >>= 1;
        }
    }
    crc
}

/// A struct representing the IFCNT register.
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

/// A stuct representing the IOIN register.
#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct Ioin {
    /// (From the docs p.9) Enable not input. The power stage becomes
    /// switched off (all motor outputs floating) when this pin is
    /// driven high.
    #[packed_field(bits = "0")]
    pub enn: bool,
    #[packed_field(bits = "1")]
    pub zero1: Integer<u8, Bits<1>>,
    #[packed_field(bits = "2")]
    pub ms1: bool,
    #[packed_field(bits = "3")]
    pub ms2: bool,
    #[packed_field(bits = "4")]
    pub diag: bool,
    #[packed_field(bits = "5")]
    pub zero2: Integer<u8, Bits<1>>,
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

/// A struct representing the GCONF register.
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

/// A struct representing the GSTAT register.
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

/// A struct representing the NODECONF register.
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

/// A struct representing the IHOLDIRUN register.
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

/// A struct representing the TPOWERDOWN register.
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

/// A struct representing the TSTEP register.
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

/// A struct representing the TPWMTHRS register.
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

/// A struct representing the VACTUAL register.
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

/// A struct representing the VACTUAL register.
#[derive(PackedStruct, Default)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct TCoolThrs {
    #[packed_field(bits = "0..=19")]
    pub tcoolthrs: Integer<u32, Bits<24>>,
}

impl TCoolThrs {
    pub fn new(v: u32) -> Self {
        TCoolThrs {
            tcoolthrs: v.into(),
        }
    }
}

impl Datagram for TCoolThrs {
    fn read_reg_addr() -> u8 {
        0x14
    }
}

/// A struct representing the CHOPCONF register.
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

/// A struct representing the PWMCONF register.
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
