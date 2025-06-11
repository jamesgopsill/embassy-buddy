use packed_struct::PackedStructSlice;

use crate::errors::TMCError;

const SYNC_BYTE: u8 = 0x05;
const WRITE_OFFSET: u8 = 0x80;

pub trait Datagram: PackedStructSlice {
    fn read_register(&self) -> u8;

    fn write_register(&self) -> u8 {
        self.read_register() + WRITE_OFFSET
    }

    fn update(
        &self,
        datagram: &[u8],
    ) -> Result<(), TMCError> {
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
        if datagram[2] != self.read_register() {
            return Err(TMCError::RegisterAddrDoesNotMatch(
                self.read_register(),
                datagram[2],
            ));
        }
        if Self::unpack_from_slice(&datagram[3..7]).is_err() {
            return Err(TMCError::UnpackingError);
        };
        Ok(())
    }

    fn read_request(
        &self,
        uart_addr: u8,
    ) -> Result<[u8; 4], TMCError> {
        if uart_addr > 3 {
            return Err(TMCError::InvalidMotorAddress(uart_addr));
        }
        let crc = crc8_atm(&[SYNC_BYTE, uart_addr, self.read_register()]);
        Ok([SYNC_BYTE, uart_addr, self.read_register(), crc])
    }

    fn write_request(
        &self,
        uart_addr: u8,
    ) -> Result<[u8; 8], TMCError> {
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
            self.write_register(),
            payload[0],
            payload[1],
            payload[2],
            payload[3],
        ]);
        Ok([
            SYNC_BYTE,
            uart_addr,
            self.write_register(),
            payload[0],
            payload[1],
            payload[2],
            payload[3],
            crc,
        ])
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
