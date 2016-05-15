# rustzx
ZX Spectrum emulator which I writing in rust.   
I develop this project just for fun and for learning the basics of architecture architecture.  
Licensed under MIT License.

[![Build Status](https://travis-ci.org/pacmancoder/rustzx.svg?branch=master)](https://travis-ci.org/pacmancoder/rustzx)

## Current progress
Implementation of ZX Spectrum 48K hardware.  
Tests, refactoring, reorganization.
## Compiling
Rustzx is not usable at the moment.
If you want to test it anyway - copy ROM (machine is 48K) to
`src/app` folder and name it `48.rom`.
And then just execute
```bash
cargo run --release
```
## Screenshots
![](screenshots/ulatest3.png) ![](screenshots/fusetest.png)  
![](screenshots/floatspy.png) ![](screenshots/diagnostics.png)  
![](screenshots/timings.png)
## References
Of course, I used many resources to find out, how to build my first
emulator in life. So there is a list of useful references, from where I dig most information about Z80, ULA and other ZX Spectrum hardware parts:  
- Of course [z80.info](http://www.z80.info/)
    - [Decoding Z80 opcodes](http://www.z80.info/decoding.htm)
    - [Opcodes list](http://www.z80.info/z80code.txt)
    - [CPU user manual](http://www.z80.info/zip/z80cpu_um.pdf)
    - [CPU architecture](http://www.z80.info/z80arki.htm)
    - [Interrupt behaviour](http://www.z80.info/interrup.htm)
    - [Z80 undocumented documented](http://www.z80.info/zip/z80-documented.pdf)
- Instruction table from [ClrHome](http://clrhome.org/table/)
- "Floating bus explained!" by [Ramsoft](http://ramsoft.bbk.org.omegahg.com/floatingbus.html)
- 16K / 48K ZX Spectrum [Reference](http://www.worldofspectrum.org/faq/reference/48kreference.htm)
- 128K ZX Spectrum [Reference](http://www.worldofspectrum.org/faq/reference/128kreference.htm)
- [Z80 hardware organization](http://www.msxarchive.nl/pub/msx/mirrors/msx2.com/zaks/z80prg02.htm)
- [disassembler.io](https://www.onlinedisassembler.com) online disassembler
- Cool z80 assembler [zasm](http://k1.spdns.de/Develop/Projects/zasm-4.0/Distributions/)
- Diagnostic ROM by [Phill](http://www.retroleum.co.uk/electronics-articles/a-diagnostic-rom-image-for-the-zx-spectrum/)
- [zx-modules.de](http://www.zx-modules.de/) - great resource, check it out!
- [speccy.info](http://speccy.info)
- [Harlequin](http://www.zxdesign.info/harlequin.shtml)
- And many other great material, which helped me to make rustzx!
- [FUSE](http://fuse-emulator.sourceforge.net/) emulator source for finding out correct timings
