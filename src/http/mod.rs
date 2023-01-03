use std::net::{Ipv4Addr, SocketAddr};

use crate::db::{Db, Leases4Record};
use anyhow::Result;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use chrono::{DateTime, Local};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Lease4 {
    hardware_address: Vec<u8>,
    ip_addr: Ipv4Addr,
    ttl: DateTime<Local>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Lease4AllResponse {
    Ok { leases: Vec<Lease4> },
    Err(String),
}

impl From<Leases4Record> for Lease4 {
    fn from(value: Leases4Record) -> Self {
        Lease4 {
            hardware_address: value.hardware_address,
            ip_addr: value.ip_addr,
            ttl: value.ttl
        }
    }
}

async fn get_all_leases() -> impl IntoResponse {
    let Ok(leases) = Db::open().leases_tree() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(Lease4AllResponse::Err("Internal Server Error".to_string())));
    };
    (
        StatusCode::OK,
        Json(Lease4AllResponse::Ok {
            leases: leases.all().into_iter().map(Lease4::from).collect(),
        }),
    )
}

pub async fn serve() -> Result<()> {
    let app = Router::new().route("/leases4", get(get_all_leases));
    let addr = SocketAddr::from(([0, 0, 0, 0], 5512));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
