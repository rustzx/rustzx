use expect_test::expect;
use rustzx_core::zx::mouse::kempston::{KempstonMouseButton, KempstonMouseWheelDirection};
use rustzx_test::framework::{presets, RustZXTester};
use std::time::Duration;

#[test]
fn kempston_mouse() {
    let mut settings = presets::settings_48k_nosound();
    settings.mouse_enabled = true;

    let mut tester = RustZXTester::new("kempston_mouse", settings);
    tester.load_sna("mouse.48k.sna.gz");
    tester.emulate_for(Duration::from_millis(250));
    tester.expect_screen(
        "sna_loaded",
        expect![[r#"7AVgP7YkJPt4y4x1NnlMyWSo6wntyPaRcrVHBmfSfFk="#]],
    );

    tester.emulator().send_mouse_pos_diff(64, 32);
    tester.emulate_for(Duration::from_millis(250));
    tester.expect_screen(
        "cursor_moved_1",
        expect![[r#"KXWv0nZ+P2/PCWAWXh0eIOsncVrPISgYqggniZifLQs="#]],
    );

    tester.emulator().send_mouse_pos_diff(-48, -16);
    tester.emulate_for(Duration::from_millis(250));
    tester.expect_screen(
        "cursor_moved_2",
        expect![[r#"HUIX0sv7yUhYjAU6rNxF4ODzLhm5WYGgyHKcJ3z0EuI="#]],
    );

    tester
        .emulator()
        .send_mouse_button(KempstonMouseButton::Left, true);
    tester.emulate_for(Duration::from_millis(250));
    tester.expect_screen(
        "left_pressed",
        expect![[r#"zJJGnWCk90irqNKUEeEAMoYxutt2sI6b1oaFN9sRWmA="#]],
    );
    tester
        .emulator()
        .send_mouse_button(KempstonMouseButton::Left, false);

    tester
        .emulator()
        .send_mouse_button(KempstonMouseButton::Right, true);
    tester.emulate_for(Duration::from_millis(250));
    tester.expect_screen(
        "right_pressed",
        expect![[r#"WDsRHUt7EzlfGvlPMOsPvmCzdP0ob15l5FARzE390b8="#]],
    );
    tester
        .emulator()
        .send_mouse_button(KempstonMouseButton::Right, false);

    tester
        .emulator()
        .send_mouse_button(KempstonMouseButton::Middle, true);
    tester.emulate_for(Duration::from_millis(250));
    tester.expect_screen(
        "middle_pressed",
        expect![[r#"RaAwafxap1fFOUTBOeaNHlSgIEW+kRN41umpEc/ccdk="#]],
    );
    tester
        .emulator()
        .send_mouse_button(KempstonMouseButton::Middle, false);

    tester
        .emulator()
        .send_mouse_button(KempstonMouseButton::Additional, true);
    tester.emulate_for(Duration::from_millis(250));
    tester.expect_screen(
        "ext_pressed",
        expect![[r#"ffhRpkdMjqHxAns5Wb40Sp2IzMR6gYuxJXrRiOB7ZNQ="#]],
    );
    tester
        .emulator()
        .send_mouse_button(KempstonMouseButton::Additional, false);

    (0..10).for_each(|_| {
        tester
            .emulator()
            .send_mouse_wheel(KempstonMouseWheelDirection::Down)
    });
    tester.emulate_for(Duration::from_millis(250));
    (0..10).for_each(|_| {
        tester
            .emulator()
            .send_mouse_wheel(KempstonMouseWheelDirection::Down)
    });
    tester.emulate_for(Duration::from_millis(250));
    tester.expect_screen(
        "wheel_2",
        expect![[r#"oeQS+og5xrGSyz3Zk+f3L4U93aB2azba0jl0mVU96xI="#]],
    );

    (0..10).for_each(|_| {
        tester
            .emulator()
            .send_mouse_wheel(KempstonMouseWheelDirection::Up)
    });
    tester.emulate_for(Duration::from_millis(250));
    (0..10).for_each(|_| {
        tester
            .emulator()
            .send_mouse_wheel(KempstonMouseWheelDirection::Up)
    });
    tester.emulate_for(Duration::from_millis(250));
    tester.expect_screen(
        "wheel_3",
        expect![[r#"I+2mija0+YU60eHjAehkN9MpfgMli2ym7pMoChVbcFo="#]],
    );
}
