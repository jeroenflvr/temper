use serde_derive::Deserialize;
use axum::{routing::post, Router, Json, http::StatusCode, response::IntoResponse, extract::State};
use std::net::SocketAddr;
use chrono::{Utc, DateTime};
use futures::stream;
use influxdb2::{Client, models::DataPoint};

#[derive(Debug, Deserialize)]
struct SensorData {
    name: String,
    rpi_temp: f32,
    temp: f32,
    pressure: f32,
    humidity: f32,
    ip_address: String,
    uptime: u64,
}

#[derive(Debug)]
struct SensorDataWithTimestamp {
    name: String,
    rpi_temp: f32,
    temp: f32,
    pressure: f32,
    humidity: f32,
    ip_address: String,
    uptime: u64,
    timestamp: DateTime<Utc>,
}

async fn sensors_handler(
    State(influx): State<Client>,
    Json(payload): Json<SensorData>
) -> Result<StatusCode, (StatusCode, String)> {
    let data_with_timestamp = SensorDataWithTimestamp {
        name: payload.name.clone(),
        rpi_temp: payload.rpi_temp,
        temp: payload.temp,
        pressure: payload.pressure,
        humidity: payload.humidity,
        ip_address: payload.ip_address.clone(),
        uptime: payload.uptime,
        timestamp: Utc::now(),
    };
    println!("Received sensor data: {:?}", data_with_timestamp);

    let point = DataPoint::builder("rpi_sensors")
        .tag("name", payload.name.clone())
        .tag("ip_address", payload.ip_address.clone())
        .field("rpi_temp", payload.rpi_temp as f64)
        .field("temp", payload.temp as f64)
        .field("pressure", payload.pressure as f64)
        .field("humidity", payload.humidity as f64)
        .field("uptime", payload.uptime as i64)
        .build()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DataPoint error: {}", e)))?;

    influx
        .write("sensors-bucket", stream::iter([point]))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Influx write error: {}", e)))?;

    Ok(StatusCode::OK)
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16 number");

    let influx_host = std::env::var("INFLUX_HOST").expect("INFLUX_HOST must be set");
    let influx_port = std::env::var("INFLUX_PORT").unwrap_or_else(|_| "8086".to_string());
    let influx_token = std::env::var("INFLUX_TOKEN").expect("INFLUX_TOKEN must be set");
    let influx_org = std::env::var("INFLUX_ORG").expect("INFLUX_ORG must be set");
    let influx_url = format!("http://{}:{}", influx_host, influx_port);
    let influx = Client::new(influx_url, influx_org, influx_token);

    let app = Router::new()
        .route("/sensors", post(sensors_handler))
        .with_state(influx);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Listening on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service()
    )
    .await
    .unwrap();
}
