
use crate::{
    structs::AppState, 
    model::PlaintextDistances,
    model::CiphertextDistances,
    schema::PlaintextCoordinatesSchema,
    schema::CiphertextCoordinatesSchema,
};

use actix_web::{get, post, delete, web, HttpResponse, Responder};
use diesel::IntoSql;
use serde_json::json;

use tfhe::prelude::*;
use tfhe::{FheInt32, FheUint8};

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

#[post("/admin/calc/dist")]
async fn test_calculate_distance(
    body: web::Json<PlaintextCoordinatesSchema>,
    data: web::Data<AppState>,
) -> impl Responder {

    // TODO: this should be done on the client's side
    // Validate the incoming data
    if body.coordinate_a.x < 0 || body.coordinate_a.y < 0 || body.coordinate_b.x < 0 || body.coordinate_b.y < 0 {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid input. Values must be greater than or equal to 0."}
        ));
    }

    // Select max and min from coordinates
    let x1: i32 = std::cmp::max(body.coordinate_a.x, body.coordinate_b.x);
    let x2: i32 = std::cmp::min(body.coordinate_a.x, body.coordinate_b.x);
    let y1: i32 = std::cmp::max(body.coordinate_a.y, body.coordinate_b.y);
    let y2: i32 = std::cmp::min(body.coordinate_a.y, body.coordinate_b.y);

    // Compute the radicand
    let radicand = (x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1);

    // Search for the radicand
    let query_result = sqlx::query_as!(
        PlaintextDistances,
        "SELECT id, distance FROM plaintextdistances WHERE id = $1",
        radicand
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

    // Compute the square root
    let square_root = f32::sqrt(radicand as f32);

    // Insert the square root into the table
    let query_result = sqlx::query_as!(
        PlaintextDistances,
        "INSERT INTO plaintextdistances (id, distance) VALUES ($1, $2)",
        radicand,
        square_root as f32
    )
    .execute(&data.db)
    .await;

    match query_result {
        // Return the newly computed square root
        Ok(_) => {
            let response = serde_json::json!({"status": "success", "data": serde_json::json!({
                "distance": square_root
            })});

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

// TODO: continue this
#[post("/calc/dist")]
async fn calculate_distance_handler(
    body: web::Json<CiphertextCoordinatesSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    // Select max and min from coordinates
    let x1: FheInt32 = body.coordinate_a.x.min(&body.coordinate_b.x);
    let x2: FheInt32 = body.coordinate_a.x.min(&body.coordinate_b.x);
    let y1: FheInt32 = body.coordinate_a.y.min(&body.coordinate_b.y);
    let y2: FheInt32 = body.coordinate_a.y.min(&body.coordinate_b.y);

    // Compute the radicand
    let radicand: FheInt32 = (x2.clone() - x1.clone()) * (x2.clone() - x1.clone()) + (y2.clone() - y1.clone()) * (y2.clone() - y1.clone());
    let encoded: Vec<u8> = bincode::serialize(&radicand).unwrap();
    let encstr: String = String::from_utf8(encoded).expect("The byte stream should be UTF-8 encoded");
    println!("{:#?}", encstr);

    // Search for the radicand
    // let query_result = sqlx::query_as!(
    //     CiphertextDistances,
    //     "SELECT id, distance FROM plaintextdistances WHERE id = $1",
    //     encoded
    // )
    // .fetch_one(&data.db)
    // .await;

    // match query_result {
    //     // Return the square root if it was already computed
    //     Ok(value) => {
    //         let response = serde_json::json!({"status": "success", "data": serde_json::json!({
    //             "distance": value.distance
    //         })});

    //         return HttpResponse::Ok().json(response);
    //     }

    //     Err(e) => {
    //         println!("{:?}", e.to_string());
    //         if !e.to_string().contains("no rows returned by a query that expected to return at least one row") {
    //             return HttpResponse::InternalServerError()
    //             .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
    //         }
    //     }
    // }

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
        .service(test_calculate_distance);
        // .service(calculate_distance_handler);

    conf.service(scope);
}