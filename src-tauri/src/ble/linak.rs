//! Linak DPG1C BLE protocol — pure encode/decode + constants. No IO.
//!
//! All writes to the desk must use
//! [`btleplug::api::WriteType::WithoutResponse`].

use uuid::{uuid, Uuid};

pub const SERVICE_UUID: Uuid = uuid!("99fa0001-338a-1024-8a49-009c0215f78a");

/// Height + speed notify characteristic. 4 bytes: LE u16 raw (0.1 mm above
/// min), LE i16 speed.
pub const CHAR_POSITION: Uuid = uuid!("99fa0021-338a-1024-8a49-009c0215f78a");

/// Discrete command characteristic (up/down/stop/wake).
pub const CHAR_CONTROL: Uuid = uuid!("99fa0002-338a-1024-8a49-009c0215f78a");

/// Control error notifications — non-zero values are firmware error codes
/// (overcurrent, collision, out-of-range). E16 on the display ≈ the desk
/// tripped a safety limit and needs a manual reset.
pub const CHAR_CONTROL_ERROR: Uuid = uuid!("99fa0003-338a-1024-8a49-009c0215f78a");

/// Reference-input characteristic — write target height (LE u16) repeatedly
/// ~every 180 ms until speed reads zero.
pub const CHAR_REFINPUT: Uuid = uuid!("99fa0031-338a-1024-8a49-009c0215f78a");

/// DPG characteristic — used to probe capabilities and, on DPG1C controllers,
/// to set `user_id[0] = 1`. Without that handshake the desk silently ignores
/// REFERENCE_INPUT writes.
pub const CHAR_DPG: Uuid = uuid!("99fa0011-338a-1024-8a49-009c0215f78a");

/// DPG command ids (only the ones we use for the user-id dance).
pub const DPG_CMD_USER_ID: u8 = 134;

pub const CMD_UP: [u8; 2] = [0x47, 0x00];
pub const CMD_DOWN: [u8; 2] = [0x46, 0x00];
pub const CMD_STOP: [u8; 2] = [0xFF, 0x00];
pub const CMD_WAKE: [u8; 2] = [0xFE, 0x00];

// Linak desks report position as a little-endian u16 in tenths of a
// millimetre. Different controllers use different factory offsets — this
// DPG1C reports 68 cm at the mechanical floor, so the safe range is
// tighter than the protocol maximum.
pub const HEIGHT_MIN_CM: f32 = 68.0;
pub const HEIGHT_MAX_CM: f32 = 127.0;

/// Value the decoder adds to `(raw / 100)` to produce cm. Measured for this
/// desk: raw=1600 written by us ended up at physical 84 cm → base 68 cm.
pub const DECODE_BASE_CM: f32 = 68.0;

/// Encode a target height (cm) to the 2-byte little-endian payload expected by
/// [`CHAR_REFINPUT`]. Clamps silently to `[HEIGHT_MIN_CM, HEIGHT_MAX_CM]`.
///
/// The encoding uses the decoder's base (`(cm - DECODE_BASE_CM) * 100`) so
/// encode/decode are symmetric even after calibration — otherwise the desk
/// would silently overshoot targets.
pub fn encode_height_cm(cm: f32) -> [u8; 2] {
    let clamped = cm.clamp(HEIGHT_MIN_CM, HEIGHT_MAX_CM);
    let raw = (((clamped - DECODE_BASE_CM) * 100.0).round()) as u16;
    raw.to_le_bytes()
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// Raw height above minimum, in tenths of a millimetre.
    pub raw: u16,
    /// Height in centimetres, `raw/100 + 62`.
    pub cm: f32,
    /// Signed speed; zero means idle.
    pub speed: i16,
}

pub fn decode_position(bytes: &[u8]) -> Option<Position> {
    if bytes.len() < 4 {
        return None;
    }
    let raw = u16::from_le_bytes([bytes[0], bytes[1]]);
    let speed = i16::from_le_bytes([bytes[2], bytes[3]]);
    Some(Position {
        raw,
        cm: (raw as f32) / 100.0 + DECODE_BASE_CM,
        speed,
    })
}

pub fn is_desk_name(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("desk") || n.contains("linak") || n.contains("dpg") || n.contains("idasen")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_round_trip() {
        for cm in [68.0_f32, 72.0, 85.0, 100.0, 120.0, 127.0] {
            let bytes = encode_height_cm(cm);
            let raw = u16::from_le_bytes(bytes);
            let padded = [bytes[0], bytes[1], 0, 0];
            let pos = decode_position(&padded).unwrap();
            assert_eq!(pos.raw, raw);
            assert!((pos.cm - cm).abs() < 0.01, "cm {cm} -> decoded {}", pos.cm);
        }
    }

    #[test]
    fn encode_clamps_out_of_range() {
        let lo = u16::from_le_bytes(encode_height_cm(50.0));
        assert_eq!(lo, 0); // clamped to 68 cm -> (68-68)*100
        let hi = u16::from_le_bytes(encode_height_cm(200.0));
        assert_eq!(hi, 5900); // 127 cm -> (127-68)*100
    }

    #[test]
    fn decode_short_buffer_is_none() {
        assert!(decode_position(&[0; 3]).is_none());
    }

    #[test]
    fn decode_speed_sign() {
        // raw=100 (63 cm), speed = -200
        let bytes = [100_u16.to_le_bytes(), (-200_i16).to_le_bytes()].concat();
        let pos = decode_position(&bytes).unwrap();
        assert_eq!(pos.speed, -200);
    }

    #[test]
    fn name_matcher() {
        assert!(is_desk_name("Desk 1234"));
        assert!(is_desk_name("LINAK"));
        assert!(is_desk_name("dpg1c-f00"));
        assert!(is_desk_name("IDASEN"));
        assert!(!is_desk_name("iPhone"));
    }
}
