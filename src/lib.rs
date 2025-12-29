#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![cfg_attr(not(test), no_std)]
#![warn(clippy::all, clippy::cargo)]

pub mod ser;
pub mod symbols;

#[cfg(feature = "avr-progmem")]
pub mod avr_progmem;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
pub enum Disparity {
    Negative = 0,
    Positive = 1,
}

impl Disparity {
    pub const fn flip(self) -> Self {
        match self {
            Self::Negative => Disparity::Positive,
            Self::Positive => Disparity::Negative,
        }
    }

    /// Applies the current disparity to the passed symbol
    pub const fn with_disparity(self, symbol: u16) -> u16 {
        match self {
            Self::Positive => symbol,
            Self::Negative => !symbol & 0x3FF,
        }
    }

    /// Returns the disparity after the passed symbol has been processed
    pub const fn after_symbol(self, symbol: u16) -> Self {
        let ones = symbol.count_ones();

        if ones > 5 {
            Self::Positive
        } else if ones < 5 {
            Self::Negative
        } else {
            // When equal, check sub-blocks
            let b6 = symbol & 0x3F;
            let b4 = (symbol >> 6) & 0x0F;
            let ones_6b = b6.count_ones();
            let ones_4b = b4.count_ones();

            if ones_6b > 3 || ones_4b > 2 {
                Disparity::Positive
            } else if ones_6b < 3 || ones_4b < 2 {
                Disparity::Negative
            } else {
                self
            }
        }
    }
}


#[cfg(feature = "avr-progmem")]
pub use crate::avr_progmem::decode_8b10b;
#[cfg(feature = "avr-progmem")]
pub use crate::avr_progmem::encode_8b10b;

#[cfg(not(feature = "avr-progmem"))]
pub use crate::ser::encode_8b10b_const as encode_8b10b;
#[cfg(not(feature = "avr-progmem"))]
pub use crate::ser::decode_8b10b_const as decode_8b10b;

pub use crate::ser::is_comma;
