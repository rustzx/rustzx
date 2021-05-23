use crate::zx::keys::ZXKey;

pub enum SinclairKey {
    Left,
    Right,
    Up,
    Down,
    Fire,
}

pub enum SinclairJoyNum {
    Fist,
    Second,
}

pub(crate) fn sinclair_event_to_zx_key(key: SinclairKey, num: SinclairJoyNum) -> ZXKey {
    match (num, key) {
        (SinclairJoyNum::Fist, SinclairKey::Left) => ZXKey::N6,
        (SinclairJoyNum::Fist, SinclairKey::Right) => ZXKey::N7,
        (SinclairJoyNum::Fist, SinclairKey::Up) => ZXKey::N9,
        (SinclairJoyNum::Fist, SinclairKey::Down) => ZXKey::N8,
        (SinclairJoyNum::Fist, SinclairKey::Fire) => ZXKey::N0,
        (SinclairJoyNum::Second, SinclairKey::Left) => ZXKey::N1,
        (SinclairJoyNum::Second, SinclairKey::Right) => ZXKey::N2,
        (SinclairJoyNum::Second, SinclairKey::Up) => ZXKey::N4,
        (SinclairJoyNum::Second, SinclairKey::Down) => ZXKey::N2,
        (SinclairJoyNum::Second, SinclairKey::Fire) => ZXKey::N5,
    }
}
