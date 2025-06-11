use thiserror::Error;

#[derive(Debug, Error)]
pub enum TMCError {
    #[error("Invalid motor address. Expected: 0-3, Received: {0}")]
    InvalidMotorAddress(u8),
    #[error("The SYNC byte was invalid. Expected: 5, Received: {0}")]
    InvalidSyncByte(u8),
    #[error("Invalid 8-bit master address. Expected: 255, Received: {0}")]
    InvalidMasterAddress(u8),
    #[error("The CRC received and the CRC constructed do not match.")]
    CrcDoesNotMatch,
    #[error("Motor Addresses do not match. Expected: {0}, Recevied: {1}")]
    MotorAddrDoesNotMatch(u8, u8),
    #[error("Register Addresses do not match. Expected: {0}, Recevied: {1}")]
    RegisterAddrDoesNotMatch(u8, u8),
    #[error("Incorrect Datagram Length. Expected: 8, Received: {0}")]
    DatagramLength(usize),
    #[error("PackStruct Packing Error")]
    PackingError,
    #[error("PackStruct Unpacking Error")]
    UnpackingError,
    #[error("Usart Error")]
    UsartError,
}
