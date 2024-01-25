pub mod client {
    use std::{
        error::Error,
        io::{Read, Write},
        net::TcpStream,
        sync::Arc,
    };

    use reqwest::Url;
    use rustls::{pki_types::ServerName, ClientConnection, RootCertStore};

    pub const PACKET_SIZE: usize = 1400;

    pub struct Client {
        pub url: Url,
        root_store: RootCertStore,
    }

    impl Client {
        pub fn new(url: &str) -> Self {
            let mut root_store = rustls::RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

            Self {
                url: Url::parse(url).unwrap(),
                root_store,
            }
        }

        fn create_connection(&self) -> Result<ClientConnection, Box<dyn Error>> {
            let config = rustls::ClientConfig::builder()
                .with_root_certificates(Arc::new(self.root_store.clone()))
                .with_no_client_auth();

            let server_name_str = self.url.host_str().ok_or("No host in url")?.to_string();
            let server_name = ServerName::try_from(server_name_str)?;

            Ok(ClientConnection::new(Arc::new(config), server_name)?)
        }

        fn create_tcp_stream(&self) -> Result<TcpStream, Box<dyn Error>> {
            let host = format!(
                "{}:{}",
                self.url.host_str().ok_or("No host in url")?,
                self.url.port().or_else(|| match self.url.scheme() {
                    "https" => Some(443),
                    "http" => Some(80),
                    _ => None,
                }).ok_or("No port could be found and the schema is not http or https. Please manually set a port.")?,
            );

            Ok(TcpStream::connect(host)?)
        }

        fn create_http_header(&self, method: &str) -> Result<String, Box<dyn Error>> {
            Ok(format!(
                concat!(
                    "{} {} HTTP/1.1\r\n",
                    "Host: {}\r\n",
                    "Connection: close\r\n",
                    "Accept-Encoding: identity\r\n",
                    "\r\n",
                ),
                method,
                self.url.path(),
                self.url.host_str().ok_or("No host in url")?,
            ))
        }

        pub fn request(
            &self,
            method: &str,
            on_partial: fn(&[u8; PACKET_SIZE], &[u8]),
        ) -> Result<Vec<u8>, Box<dyn Error>> {
            let mut connection = self.create_connection()?;

            let mut sock = self.create_tcp_stream()?;
            let mut tls = rustls::Stream::new(&mut connection, &mut sock);
            let http_header = self.create_http_header(method)?;
            tls.write_all(&http_header.as_bytes())?;

            // Read packages one by one
            let mut data = Vec::new();
            loop {
                let mut buf: [u8; PACKET_SIZE] = [0; PACKET_SIZE];

                let n = tls.read(&mut buf)?;

                on_partial(&buf, &data);

                if n == 0 {
                    break;
                }

                data.extend_from_slice(&buf[..n]);
            }

            Ok(data)
        }
    }
}
