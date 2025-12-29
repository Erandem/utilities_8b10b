use crate::Disparity;
use crate::symbols::{ControlChars, CONTROL_CHARS_POSITIVE, ENCODE_8B10B_POSITIVE, DECODE_8B10B_POSITIVE};

pub const LOWER_10_BITMASK: u16 = 0x3FF;

pub const fn is_comma(symbol: u16) -> bool {
    symbol == 0b0011111010 || symbol == 0b1100000101
}

#[inline(never)]
pub const fn encode_8b10b_const(word: DataWord) -> Symbol {
    let data = word.byte();
    let is_control = word.is_control();
    let disparity = word.disparity();

    let symbol_positive = if is_control {
        let mut found_control_code = None;
        let mut i = 0;
        while i < CONTROL_CHARS_POSITIVE.len() {
            let (code, symbol) = CONTROL_CHARS_POSITIVE[i];

            if code == data {
                found_control_code = Some(symbol);
                break;
            }

            i += 1;
        }

        if let Some(control_code) = found_control_code {
            control_code
        } else {
            ENCODE_8B10B_POSITIVE[data as usize]
        }
    } else {
        ENCODE_8B10B_POSITIVE[data as usize]
    };

    let symbol = disparity.with_disparity(symbol_positive);
    let new_disp = disparity.after_symbol(symbol);

    Symbol::new_unchecked(symbol, new_disp)
}

/// The opposite of `encode_8b10b`
///
/// # Returns
/// This function returns an Option of the following tuple
/// - [`u8`]`: decoded byte
/// - [`bool`]`: if this is a control character
/// - [`Disparity`]`: the new disparity
pub const fn decode_8b10b_const(encoded_symbol: Symbol) -> Option<DataWord> {
    let symbol = encoded_symbol.symbol();
    let disparity = encoded_symbol.disparity();

    if is_comma(symbol) {
        return Some(DataWord::new(ControlChars::K28_5 as u8, true, disparity.flip()));
    }

    // Validate that the passed value fits in a 10-bit symbol
    if symbol > 0x3FF {
        return None;
    }

    // Validate disparity
    let ones = symbol.count_ones();
    if ones < 4 || ones > 6 {
        return None;
    }

    // Our lookup table only stores positive disparity
    let symbol_positive = disparity.with_disparity(symbol);

    // Handle control characters seperately
    if let Some(code) = {
        let mut i = 0;
        let mut found = None;

        while i < CONTROL_CHARS_POSITIVE.len() {
            if CONTROL_CHARS_POSITIVE[i].1 == symbol_positive {
                found = Some(CONTROL_CHARS_POSITIVE[i].0);
                break;
            }

            i += 1;
        }

        found
    } {
        let new_disp = disparity.after_symbol(symbol);
        return Some(DataWord::new(code, true, new_disp));
    }

    let decoded = DECODE_8B10B_POSITIVE[symbol_positive as usize];
    if decoded == 0xFF {
        // Symbol was not found
        None
    } else {
        let new_disp = disparity.after_symbol(symbol);
        Some(DataWord::new(decoded, false, new_disp))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
pub struct DataWord {
    byte: u8,
    is_control: bool,
    disparity: Disparity,
}

impl DataWord {
    pub const fn new(byte: u8, is_control: bool, disparity: Disparity) -> Self {
        Self {
            byte, is_control, disparity
        }
    }

    pub const fn byte(&self) -> u8 {
        self.byte
    }

    pub const fn is_control(&self) -> bool {
        self.is_control
    }

    pub const fn disparity(&self) -> Disparity {
        self.disparity
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
pub struct Symbol {
    symbol: u16,
    disparity: Disparity,
}

impl Symbol {
    pub const fn new(symbol: u16, disparity: Disparity) -> Option<Self> {
        if symbol <= LOWER_10_BITMASK {
            Some(Self::new_unchecked(symbol, disparity))
        } else {
            None
        }
    }

    /// Creates a new [`Symbol`] without checking `symbol <= LOWER_10_BITMASK`
    /// 
    /// This is useful when you know the symbol is going to be valid, without
    /// having to litter your code with panics.
    pub const fn new_unchecked(symbol: u16, disparity: Disparity) -> Self {
        Self {
            symbol, disparity
        }
    }

    pub const fn symbol(&self) -> u16 {
        self.symbol
    }

    pub const fn disparity(&self) -> Disparity {
        self.disparity
    }
}