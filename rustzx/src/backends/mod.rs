//! Module contains sdl thread local static initialization
use sdl2::{self, Sdl};
use std::cell::RefCell;

thread_local! (pub static SDL_CONTEXT: RefCell<Sdl> = RefCell::new(
        sdl2::init().expect("SDL init failed")));
