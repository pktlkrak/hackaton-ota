// Can work in an embedded env, as long as there exists the concept of a heap:
#![no_std]
extern crate alloc;

pub mod core;
pub mod errors;
pub mod structs;
pub mod traits;
