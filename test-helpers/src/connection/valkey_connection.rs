use anyhow::Context;
use redis::aio::MultiplexedConnection;
use redis::{Client, ClientTlsConfig, TlsCertificates};
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Duration;

fn new_sync_connection(address: &str, port: u16) -> redis::Connection {
    let connection = Client::open((address, port))
        .unwrap()
        .get_connection()
        .with_context(|| format!("Failed to create valkey connection to port {port}"))
        .unwrap();
    connection
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    connection
}

async fn new_async_connection(address: &str, port: u16) -> MultiplexedConnection {
    Client::open((address, port))
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .unwrap()
}

fn create_tls_client(address: &str, port: u16) -> Client {
    let root_cert_file =
        File::open("tests/test-configs/valkey/tls/certs/localhost_CA.crt").unwrap();
    let mut root_cert_vec = Vec::new();
    BufReader::new(root_cert_file)
        .read_to_end(&mut root_cert_vec)
        .unwrap();

    let cert_file = File::open("tests/test-configs/valkey/tls/certs/localhost.crt").unwrap();
    let mut client_cert_vec = Vec::new();
    BufReader::new(cert_file)
        .read_to_end(&mut client_cert_vec)
        .unwrap();

    let key_file = File::open("tests/test-configs/valkey/tls/certs/localhost.key").unwrap();
    let mut client_key_vec = Vec::new();
    BufReader::new(key_file)
        .read_to_end(&mut client_key_vec)
        .unwrap();

    Client::build_with_tls(
        format!("rediss://{address}:{port}/#insecure"),
        TlsCertificates {
            client_tls: Some(ClientTlsConfig {
                client_cert: client_cert_vec,
                client_key: client_key_vec,
            }),
            root_cert: Some(root_cert_vec),
        },
    )
    .unwrap()
}

async fn new_async_tls_connection(address: &str, port: u16) -> redis::aio::MultiplexedConnection {
    let client = create_tls_client(address, port);
    client.get_multiplexed_async_connection().await.unwrap()
}

fn new_sync_tls_connection(address: &str, port: u16) -> redis::Connection {
    let client = create_tls_client(address, port);
    client.get_connection().unwrap()
}

pub struct ValkeyConnectionCreator {
    pub address: String,
    pub port: u16,
    pub tls: bool,
}

impl ValkeyConnectionCreator {
    pub async fn new_async(&self) -> redis::aio::MultiplexedConnection {
        if self.tls {
            new_async_tls_connection(&self.address, self.port).await
        } else {
            new_async_connection(&self.address, self.port).await
        }
    }

    pub fn new_sync(&self) -> redis::Connection {
        if self.tls {
            new_sync_tls_connection(&self.address, self.port)
        } else {
            new_sync_connection(&self.address, self.port)
        }
    }
}
