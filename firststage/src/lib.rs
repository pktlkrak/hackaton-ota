// Can work in an embedded env, as long as there exists the concept of a heap:
#![no_std]
extern crate alloc;

mod structs;
mod traits;
mod core;