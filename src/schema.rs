use serde::{Deserialize, Serialize};

// --------------------------------
// |    Ciphertext Coordinates    |
// --------------------------------

#[derive(Serialize, Deserialize)]
pub struct CiphertextCoordinatesSchema {
    pub coordinate_a: CiphertextCoordinate,
    pub coordinate_b: CiphertextCoordinate,
}

#[derive(Serialize, Deserialize)]
pub struct CiphertextCoordinate {
    pub x: Vec<u8>,
    pub y: Vec<u8>,
}

// -------------------------------
// |    Plaintext Coordinates    |
// -------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaintextCoordinatesSchema {
    pub coordinate_a: PlaintextCoordinate,
    pub coordinate_b: PlaintextCoordinate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaintextCoordinate {
    pub x: Vec<u8>,
    pub y: Vec<u8>,
}
