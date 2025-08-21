use packed_struct::PackedStructSlice;

use crate::errors::TMCError;

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
