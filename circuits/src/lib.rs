use circuit::Circuit;

pub mod circuit;
pub mod merkle_tree;
pub mod poseidon;
pub mod utils;
pub mod wasm;

pub const LEVEL: usize = 26;
pub const ZERO_VALUE: &str =
    "18688842432741139442778047327644092677418528270738216181718229581494125774932";

pub type CircuitType = Circuit<LEVEL>;
