use embassy_stm32::{
    gpio::{Level, Output, Speed},
    mode::Async,
    peripherals::{DMA1_CH2, DMA1_CH5, PC10, PC11, PC12, PD7, SPI3},
    spi::{Config, Spi},
};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::Mutex,
};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;
use packed_struct::{PackedStructSlice, derive::PackedStruct};

pub type BuddyFlash<'a> = W25q64jv<ThreadModeRawMutex, Spi<'a, Async>, Output<'a>>;

pub fn init_flash<'a>(
    peri: SPI3,
    sck: PC10,
    mosi: PC12,
    miso: PC11,
    tx_dma: DMA1_CH5,
    rx_dma: DMA1_CH2,
    cs: PD7,
) -> BuddyFlash<'a> {
    let config = Config::default();
    let spi = Spi::new(peri, sck, mosi, miso, tx_dma, rx_dma, config);
    let cs = Output::new(cs, Level::High, Speed::VeryHigh);
    W25q64jv::new(spi, cs)
}

pub struct W25q64jv<M: RawMutex, T: SpiBus, O: OutputPin> {
    spi: Mutex<M, T>,
    cs: Mutex<M, O>,
}

impl<M: RawMutex, T: SpiBus, O: OutputPin> W25q64jv<M, T, O> {
    pub fn new(spi: T, cs: O) -> Self {
        Self {
            spi: Mutex::new(spi),
            cs: Mutex::new(cs),
        }
    }

    pub async fn send_instruction(&self, ins: Instruction) -> [u8; 6] {
        let mut data: [u8; 6] = [0; 6];
        let mut spi = self.spi.lock().await;
        let mut cs = self.cs.lock().await;
        let ins = &[ins.to_byte()];
        cs.set_low().unwrap();
        spi.write(ins).await.unwrap();
        spi.read(&mut data).await.unwrap();
        cs.set_high().unwrap();
        data
    }

    pub async fn read_status_register_one(&self) -> RegisterOne {
        let data = self
            .send_instruction(Instruction::ReadStatusRegister1)
            .await;
        let data = data[0];
        RegisterOne::unpack_from_slice(&[data]).unwrap()
    }

    pub async fn read_status_register_two(&self) -> RegisterTwo {
        let data = self
            .send_instruction(Instruction::ReadStatusRegister2)
            .await;
        let data = data[0];
        RegisterTwo::unpack_from_slice(&[data]).unwrap()
    }

    pub async fn read_status_register_three(&self) -> RegisterThree {
        let data = self
            .send_instruction(Instruction::ReadStatusRegister3)
            .await;
        let data = data[0];
        RegisterThree::unpack_from_slice(&[data]).unwrap()
    }
}

pub enum Instruction {
    ReadStatusRegister1,
    ReadStatusRegister2,
    ReadStatusRegister3,
}

impl Instruction {
    pub const fn to_byte(&self) -> u8 {
        match self {
            Self::ReadStatusRegister1 => 0x05,
            Self::ReadStatusRegister2 => 0x35,
            Self::ReadStatusRegister3 => 0x15,
        }
    }
}

#[derive(Debug, PackedStruct)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct RegisterOne {
    #[packed_field(bits = "0")]
    pub busy: bool,
    #[packed_field(bits = "1")]
    pub write_enable: bool,
    #[packed_field(bits = "2")]
    pub block_protect_0: bool,
    #[packed_field(bits = "3")]
    pub block_protect_1: bool,
    #[packed_field(bits = "4")]
    pub block_protect_2: bool,
    #[packed_field(bits = "5")]
    pub top_bottom_protect: bool,
    #[packed_field(bits = "6")]
    pub sector_block_protect: bool,
    #[packed_field(bits = "7")]
    pub complement_protect: bool,
}

#[derive(Debug, PackedStruct)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct RegisterTwo {
    #[packed_field(bits = "0")]
    pub status_register_lock: bool,
    #[packed_field(bits = "1")]
    pub quad_enable: bool,
    #[packed_field(bits = "2")]
    pub reserved: bool,
    #[packed_field(bits = "3")]
    pub lock_bit_1: bool,
    #[packed_field(bits = "4")]
    pub lock_bit_2: bool,
    #[packed_field(bits = "5")]
    pub lock_bit_3: bool,
    #[packed_field(bits = "6")]
    pub complement_protect: bool,
    #[packed_field(bits = "7")]
    pub suspend_status: bool,
}

#[derive(Debug, PackedStruct)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct RegisterThree {
    #[packed_field(bits = "0")]
    pub reserved_0: bool,
    #[packed_field(bits = "1")]
    pub reserved_1: bool,
    #[packed_field(bits = "2")]
    pub write_protect_selection: bool,
    #[packed_field(bits = "3")]
    pub reserved_3: bool,
    #[packed_field(bits = "4")]
    pub reserved_4: bool,
    #[packed_field(bits = "5")]
    pub output_driver_strength_0: bool,
    #[packed_field(bits = "6")]
    pub output_driver_strength_1: bool,
    #[packed_field(bits = "7")]
    pub reserved_7: bool,
}
