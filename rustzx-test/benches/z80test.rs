use std::time::Duration;

use rustzx_core::zx::keys::ZXKey;
use rustzx_test::framework::{presets, RustZXTester};

fn z80test_setup() -> RustZXTester {
    let settings = presets::settings_48k();
    let mut tester = RustZXTester::new("z80full", settings);
    tester.load_tap("z80full.tap.gz");
    tester.disable_scroll_message();

    tester
}

fn z80test(mut tester: RustZXTester) {
    tester.emulate_for(Duration::from_millis(358700));
}

fn main() {
    let pool = threadpool::Builder::new().build();

    let (tx, rx) = std::sync::mpsc::channel();

    const TEST_ITERATIONS: usize = 100;

    for _ in 0..TEST_ITERATIONS {
        let tx = tx.clone();
        pool.execute(move || {
            let env = z80test_setup();
            let start = std::time::Instant::now();
            z80test(env);
            let duration = std::time::Instant::now() - start;
            tx.send(duration).unwrap();
        })
    }

    drop(tx);

    pool.join();

    let mut acc = Duration::ZERO;

    while let Ok(duration) = rx.recv() {
        acc += duration;
    }

    let duration = acc / TEST_ITERATIONS as u32;
    println!("`z80test` bench took {}ms", duration.as_millis());
}
