use serde_derive::Deserialize;
use axum::{routing::post, Router, Json, http::StatusCode, response::IntoResponse};
use std::net::SocketAddr;
use chrono::{Utc, DateTime};

#[derive(Debug, Deserialize)]
struct SensorData {
    name: String,
    temp: f32,
    pressure: i32,
    humidity: i32,
    ip_address: String,
    uptime: u64,
}


#[derive(Debug)]
struct SensorDataWithTimestamp {
    name: String,
    temp: f32,
    pressure: i32,
    humidity: i32,
    ip_address: String,
    uptime: u64,
    timestamp: DateTime<Utc>,
}

async fn sensors_handler(Json(payload): Json<SensorData>) -> impl IntoResponse {
    let data_with_timestamp = SensorDataWithTimestamp {
        name: payload.name,
        temp: payload.temp,
        pressure: payload.pressure,
        humidity: payload.humidity,
        ip_address: payload.ip_address,
        uptime: payload.uptime,
        timestamp: Utc::now(),
    };
    println!("Received sensor data: {:?}", data_with_timestamp);
    StatusCode::OK
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/sensors", post(sensors_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 9005));
    println!("Listening on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service()
    )
    .await
    .unwrap();
}
