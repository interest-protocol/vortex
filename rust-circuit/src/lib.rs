pub mod circuit;
pub mod constants;
pub mod merkle_tree;
pub mod poseidon_opt;
pub mod utils;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub const ZERO_VALUE: &str =
    "18688842432741139442778047327644092677418528270738216181718229581494125774932";
