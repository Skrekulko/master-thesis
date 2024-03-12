use sqlx::FromRow;
use serde::{Deserialize, Serialize};

// --------------------------------
// |    Ciphertext Table Model    |
// --------------------------------

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct CiphertextDistances {
    pub id: Option<i32>,
    pub distance: Option<i32>,
}

// -------------------------------
// |    Plaintext Table Model    |
// -------------------------------

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct PlaintextDistances {
    pub id: Option<i32>,
    pub distance: Option<i32>,
}
