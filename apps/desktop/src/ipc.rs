use std::io::{BufRead, BufReader, Write};

use anyhow::{Context, Result, bail};
use interprocess::local_socket::traits::{ListenerExt, Stream as _};
use interprocess::local_socket::{Listener, ListenerOptions, Name, Stream};
use serde::{Deserialize, Serialize};

use crate::app;

const SOCKET: &str = "clip-desktop.sock";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    Ping,
    Status,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum Response {
    Pong,
    Status(Status),
    Error { message: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub pid: u32,
    pub version: String,
    pub shortcuts: bool,
    pub activations: u64,
}

#[cfg(unix)]
pub fn socket_path() -> Result<std::path::PathBuf> {
    Ok(app::runtime_dir()?.join(SOCKET))
}

fn endpoint() -> Result<Name<'static>> {
    #[cfg(unix)]
    {
        use interprocess::local_socket::{GenericFilePath, ToFsName};
        Ok(socket_path()?.to_fs_name::<GenericFilePath>()?)
    }
    #[cfg(windows)]
    {
        use interprocess::local_socket::{GenericNamespaced, ToNsName};
        Ok(SOCKET.to_ns_name::<GenericNamespaced>()?)
    }
}

pub fn send(request: &Request) -> Result<Response> {
    let stream = Stream::connect(endpoint()?).context("no daemon is running")?;
    let mut line = serde_json::to_string(request)?;
    line.push('\n');
    (&stream)
        .write_all(line.as_bytes())
        .context("send request")?;

    let mut reply = String::new();
    BufReader::new(&stream)
        .read_line(&mut reply)
        .context("read response")?;
    serde_json::from_str(&reply).context("decode response")
}

pub fn bind() -> Result<Listener> {
    match ListenerOptions::new().name(endpoint()?).create_sync() {
        Ok(listener) => Ok(listener),
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            if matches!(send(&Request::Ping), Ok(Response::Pong)) {
                bail!("another {} daemon is already running", app::NAME);
            }
            #[cfg(unix)]
            std::fs::remove_file(socket_path()?).context("remove stale socket")?;
            Ok(ListenerOptions::new().name(endpoint()?).create_sync()?)
        }
        Err(e) => Err(e).context("bind control socket"),
    }
}

pub fn serve(listener: Listener, handler: impl Fn(Request) -> Response) {
    for stream in listener.incoming() {
        let result = stream
            .map_err(anyhow::Error::from)
            .and_then(|stream| serve_one(&stream, &handler));
        if let Err(e) = result {
            eprintln!("{}: control connection failed: {e:#}", app::NAME);
        }
    }
}

fn serve_one(stream: &Stream, handler: &impl Fn(Request) -> Response) -> Result<()> {
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .context("read request")?;
    let response = match serde_json::from_str(&line) {
        Ok(request) => handler(request),
        Err(e) => Response::Error {
            message: format!("malformed request: {e}"),
        },
    };
    let mut out = serde_json::to_string(&response)?;
    out.push('\n');
    let mut writer = stream;
    writer.write_all(out.as_bytes()).context("send response")
}
