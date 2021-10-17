### RustZX v0.15
<!-- START_CHANGELOG|v0.15.0 -->
- **[Feature]** Added new `cpal` sound backend (#101)
- **[Feature]** Added support for `.gz`-compressed assets (#109)
- **[Feature]** Added support for non-standard Kempston joy buttons in rustzx-core (#113)
- **[Fix]** Fixed bug with integer overflow panic in tape loader (#100)
- **[Testing]** Added integration tests for rustzx-core (#92)
- **[Refactoring]** Removed cyclic dependencies between `rustzx-core` and `rustzx-utils`(#99)
<!-- END_CHANGELOG|v0.15.0 -->

### RustZX v0.14
- **[Refactoring]** Moved z80 emulation to the separate crate (`rustzx-z80`)
- **[Feature]** Added support for `*.scr` screenshot format loading
- **[Feature]** Improved beeper emulation logic
- **[Fix]** Fixed invalid EAR bit value on read from 0xFE port
- **[Fix]** Fixed bug with invalid SNA loading logic
- **[Infrastructure]** Created basic integration testing framework for rustzx

### RustZX v0.13
- **[Feature]** Support for ZX Spectrum 128K SNA variant
- **[Feature]** Quick save/load (`F1`/`F2` keys) via SNA
- **[Feature]** Sinclair joysticks support
- **[Feature]** Separate joystick keyboard layout (press `F9` to activate)
- **[Feature]** Kempston mouse support (`--mouse`)
- **[Fix]** Fixed invalid kempston joystick port
- **[Fix]** Fixed missing screen update after SNA loading

### RustZX v0.12
- **[Feature]** Now using rust-only AY sound chip library (`aym`) instead of `libayumi` C library
- **[Feature]** Added vtx sound format parsing/playback library `vtx`

### RustZX v0.11
Changes:
- **[Feature]** Separated project to `rustzx` application and `no_std`-capable `rustzx-core` crate
- **[Feature]** Updated CLI
    - More features now enabled by default
    - File autodetect from CLI
    - Added configurable sound sample rate
    - Removed redundant args
- **[Dependencies]** Switched to bundled `sdl` crate mode, making build almost dependecy-free (CMake and C compiller are still requird)
- **[Infrastructure]** Moved CI to _Github Actions_
    - Added `rustfmt` step to CI
    - Added `clippy` step to CI
- **[Refactroing]** Performed deep global refactoring of the project
    - Updated to Rust 2018 edition
    - Updated dependencies
    - Fixed all `clippy` errors
    - Minimized `rustzx-core` public interface
    - Added feature gates for resource-hungry `rustzx-core` features
    - Eliminated a lot of not redundant code
    - Made groundwork for a future emulator features

Contributors:
- Vladyslav Nikonov (@pacmancoder)

### RustZX v0.9
Changes:
- **[Feature]** Drag-n-drop support for TAP and SNA files
- **[Refactoring]** Multiple small refactoring changes
- **[Dependencies]** Updated `sdl` crate

Contributors:
- Konstantin Mochalov (@kolen)

### Pre-pelease
Changes:
- **[19.08.2016]** RustzxApp and RustzxSettings refactoring
- **[18.08.2016]** Moved sound, video, event to sdl lib.
- **[15.08.2016]** Moving from **portaudio** to **cpal**
- **[14.08.2016]** Kempston Joystick
- **[14.08.2016]** AY implementation finished
- **[12.08.2016]** AY implementation start
- **[09.08.2016]** Refactoring
- **[08.08.2016]** Aspect ratio correction, custom 128K rom loading
- **[08.08.2016]** Window scale selection with `--scale` option
- **[07.07.2016]** ZXScreen rewrite
- **[06.07.2016]** Base 128K features implemented
- **[05.07.2016]** v0.8 development started in branch `develop`
- **[27.06.2016]** Release v0.7.1
- **[26.06.2016]** Beeper sound implemented :notes:, release planed to July 1 :rocket:
- **[12.06.2016]** Some Comand line arguments fixes/enchantments
- **[12.06.2016]** SNA files loading
- **[11.06.2016]** Command line arguments using **clap** crate
- **[11.06.2016]** Tap files fast loading implemented, finnaly!
- **[07.06.2016]** Speed improvements (maybe :smile:) in flag setting [z80]
- **[04.06.2016]** Border FX implemented
- **[28.05.2016]** Some architecture rewrite, working on border
- **[21.05.2016]** OVERSCAN and SHOCK demo's passed! :sparkles:
- **[21.05.2016]** Screen reorganization and OpenGL rendering part fix
- **[19.05.2016]** Documentation, Rustfmt
- **[15.05.2016]** Fixed bug in INC/DEC (IX/IY + dd). After 2 weeks :smile:
- **[12.05.2016]** Fixed CALL timings
- **[06.05.2016]** Floating bus fix
- **[28.04.2016]** All contentions implemented!
- **[24.04.2016]** Almost all contentions working perfectly (IO still broken)
- **[24.04.2016]** IM2 bug fixed, finally I found it! :smile:
- **[24.04.2016]** new Z80Bus interface, serious z80 emulation part rewrite
- **[22.04.2016]** work on implementing correct timings started.
- **[18.04.2016]** fixed many instruction bugs (IO section still not finished)
- **[12.04.2016]** fixed shader bug causing bad performance - palette was declared as non-const
- **[12.04.2016]** log file added
- **[29.03.2016]** Screen emulation, keyboard, test run of ROMs
- **[14.03.2016]** All features of CPU have been implemented :sunglasses:
- **[11.03.2016]** Serious code reorganization
- **[06.03.2016]** All Z80 instruction groups have been implemented! :tada:
- **[02.02.2016]** First commit

Contributors:
- Vladyslav Nikonov (@pacmancoder)
