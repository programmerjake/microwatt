#![cfg_attr(feature = "embedded", no_std)]
#![cfg_attr(feature = "embedded", no_main)]

extern crate rust_voxels_game;

#[cfg(feature = "hosted")]
fn main() {
    rust_voxels_game::main()
}
