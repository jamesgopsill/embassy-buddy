use super::{registers::TMC2209Register, TMC2209Error};

const SYNC_BYTE: u8 = 0x05;

pub struct TMC2209RequestDatagram {
    datagram: [u8; 4],
}

impl TMC2209RequestDatagram {
    pub fn new(address: u8, register: TMC2209Register) -> Self {
        let crc = crc8_atm(&[SYNC_BYTE, address, register.read_value()]);
        Self {
            datagram: [SYNC_BYTE, address, register.read_value(), crc],
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.datagram
    }

    pub fn crc(&self) -> u8 {
        self.datagram[3]
    }
}

pub struct TMC2209RegisterDatagram {
    datagram: [u8; 8],
}

impl TMC2209RegisterDatagram {
    pub fn new(address: u8, register: TMC2209Register, payload: [u8; 4]) -> Self {
        let crc = crc8_atm(&[
            SYNC_BYTE,
            address,
            register.write_value(),
            payload[0],
            payload[1],
            payload[2],
            payload[3],
        ]);
        Self {
            datagram: [
                SYNC_BYTE,
                address,
                register.write_value(),
                payload[0],
                payload[1],
                payload[2],
                payload[3],
                crc,
            ],
        }
    }

    pub fn from_reply(datagram: [u8; 8]) -> Self {
        Self { datagram }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.datagram
    }

    pub fn crc(&self) -> u8 {
        self.datagram[7]
    }

    pub fn is_valid(&self) -> Result<(), TMC2209Error> {
        let expected_crc = self.datagram[7];
        if expected_crc == crc8_atm(&self.datagram[0..7]) {
            Ok(())
        } else {
            Err(TMC2209Error::InvalidCRC)
        }
    }

    pub fn payload(&self) -> [u8; 4] {
        [
            self.datagram[3],
            self.datagram[4],
            self.datagram[5],
            self.datagram[6],
        ]
    }
}

/// CRC8-ATM polynomial calculation following the datasheet c-code reference.
/// https://www.analog.com/media/en/technical-documentation/data-sheets/TMC2209_datasheet_rev1.09.pdf
fn crc8_atm(datagram: &[u8]) -> u8 {
    let mut crc = 0u8;
    for b in datagram {
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
