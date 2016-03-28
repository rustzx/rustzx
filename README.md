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
## Log
[02.02.2016] First commit  
[06.03.2016] All Z80 instruction groups have been implemented! :tada:  
[11.03.2016] Serious code reorganization  
[14.03.2016] All features of CPU have been implemented :sunglasses:  
[29.03.2016] Screen emulation, keyboard, test run of ROMs    
