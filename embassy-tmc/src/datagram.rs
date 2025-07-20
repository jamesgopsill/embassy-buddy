#![allow(async_fn_in_trait)]
use embedded_io_async::{Read, Write};
use packed_struct::PackedStructSlice;

use crate::{errors::TMCError, tmc2209::IfCnt};

const SYNC_BYTE: u8 = 0x05;
const WRITE_OFFSET: u8 = 0x80;

pub trait Datagram: PackedStructSlice + Default {
    /// Return the address of the read register for the datagram.
    fn read_reg_addr() -> u8;

    /// Return the address of the write register for the datagram.
    fn write_reg_addr() -> u8 {
        Self::read_reg_addr() + WRITE_OFFSET
    }

    /// Create a read register request.
    fn read_request(motor_addr: u8) -> Result<[u8; 4], TMCError> {
        if motor_addr > 3 {
            return Err(TMCError::InvalidMotorAddress(motor_addr));
        }
        let crc = crc8_atm(&[SYNC_BYTE, motor_addr, Self::read_reg_addr()]);
        Ok([SYNC_BYTE, motor_addr, Self::read_reg_addr(), crc])
    }

    /// Perform a read and return the datagram
    async fn read<T: Read + Write>(usart: &mut T, motor_addr: u8) -> Result<Self, TMCError> {
        let datagram = Self::read_request(motor_addr)?;
        if usart.write(datagram.as_slice()).await.is_err() {
            return Err(TMCError::UsartError);
        }
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
        //info!("Bytes Received: {}", msg[..end]);
        let msg = &mut msg[..end];

        //info!("Updating Register");
        //sel.parse(&msg[start..end])?;
        Self::from_datagram(msg)
    }

    /// Write the datagram back into the register.
    async fn write<T: Read + Write>(
        &mut self,
        usart: &mut T,
        motor_addr: u8,
    ) -> Result<(), TMCError> {
        let ifcnt_before = IfCnt::read(usart, motor_addr).await?;

        let datagram = self.as_write_request(motor_addr)?;
        if usart.write(datagram.as_slice()).await.is_err() {
            return Err(TMCError::UsartError);
        }

        let ifcnt_after = IfCnt::read(usart, motor_addr).await?;

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

    /// TODO
    fn as_write_request(&self, uart_addr: u8) -> Result<[u8; 8], TMCError> {
        if uart_addr > 3 {
            return Err(TMCError::InvalidMotorAddress(uart_addr));
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
