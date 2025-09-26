#![doc = include_str!("../../docs/eeprom.md")]
use embassy_stm32::{
    bind_interrupts,
    i2c::I2c as Stm32I2c,
    peripherals::{DMA1_CH5, DMA1_CH6, I2C1, PB8, PB9},
    time::Hertz,
};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::Mutex,
};
use embedded_hal_async::i2c::I2c;
use thiserror::Error;

bind_interrupts!(struct Irqs {
    I2C1_EV => embassy_stm32::i2c::EventInterruptHandler<I2C1>;
    I2C1_ER => embassy_stm32::i2c::ErrorInterruptHandler<I2C1>;
});

/// A convenience type to represent the EEPROM type.
pub type BuddyEeprom<'a> = St25dv<Stm32I2c<'a, embassy_stm32::mode::Async>, ThreadModeRawMutex>;

/// A convenience function to initialise the eeprom on the buddy board.
pub fn build_eeprom<'a>(
    peri: I2C1,
    scl: PB8,
    sda: PB9,
    tx_dma: DMA1_CH6,
    rx_dma: DMA1_CH5,
) -> BuddyEeprom<'a> {
    let i2c = Stm32I2c::new(
        peri,
        scl,
        sda,
        Irqs,
        tx_dma,
        rx_dma,
        Hertz(100_000),
        Default::default(),
    );
    let eeprom: BuddyEeprom<'_> = St25dv::new(i2c);
    eeprom
}

#[derive(Debug, Error)]
pub enum St25dvError {
    #[error("Read Error")]
    ReadError,
    #[error("Write Error")]
    WriteError,
    #[error(
        "The size (const N) used in the sequential write is too small to hold your payload + byte address."
    )]
    SequentialWritePayloadTooSmall,
}

/// The buddy board features a [ST25DV04KC](https://www.st.com/resource/en/datasheet/st25dv04kc.pdf) eeprom chip connected via I2C using PB8 (SCL) and PB9 (SDA) pins. This struct is an interface/driver for the EEPROM and should work with any of the ST25DVxx series of EEPROMs
pub struct St25dv<T: I2c, M: RawMutex> {
    /// Holds a Mutex for the I2C connection.
    i2c: Mutex<M, T>,
}

impl<T: I2c, M: RawMutex> St25dv<T, M> {
    /// Create a new instance of the ST25DV interface.
    pub fn new(i2c: T) -> Self {
        Self {
            i2c: Mutex::new(i2c),
        }
    }

    /// Turn on RF.
    pub async fn rf_on(&self) -> Result<(), St25dvError> {
        let ds = 0b0101_0001;
        let mut i2c = self.i2c.lock().await;
        if i2c.write(ds, &[]).await.is_err() {
            return Err(St25dvError::WriteError);
        }
        Ok(())
    }

    /// Turn off RF.
    pub async fn rf_off(&self) -> Result<(), St25dvError> {
        let ds = 0b0101_0101;
        let mut i2c = self.i2c.lock().await;
        if i2c.write(ds, &[]).await.is_err() {
            return Err(St25dvError::WriteError);
        }
        Ok(())
    }

    /// Read some memory out of the device.
    pub async fn sequential_current_read(
        &self,
        memory: Memory,
        data_out: &mut [u8],
    ) -> Result<(), St25dvError> {
        let ds = memory.device_select_code();
        let mut i2c = self.i2c.lock().await;
        if i2c.read(ds, data_out).await.is_err() {
            return Err(St25dvError::ReadError);
        }
        Ok(())
    }

    /// Read the current address out of the device.
    pub async fn current_address_read(&self, memory: Memory) -> Result<u8, St25dvError> {
        let mut data_out: [u8; 1] = [0; 1];
        self.sequential_current_read(memory, &mut data_out).await?;
        Ok(data_out[0])
    }

    /// Read the memory from a specified starting point out of the device.
    pub async fn sequential_random_read(
        &self,
        memory: Memory,
        byte_addr: u16,
        data_out: &mut [u8],
    ) -> Result<(), St25dvError> {
        let ds = memory.device_select_code();
        let mut i2c = self.i2c.lock().await;
        if i2c
            .write_read(ds, &byte_addr.to_le_bytes(), data_out)
            .await
            .is_err()
        {
            return Err(St25dvError::ReadError);
        }
        Ok(())
    }

    /// Read a single address.
    pub async fn random_address_read(
        &self,
        memory: Memory,
        byte_addr: u16,
    ) -> Result<u8, St25dvError> {
        let mut data_out: [u8; 1] = [0; 1];
        self.sequential_random_read(memory, byte_addr, &mut data_out)
            .await?;
        Ok(data_out[0])
    }

    /// N must be at least the len of the data + 2 to include the memory address. The function will shorten the slice based on the data slice so it won't write null bytes beyond the length of the data.
    pub async fn sequential_write<const N: usize>(
        &self,
        memory: Memory,
        addr: u16,
        data: &[u8],
    ) -> Result<(), St25dvError> {
        // TODO: Concatenate
        let ds = memory.device_select_code();
        let mut payload: [u8; N] = [0; N];

        if payload.len() < data.len() + 2 {
            return Err(St25dvError::SequentialWritePayloadTooSmall);
        }

        payload[0] = addr.to_le_bytes()[0];
        payload[1] = addr.to_le_bytes()[1];

        let mut n = 1;
        for d in data {
            n += 1;
            payload[n] = *d;
        }

        let mut i2c = self.i2c.lock().await;
        if i2c.write(ds, &payload).await.is_err() {
            return Err(St25dvError::WriteError);
        }
        Ok(())
    }

    /// Write a byte to a specific address.
    pub async fn byte_write(&self, memory: Memory, addr: u16, data: u8) -> Result<(), St25dvError> {
        let ds = memory.device_select_code();
        let data = [addr.to_le_bytes()[0], addr.to_le_bytes()[1], data];
        let mut i2c = self.i2c.lock().await;
        if i2c.write(ds, &data).await.is_err() {
            return Err(St25dvError::WriteError);
        }
        Ok(())
    }
}

/// A representation of the different memory sections in the EEPROM.
pub enum Memory {
    User,
    System,
}

impl Memory {
    /// Return the 7-bit device select code for accessing the respective memory. This is bit-shifted and the R/W bit appended by the I2C protocol.
    pub const fn device_select_code(&self) -> u8 {
        match &self {
            Self::User => 0b0101_0011,
            Self::System => 0b0101_0111,
        }
    }
}
