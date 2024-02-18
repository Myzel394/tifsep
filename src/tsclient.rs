pub mod tsclient {
    use std::io::{self, BufReader, Read, Write};
    use std::net::ToSocketAddrs;
    use std::sync::Arc;
    use std::time::Instant;
    use std::{fs, process, str};

    use mio::net::TcpStream;

    use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
    use rustls::RootCertStore;

    const CLIENT: mio::Token = mio::Token(0);

    /// This encapsulates the TCP-level connection, some connection
    /// state, and the underlying TLS-level session.
    pub struct TlsClient {
        socket: TcpStream,
        closing: bool,
        clean_closure: bool,
        tls_conn: rustls::ClientConnection,
        now: Instant,
    }

    impl TlsClient {
        pub fn new(
            sock: TcpStream,
            server_name: ServerName<'static>,
            cfg: Arc<rustls::ClientConfig>,
        ) -> Self {
            Self {
                socket: sock,
                closing: false,
                clean_closure: false,
                tls_conn: rustls::ClientConnection::new(cfg, server_name).unwrap(),
                now: Instant::now(),
            }
        }

        /// Handles events sent to the TlsClient by mio::Poll
        pub fn ready(&mut self, ev: &mio::event::Event) {
            assert_eq!(ev.token(), CLIENT);

            if ev.is_readable() {
                self.do_read();
            }

            if ev.is_writable() {
                self.do_write();
            }

            if self.is_closed() {
                println!("Connection closed");
            }
        }

        fn read_source_to_end(&mut self, rd: &mut dyn io::Read) -> io::Result<usize> {
            let mut buf = Vec::new();
            let len = rd.read_to_end(&mut buf)?;
            self.tls_conn.writer().write_all(&buf).unwrap();
            Ok(len)
        }

        /// We're ready to do a read.
        fn do_read(&mut self) {
            // Read TLS data.  This fails if the underlying TCP connection
            // is broken.
            match self.tls_conn.read_tls(&mut self.socket) {
                Err(error) => {
                    if error.kind() == io::ErrorKind::WouldBlock {
                        return;
                    }
                    println!("TLS read error: {:?}", error);
                    self.closing = true;
                    return;
                }

                // If we're ready but there's no data: EOF.
                Ok(0) => {
                    self.closing = true;
                    self.clean_closure = true;
                    return;
                }

                Ok(_) => {}
            };

            // Reading some TLS data might have yielded new TLS
            // messages to process.  Errors from this indicate
            // TLS protocol problems and are fatal.
            let io_state = match self.tls_conn.process_new_packets() {
                Ok(io_state) => io_state,
                Err(err) => {
                    println!("TLS error: {:?}", err);
                    self.closing = true;
                    return;
                }
            };

            // Having read some TLS data, and processed any new messages,
            // we might have new plaintext as a result.
            //
            // Read it and then write it to stdout.
            if io_state.plaintext_bytes_to_read() > 0 {
                let mut plaintext = vec![0u8; io_state.plaintext_bytes_to_read()];
                self.tls_conn.reader().read_exact(&mut plaintext).unwrap();

                // dbg!(self.now.elapsed());
                println!("{}", "=======");
            }

            // If that fails, the peer might have started a clean TLS-level
            // session closure.
            if io_state.peer_has_closed() {
                self.clean_closure = true;
                self.closing = true;
            }
        }

        fn do_write(&mut self) {
            self.tls_conn.write_tls(&mut self.socket).unwrap();
        }

        /// Registers self as a 'listener' in mio::Registry
        pub fn register(&mut self, registry: &mio::Registry) {
            let interest = self.event_set();
            registry
                .register(&mut self.socket, CLIENT, interest)
                .unwrap();
        }

        /// Reregisters self as a 'listener' in mio::Registry.
        pub fn reregister(&mut self, registry: &mio::Registry) {
            let interest = self.event_set();
            registry
                .reregister(&mut self.socket, CLIENT, interest)
                .unwrap();
        }

        /// Use wants_read/wants_write to register for different mio-level
        /// IO readiness events.
        fn event_set(&self) -> mio::Interest {
            let rd = self.tls_conn.wants_read();
            let wr = self.tls_conn.wants_write();

            if rd && wr {
                mio::Interest::READABLE | mio::Interest::WRITABLE
            } else if wr {
                mio::Interest::WRITABLE
            } else {
                mio::Interest::READABLE
            }
        }

        pub fn is_closed(&self) -> bool {
            self.closing
        }
    }
    impl io::Write for TlsClient {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.tls_conn.writer().write(bytes)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.tls_conn.writer().flush()
        }
    }

    impl io::Read for TlsClient {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            self.tls_conn.reader().read(bytes)
        }
    }
}
