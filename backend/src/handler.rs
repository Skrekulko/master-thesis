use crate::ieee754::fheint32_to_ieee754;
use crate::{
    structs::AppState, 
    model::PlaintextDistances,
    model::CiphertextDistances,
    schema::PlaintextCoordinatesSchema,
    schema::CiphertextCoordinatesSchema,
    ieee754::u32_to_ieee754_2nd,
    sqrt::fsqrt,
};

use actix_web::{get, post, delete, web, HttpResponse, Responder};
use serde_json::json;
use bincode::{serialize, deserialize};
use tfhe::{generate_keys, prelude::*, set_server_key, ClientKey, ConfigBuilder, ServerKey};
use tfhe::FheInt32;
use sha2::{Sha256, Digest};

// ----------------------
// |    Health Check    |
// ----------------------

// Check the service's health
#[get("/health")]
async fn health_handler() -> impl Responder {
    const MESSAGE: &str = "The server is healthy and running!";

    HttpResponse::Ok().json(json!({"status": "success","message": MESSAGE}))
}

// ---------------------
// |    Admin Tools    |
// ---------------------

// Dump the ciphertext table
#[get("/admin/dump")]
async fn admin_dump_handler(
    data: web::Data<AppState>,
) -> impl Responder {
    let query_result = sqlx::query_as!(
        CiphertextDistances,
        "SELECT * FROM ciphertextdistances"
    )
    .fetch_all(&data.db)
    .await;

    match query_result {
        Ok(rows) => {
            let response = serde_json::json!({"status": "success", "data": serde_json::json!({
                "rows": rows
            })});

            return HttpResponse::Ok().json(response);
        }

        Err(e) => {
            return HttpResponse::InternalServerError()
            .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
        }
    }
}

// Wipe the ciphertext table
#[delete("/admin/wipe")]
async fn admin_wipe_handler(
    data: web::Data<AppState>,
) -> impl Responder {
    let query_result = sqlx::query!(
        "DELETE FROM ciphertextdistances"
    )
    .execute(&data.db)
    .await;

    match query_result {
        Ok(_) => {
            let response = serde_json::json!({
                "status": "success",
                "message": "Table cachedcalculations wiped"
            });

            return HttpResponse::Ok().json(response);
        }

        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("{:?}", e)}
            ));
        }
    }
}

// Dump the plaintext table
#[get("/admin/dump/test")]
async fn admin_dump_test_handler(
    data: web::Data<AppState>,
) -> impl Responder {
    let query_result = sqlx::query_as!(
        CiphertextDistances,
        "SELECT * FROM plaintextdistances"
    )
    .fetch_all(&data.db)
    .await;

    match query_result {
        Ok(rows) => {
            let response = serde_json::json!({"status": "success", "data": serde_json::json!({
                "rows": rows
            })});

            return HttpResponse::Ok().json(response);
        }

        Err(e) => {
            return HttpResponse::InternalServerError()
            .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
        }
    }
}

// Wipe the plaintext table
#[delete("/admin/wipe/test")]
async fn admin_wipe_test_handler(
    data: web::Data<AppState>,
) -> impl Responder {
    let query_result = sqlx::query!(
        "DELETE FROM plaintextdistances"
    )
    .execute(&data.db)
    .await;

    match query_result {
        Ok(_) => {
            let response = serde_json::json!({
                "status": "success",
                "message": "Table plaintextdistances wiped"
            });

            return HttpResponse::Ok().json(response);
        }

        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("{:?}", e)}
            ));
        }
    }
}

#[post("/admin/calc/dist")]
async fn calculate_distance_plaintext(
    body: web::Json<PlaintextCoordinatesSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    // Unwrap the Vector of bytes into 4 byte array and convert into a i32
    let pax: i32 = deserialize(&body.coordinate_a.x).unwrap();
    let pay: i32 = deserialize(&body.coordinate_a.y).unwrap();
    let pbx: i32 = deserialize(&body.coordinate_b.x).unwrap();
    let pby: i32 = deserialize(&body.coordinate_b.y).unwrap();

    // Select max and min from coordinates
    let x1: i32 = std::cmp::max(pax, pbx);
    let x2: i32 = std::cmp::min(pax, pbx);
    let y1: i32 = std::cmp::max(pay, pby);
    let y2: i32 = std::cmp::min(pay, pby);

    // Compute the radicand
    let radicand: i32 = (x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1);

    // Serialize the radicand
    let rs: Vec<u8> = serialize(&radicand).unwrap();

    // Search for the radicand
    let query_result = sqlx::query_as!(
        PlaintextDistances,
        "SELECT id, distance FROM plaintextdistances WHERE id = $1",
        rs
    )
    .fetch_one(&data.db)
    .await;

    // Create a Sha256 object
    let mut hasher = Sha256::new();

    match query_result {
        // Return the square root if it was already computed
        Ok(value) => {
            if let Some(ref distance) = value.distance {
                // Write input data
                hasher.update(distance);
                let digest = hasher.finalize();
    
                let response = serde_json::json!({"
                    status": "success",
                    "data": serde_json::json!({
                        "distance": value.distance.clone(),
                        "digest": digest[..],
                        "comment": "precalculated"
                    })
                });
    
                return HttpResponse::Ok().json(response);
            }
        }

        Err(e) => {
            println!("{:?}", e.to_string());
            if !e.to_string().contains("no rows returned by a query that expected to return at least one row") {
                return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
            }
        }
    }

    // Convert the radicand into a IEEE 754 format
    // let r754: u32 = u32_to_ieee754_2nd(radicand as u32);
    let r754: u32 = radicand as u32;

    // Compute the square root
    let sr: f32 = f32::sqrt(r754 as f32);

    // Serialize the square root
    let srs: Vec<u8> = serialize(&sr).unwrap();

    // Insert the square root into the table
    let query_result = sqlx::query_as!(
        PlaintextDistances,
        "INSERT INTO plaintextdistances (id, distance) VALUES ($1, $2)",
        rs,
        srs
    )
    .execute(&data.db)
    .await;

    match query_result {
        // Return the newly computed square root
        Ok(_) => {
            // Write input data
            hasher.update(&srs);
            let digest = hasher.finalize();

            let response = serde_json::json!({"
                status": "success",
                "data": serde_json::json!({
                    "distance": srs,
                    "digest": digest[..],
                    "comment": "calculated"
                })
            });

            return HttpResponse::Ok().json(response);
        }

        Err(e) => {
            println!("{:?}", e.to_string());
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
        }
    }
}

// ------------------------------
// |    Production Endpoints    |
// ------------------------------

// Generate and return a new pair of keys
#[get("/init")]
async fn initialize_keys() -> impl Responder {
    // Generate new pair of keys
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    
    // Serialize the keys
    let client_key_serialized: Vec<u8> = serialize(&client_key).unwrap();
    let server_key_serialized: Vec<u8> = serialize(&server_key).unwrap();

    // JSON the keys and return them
    let response = serde_json::json!({
        "status": "success",
        "data": serde_json::json!({
            "client_key": client_key_serialized,
            "server_key": server_key_serialized,
        })
    });

    return HttpResponse::Ok().json(response);
}

// Generate and return a new pair of keys
#[post("/encrypt")]
async fn encrypt(
    body: web::Json<CiphertextCoordinatesSchema>,
) -> impl Responder {
    // Deserialize the server key
    let client_key: ClientKey = deserialize(&body.client_key).unwrap();

    // Unwrap the Vector of bytes into 4 byte array and convert into a i32
    let x1: i32 = deserialize(&body.coordinate_a.x).unwrap();
    let y1: i32 = deserialize(&body.coordinate_a.y).unwrap();
    let x2: i32 = deserialize(&body.coordinate_b.x).unwrap();
    let y2: i32 = deserialize(&body.coordinate_b.y).unwrap();
    
    // Encrypt the values
    let x1_encrypted: FheInt32 = FheInt32::encrypt(x1, &client_key);
    let y1_encrypted: FheInt32 = FheInt32::encrypt(y1, &client_key);
    let x2_encrypted: FheInt32 = FheInt32::encrypt(x2, &client_key);
    let y2_encrypted: FheInt32 = FheInt32::encrypt(y2, &client_key);

    // Serialize the encrypted values
    let x1_serialized: Vec<u8> = serialize(&x1_encrypted).unwrap();
    let y1_serialized: Vec<u8> = serialize(&y1_encrypted).unwrap();
    let x2_serialized: Vec<u8> = serialize(&x2_encrypted).unwrap();
    let y2_serialized: Vec<u8> = serialize(&y2_encrypted).unwrap();

    // JSON the keys and return them
    let response = serde_json::json!({
        "status": "success",
        "data": serde_json::json!({
            "coordinate_a": {
                "x": x1_serialized,
                "y": y1_serialized,
            },
            "coordinate_b": {
                "x": x2_serialized,
                "y": y2_serialized,
            },
        })
    });

    return HttpResponse::Ok().json(response);
}

// TODO: continue this
#[post("/calc/dist")]
async fn calculate_distance_ciphertext(
    body: web::Json<CiphertextCoordinatesSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    // Deserialize the server key and set it
    let server_key: ServerKey = deserialize(&body.server_key).unwrap();
    set_server_key(server_key);

    // Deserialize the coordinations
    let pax: FheInt32 = deserialize(&body.coordinate_a.x).unwrap();
    let pay: FheInt32 = deserialize(&body.coordinate_a.y).unwrap();
    let pbx: FheInt32 = deserialize(&body.coordinate_b.x).unwrap();
    let pby: FheInt32 = deserialize(&body.coordinate_b.y).unwrap();

    // Select max and min from coordinates
    let x1: FheInt32 = pax.min(&pbx);
    let x2: FheInt32 = pay.min(&pbx);
    let y1: FheInt32 = pay.min(&pby);
    let y2: FheInt32 = pay.min(&pby);

    // Compute the radicand
    let r: FheInt32 = (x2.clone() - x1.clone()) * (x2.clone() - x1.clone()) + (y2.clone() - y1.clone()) * (y2.clone() - y1.clone());
    
    // Serialize the radicand);
    let rs: Vec<u8> = serialize(&r).unwrap();

    // Search for the radicand
    let query_result = sqlx::query_as!(
        CiphertextDistances,
        "SELECT id, distance FROM plaintextdistances WHERE id = $1",
        rs
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        // Return the square root if it was already computed
        Ok(value) => {
            let response = serde_json::json!({"status": "success", "data": serde_json::json!({
                "distance": value.distance
            })});

            return HttpResponse::Ok().json(response);
        }

        Err(e) => {
            println!("{:?}", e.to_string());
            if !e.to_string().contains("no rows returned by a query that expected to return at least one row") {
                return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
            }
        }
    }

    // Convert the radicand into a IEEE 754 format
    let r754: FheInt32 = fheint32_to_ieee754(&r);

    // Compute the square root
    // let roo: FheInt32 = fsqrt(r754);

    let response = serde_json::json!({
        "status": "success",
        "data": serde_json::json!({
            "distance": 1
        })
    });

    return HttpResponse::Ok().json(response);
}

// ------------------------
// |    Service Config    |
// ------------------------

// Service's exposed endpoints
pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(health_handler)
        .service(admin_dump_handler)
        .service(admin_wipe_handler)
        .service(admin_dump_test_handler)
        .service(admin_wipe_test_handler)
        .service(calculate_distance_plaintext)
        .service(initialize_keys)
        .service(encrypt)
        .service(calculate_distance_ciphertext);

    conf.service(scope);
}