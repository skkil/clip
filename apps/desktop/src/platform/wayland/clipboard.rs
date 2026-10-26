use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use wl_clipboard_rs::copy::{MimeSource, MimeType, Options, ServeRequests, Source};

use crate::platform::ClipboardItem;

pub fn copy(items: &[ClipboardItem]) -> Result<()> {
    if items.is_empty() {
        bail!("nothing to copy");
    }
    let exe = std::env::current_exe().context("locate own executable")?;
    let mut child = Command::new(exe)
        .arg(crate::cli::SERVE_CLIPBOARD)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .process_group(0)
        .spawn()
        .context("spawn clipboard server")?;
    child
        .stdin
        .take()
        .context("clipboard server has no stdin")?
        .write_all(&encode(items))
        .context("hand payload to clipboard server")
}

pub fn serve_from_stdin() -> Result<()> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .read_to_end(&mut bytes)
        .context("read payload")?;
    let sources = decode(&bytes)?
        .into_iter()
        .map(|item| MimeSource {
            mime_type: if item.mime.starts_with("text/plain") {
                MimeType::Text
            } else {
                MimeType::Specific(item.mime)
            },
            source: Source::Bytes(item.data.into()),
        })
        .collect();

    let mut opts = Options::new();
    opts.foreground(true);
    opts.serve_requests(ServeRequests::Unlimited);
    opts.copy_multi(sources).context("serve wayland selection")
}

fn encode(items: &[ClipboardItem]) -> Vec<u8> {
    let mut out = Vec::new();
    for item in items {
        out.extend_from_slice(&(item.mime.len() as u32).to_le_bytes());
        out.extend_from_slice(item.mime.as_bytes());
        out.extend_from_slice(&(item.data.len() as u64).to_le_bytes());
        out.extend_from_slice(&item.data);
    }
    out
}

fn decode(mut bytes: &[u8]) -> Result<Vec<ClipboardItem>> {
    fn take<'a>(bytes: &mut &'a [u8], n: usize) -> Result<&'a [u8]> {
        if bytes.len() < n {
            bail!("truncated clipboard payload");
        }
        let (head, tail) = bytes.split_at(n);
        *bytes = tail;
        Ok(head)
    }

    let mut items = Vec::new();
    while !bytes.is_empty() {
        let mime_len = u32::from_le_bytes(take(&mut bytes, 4)?.try_into()?) as usize;
        let mime = String::from_utf8(take(&mut bytes, mime_len)?.to_vec())?;
        let data_len = u64::from_le_bytes(take(&mut bytes, 8)?.try_into()?) as usize;
        let data = take(&mut bytes, data_len)?.to_vec();
        items.push(ClipboardItem { mime, data });
    }
    if items.is_empty() {
        bail!("empty clipboard payload");
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_round_trips() {
        let items = vec![
            ClipboardItem::new("text/plain;charset=utf-8", "안녕"),
            ClipboardItem::new("image/png", vec![1, 2, 3]),
        ];
        assert_eq!(decode(&encode(&items)).unwrap(), items);
    }

    #[test]
    fn truncated_payload_is_rejected() {
        let mut bytes = encode(&[ClipboardItem::new("image/png", vec![1, 2, 3])]);
        bytes.pop();
        assert!(decode(&bytes).is_err());
    }
}
