use rustzx_core::{
    zx::keys::{CompoundKey, ZXKey},
    IterableEnum,
};
use rustzx_test::framework::{presets, RustZXTester};

#[test]
fn load_szx() {
    let mut t = RustZXTester::new("standard_keys", presets::settings_48k_nosound());
    t.enable_debug_port();
    t.load_szx("Overscan.szx");
}
