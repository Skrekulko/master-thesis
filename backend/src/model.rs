use sqlx::FromRow;
use serde::{Deserialize, Serialize};

// --------------------------------
// |    Ciphertext Table Model    |
// --------------------------------

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct CiphertextDistances {
    pub id: Option<Vec<u8>>,
    pub distance: Option<Vec<u8>>,
}

// -------------------------------
// |    Plaintext Table Model    |
// -------------------------------

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct PlaintextDistances {
    pub id: Option<Vec<u8>>,
    pub distance: Option<Vec<u8>>,
}
