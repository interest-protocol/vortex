use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct NewPoolEvent(pub [u8; 32]);

#[derive(Debug, Clone, Deserialize)]
pub struct NewCommitmentEvent {
    pub index: u64,
    pub commitment: [u8; 32],
    pub encrypted_output: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NullifierSpentEvent(pub [u8; 32]);
