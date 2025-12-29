use crate::{Disparity, is_comma};
use crate::symbols::ControlChars;

avr_progmem::progmem! {
    static progmem ENCODE_8B10B_POSITIVE: [u16; 256] = crate::symbols::ENCODE_8B10B_POSITIVE;
    static progmem DECODE_8B10B_POSITIVE: [u8; 1024] = crate::symbols::DECODE_8B10B_POSITIVE;
    static progmem CONTROL_CHARS_POSITIVE: [(u8, u16); 12] = crate::symbols::CONTROL_CHARS_POSITIVE;
}

#[inline(never)]
pub fn encode_8b10b(data: u8, is_control: bool, disparity: Disparity) -> (u16, Disparity) {
    let symbol_positive = if is_control {
        CONTROL_CHARS_POSITIVE
            .iter()
            .find(|&(code, _)| code == data)
            .map(|(_, symbol)| symbol)
            .unwrap_or_else(|| ENCODE_8B10B_POSITIVE.load_at(data as usize))
    } else {
        ENCODE_8B10B_POSITIVE.load_at(data as usize)
    };

    let symbol = match disparity {
        Disparity::Positive => symbol_positive,
        Disparity::Negative => !symbol_positive & 0x3FF,
    };

    let new_disp = disparity.after_symbol(symbol);

    (symbol, new_disp)
}

/// The opposite of `encode_8b10b`
///
/// # Returns
/// This function returns an Option of the following tuple
/// - [`u8`]`: decoded byte
/// - [`bool`]`: if this is a control character
/// - [`Disparity`]`: the new disparity
pub fn decode_8b10b(symbol: u16, disparity: Disparity) -> Option<(u8, bool, Disparity)> {
    if is_comma(symbol) {
        return Some((ControlChars::K28_5 as u8, true, disparity.flip()));
    }

    // Validate that the passed value fits in a 10-bit symbol
    if symbol > 0x3FF {
        return None;
    }

    // Our lookup table only stores positive disparity
    let symbol_positive = match disparity {
        Disparity::Positive => symbol,
        Disparity::Negative => !symbol & 0x3FF,
    };

    if let Some((code, _)) = CONTROL_CHARS_POSITIVE
        .iter()
        .find(|&(_, symbol)| symbol == symbol_positive)
    {
        let new_disp = disparity.after_symbol(symbol);
        return Some((code, true, new_disp));
    }

    let decoded = DECODE_8B10B_POSITIVE.load_at(symbol_positive as usize);
    if decoded == 0xFF {
        // Symbol was not found
        None
    } else {
        let new_disp = disparity.after_symbol(symbol);
        Some((decoded, false, new_disp))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Disparity, symbols::ControlChars};
    use super::{encode_8b10b, decode_8b10b};
    use assert2::assert;

    #[test]
    fn test_encode_decode_neg() {
        for i in 0..u8::MAX {
            let s = encode_8b10b(i, false, Disparity::Negative);
            let d = decode_8b10b(s.0, Disparity::Negative);

            assert!(Some(i) == d.map(|x| x.0), "i={i}");
        }
    }

    #[test]
    fn test_encode_decode_pos() {
        for i in 0..u8::MAX {
            let s = encode_8b10b(i, false, Disparity::Positive);
            let d = decode_8b10b(s.0, Disparity::Positive);

            assert!(Some(i) == d.map(|x| x.0), "i={i}");
        }
    }

    #[test]
    fn encode_comma() {
        let sp = encode_8b10b(ControlChars::K28_5 as u8, true, Disparity::Negative);
        let sn = encode_8b10b(ControlChars::K28_5 as u8, true, Disparity::Positive);

        assert!(sp.0 == 0b0011111010);
        assert!(sn.0 == 0b1100000101);
    }
}