use defmt::Format;
use packed_struct::prelude::*;

const WRITE_OFFSET: u8 = 0x80;

#[allow(dead_code)]
pub enum TMC2209Register {
	GConf,
	IfCnt,
	Ioin,
	GStat,
	NodeConf,
	IHoldIRun,
	TPowerDown,
	TStep,
	TpwmThrs,
	VActual,
	ChopConf,
	PwmConf,
}

impl TMC2209Register {
	pub fn read_value(&self) -> u8 {
		match *self {
			TMC2209Register::GConf => 0x00,
			TMC2209Register::GStat => 0x01,
			TMC2209Register::IfCnt => 0x02,
			TMC2209Register::NodeConf => 0x03,
			TMC2209Register::Ioin => 0x06,
			TMC2209Register::IHoldIRun => 0x10,
			TMC2209Register::TPowerDown => 0x11,
			TMC2209Register::TStep => 0x12,
			TMC2209Register::TpwmThrs => 0x13,
			TMC2209Register::VActual => 0x22,
			TMC2209Register::ChopConf => 0x6C,
			TMC2209Register::PwmConf => 0x70,
		}
	}

	pub fn write_value(&self) -> u8 {
		self.read_value() + WRITE_OFFSET
	}
}

#[derive(PackedStruct, Format)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct Ioin {
	/// (From the docs p.9) Enable not input. The power stage becomes
	/// switched off (all motor outputs floating) when this pin becomes
	/// driven to a high level.
	#[packed_field(bits = "0")]
	pub enn: bool,
	#[packed_field(bits = "1")]
	pub zero1: u8, //Integer<u8, packed_bits::Bits<1>>,
	#[packed_field(bits = "2")]
	pub ms1: bool,
	#[packed_field(bits = "3")]
	pub ms2: bool,
	#[packed_field(bits = "4")]
	pub diag: bool,
	#[packed_field(bits = "5")]
	pub zero2: u8, // Integer<u8, packed_bits::Bits<1>>,
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

#[derive(PackedStruct, Format)]
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

#[derive(PackedStruct, Format)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct IfCnt {
	#[packed_field(bytes = "0")]
	pub cnt: u8,
}

#[derive(PackedStruct, Format)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct GStat {
	#[packed_field(bits = "0")]
	pub reset: bool,
	#[packed_field(bits = "1")]
	pub drv_err: bool,
	#[packed_field(bits = "2")]
	pub uv_cp: bool,
}

#[derive(PackedStruct, Format)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct NodeConf {
	#[packed_field(bits = "8..=11")]
	pub send_delay: u8,
}

#[derive(PackedStruct, Format)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct IHoldIRun {
	#[packed_field(bits = "0..=4")]
	pub ihold: u8, //Integer<u8, packed_bits::Bits<5>>,
	#[packed_field(bits = "8..=12")]
	pub irun: u8, //Integer<u8, packed_bits::Bits<5>>,
	#[packed_field(bits = "16..=19")]
	pub ihold_delay: u8, //Integer<u8, packed_bits::Bits<4>>,
}

#[derive(PackedStruct, Format)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct TPowerDown {
	#[packed_field(bytes = "0")]
	pub tpower_down: u8, //Integer<u8, packed_bits::Bits<8>>,
}

#[derive(PackedStruct, Format)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct TStep {
	#[packed_field(bits = "0..=19")]
	pub tstep: u32, //Integer<u32, packed_bits::Bits<20>>,
}

#[derive(PackedStruct, Format)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct TpwmThrs {
	#[packed_field(bits = "0..=19")]
	pub tpwm_thrs: u32, //Integer<u32, packed_bits::Bits<20>>,
}

#[derive(PackedStruct, Format)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct VActual {
	#[packed_field(bits = "0..=23")]
	pub vactual: i32, // Integer<i32, packed_bits::Bits<24>>,
}

#[derive(PackedStruct, Format)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0", endian = "msb")]
pub struct ChopConf {
	#[packed_field(bits = "0..=3")]
	pub toff: u8, //Integer<u8, packed_bits::Bits<4>>,
	#[packed_field(bits = "4..=6")]
	pub hstrt: u8, // Integer<u8, packed_bits::Bits<3>>,
	#[packed_field(bits = "8..=10")]
	pub hend: u8,
	#[packed_field(bits = "15..=16")]
	pub tbl: u8,
	#[packed_field(bits = "17")]
	pub vsense: bool,
	#[packed_field(bits = "24..=27")]
	pub mres: u8, // Integer<u8, packed_bits::Bits<4>>,
	#[packed_field(bits = "28")]
	pub intpol: bool,
	#[packed_field(bits = "29")]
	pub dedge: bool,
	#[packed_field(bits = "30")]
	pub diss2g: bool,
	#[packed_field(bits = "31")]
	pub diss2vs: bool,
}

#[derive(PackedStruct, Format)]
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
