#![allow(unused)]
// Snippet from https://github.com/embassy-rs/embassy/blob/main/embassy-stm32/src/fmt.rs

#[collapse_debuginfo(yes)]
macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            ::defmt::info!($s $(, $x)*);
            #[cfg(not(any(feature="defmt")))]
            let _ = ($( & $x ),*);
        }
    };
}

#[collapse_debuginfo(yes)]
macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            ::defmt::error!($s $(, $x)*);
            #[cfg(not(any(feature="defmt")))]
            let _ = ($( & $x ),*);
        }
    };
}

pub(crate) use error;
pub(crate) use info;
