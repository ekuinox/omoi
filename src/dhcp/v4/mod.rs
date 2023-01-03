mod discover;
mod request;

use crate::{
    conf::{OmoiConfig, OMOI_CONFIG},
    db::Db,
};
use anyhow::{bail, Result};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Local};
use dhcproto::{
    v4::{self, Message},
    Decodable, Decoder,
};
use std::{
    collections::{HashMap, HashSet},
    net::{Ipv4Addr, SocketAddr},
    ops::Add,
    sync::{Arc, Mutex},
};
use tokio::net::UdpSocket;

use self::{discover::DiscoverHandler, request::RequestHandler};

pub const BUFFER_SIZE: usize = 1024;
pub const TRANSACTION_EXPIRATION_HOURS: i64 = 1;

fn decode(buffer: &[u8]) -> Result<Message> {
    let mut decoder = Decoder::new(buffer);
    let message = Message::decode(&mut decoder)?;
    Ok(message)
}

pub async fn handle_request(context: Context, buffer: Vec<u8>, _addr: SocketAddr) -> Result<()> {
    let message = decode(&buffer)?;
    let request = Request {
        message: Arc::new(message),
        context,
    };

    if let Ok(_) = DiscoverHandler.handle(request.clone()).await {
        return Ok(());
    }
    if let Ok(_) = RequestHandler.handle(request).await {
        return Ok(());
    }

    Ok(())
}

#[derive(PartialEq, Eq, Debug)]
pub struct Transaction {
    xid: u32,
    offered_ipv4_addr: Ipv4Addr,
    created_at: DateTime<Local>,
}

#[derive(Clone, Debug)]
pub struct Transactions(Arc<Mutex<HashMap<u32, Transaction>>>);

impl Transactions {
    pub fn new() -> Transactions {
        Transactions(Arc::new(Mutex::new(HashMap::new())))
    }
    pub fn new_transaction(&self, xid: u32, offered_ipv4_addr: Ipv4Addr) -> Result<()> {
        let Ok(mut transactions) = self.0.lock() else {
            bail!("transactions lock failed");
        };
        transactions.insert(
            xid,
            Transaction {
                xid,
                offered_ipv4_addr,
                created_at: Local::now(),
            },
        );
        Ok(())
    }
    pub fn remove(&self, xid: u32) -> Result<Transaction> {
        let Ok(mut transactions) = self.0.lock() else {
            bail!("transactions lock failed");
        };
        let Some(transaction) = transactions.remove(&xid) else {
            bail!("transaction xid={xid} not found");
        };
        Ok(transaction)
    }
    pub fn offered_ipv4_addresses(&self) -> Result<HashSet<Ipv4Addr>> {
        let Ok(transactions) = self.0.lock() else {
            bail!("transaction lock failed");
        };
        let now = Local::now();
        let duration = Duration::hours(TRANSACTION_EXPIRATION_HOURS);
        Ok(transactions
            .iter()
            .filter(|(_, t)| now < t.created_at.add(duration))
            .map(|(_, t)| t.offered_ipv4_addr)
            .collect())
    }
}

#[derive(Clone, Debug)]
pub struct Request {
    pub context: Context,
    pub message: Arc<v4::Message>,
}

#[derive(Clone, Debug)]
pub struct Context {
    pub db: Db,
    pub config: Arc<OmoiConfig>,
    pub transactions: Transactions,
    pub socket: Arc<UdpSocket>,
}

#[async_trait]
pub trait Handler {
    async fn handle(&self, request: Request) -> Result<()>;
}

pub async fn serve() -> Result<()> {
    let socket = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), v4::SERVER_PORT)).await?;
    socket.set_broadcast(true)?;
    let socket = Arc::new(socket);
    let context = Context {
        db: Db::open(),
        config: Arc::new(OMOI_CONFIG.clone()),
        transactions: Transactions::new(),
        socket,
    };

    loop {
        let context = context.clone();
        let mut buffer = vec![0u8; BUFFER_SIZE];
        let (_size, addr) = context.socket.recv_from(&mut buffer).await?;
        tokio::spawn(async move {
            if let Err(e) = handle_request(context, buffer, addr).await {
                eprintln!("{e}");
            }
        });
    }
}
