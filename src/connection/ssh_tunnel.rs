use std::fmt;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use ssh2::Session;

#[derive(Debug, Clone)]
pub struct SSHTunnelConfig {
    pub ssh_host: String,
    pub ssh_port: u16,
    pub ssh_username: String,
    pub ssh_password: Option<String>,
    pub ssh_private_key_path: Option<String>,
    pub ssh_passphrase: Option<String>,
    pub remote_host: String,
    pub remote_port: u16,
}

pub struct SSHTunnel {
    local_port: u16,
    running: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
    config: SSHTunnelConfig,
}

impl fmt::Debug for SSHTunnel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SSHTunnel")
            .field("local_port", &self.local_port)
            .field("running", &self.running.load(Ordering::SeqCst))
            .finish()
    }
}

impl SSHTunnel {
    pub fn start(config: SSHTunnelConfig) -> Result<Self, String> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("Failed to bind local port: {}", e))?;

        let local_port = listener
            .local_addr()
            .map_err(|e| format!("Failed to get local address: {}", e))?
            .port();

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let config_clone = config.clone();

        let handle = thread::spawn(move || {
            while running_clone.load(Ordering::SeqCst) {
                listener.set_nonblocking(true).ok();

                match listener.accept() {
                    Ok((local_stream, _)) => {
                        let running_inner = running_clone.clone();
                        let config_inner = config_clone.clone();

                        thread::spawn(move || {
                            if let Err(e) =
                                handle_connection(local_stream, &config_inner, running_inner)
                            {
                                tracing::error!("SSH tunnel connection error: {}", e);
                            }
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept connection: {}", e);
                    }
                }
            }
        });

        Ok(Self {
            local_port,
            running,
            thread_handle: Some(handle),
            config,
        })
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn check_health(&self) -> bool {
        if !self.is_running() {
            return false;
        }

        if let Ok(stream) = TcpStream::connect(("127.0.0.1", self.local_port)) {
            drop(stream);
            true
        } else {
            false
        }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for SSHTunnel {
    fn drop(&mut self) {
        self.stop();
    }
}

fn handle_connection(
    mut local_stream: TcpStream,
    config: &SSHTunnelConfig,
    running: Arc<AtomicBool>,
) -> Result<(), String> {
    let tcp = TcpStream::connect((config.ssh_host.as_str(), config.ssh_port))
        .map_err(|e| format!("Failed to connect to SSH server: {}", e))?;

    tcp.set_read_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;
    tcp.set_write_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    let mut sess = Session::new().map_err(|e| format!("Failed to create SSH session: {}", e))?;

    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;

    if let Some(ref password) = config.ssh_password {
        sess.userauth_password(&config.ssh_username, password)
            .map_err(|e| format!("SSH authentication failed: {}", e))?;
    } else if let Some(ref key_path) = config.ssh_private_key_path {
        sess.userauth_pubkey_file(
            &config.ssh_username,
            None,
            std::path::Path::new(key_path),
            config.ssh_passphrase.as_deref(),
        )
        .map_err(|e| format!("SSH key authentication failed: {}", e))?;
    } else {
        return Err("No SSH authentication method provided".to_string());
    }

    let mut channel = sess
        .channel_direct_tcpip(config.remote_host.as_str(), config.remote_port, None)
        .map_err(|e| format!("Failed to create SSH channel: {}", e))?;

    local_stream.set_nonblocking(false).ok();

    let mut buf = [0u8; 8192];

    while running.load(Ordering::SeqCst) {
        let mut read_done = false;

        match local_stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                channel
                    .write_all(&buf[..n])
                    .map_err(|e| format!("Failed to write to SSH channel: {}", e))?;
                read_done = true;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                tracing::debug!("Local stream read error: {}", e);
                break;
            }
        }

        match channel.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                local_stream
                    .write_all(&buf[..n])
                    .map_err(|e| format!("Failed to write to local stream: {}", e))?;
                read_done = true;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                tracing::debug!("SSH channel read error: {}", e);
                break;
            }
        }

        if !read_done {
            thread::sleep(Duration::from_millis(10));
        }
    }

    let _ = channel.send_eof();
    Ok(())
}
