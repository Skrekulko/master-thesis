use serde::{Deserialize, Serialize};
use tfhe::FheInt32;

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
    pub x: FheInt32,
    pub y: FheInt32,
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
    pub x: i32,
    pub y: i32,
}
