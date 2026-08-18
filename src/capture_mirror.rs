use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::Framebuffer;

const MIRROR_ENV: &str = "GPE_OBS_MIRROR";
const DEFAULT_ADDRESS: &str = "127.0.0.1:7878";

pub(crate) struct CaptureMirror {
    latest_rgba: Arc<Mutex<Vec<u8>>>,
}

impl CaptureMirror {
    pub(crate) fn from_env(width: u32, height: u32) -> Option<Self> {
        let raw = std::env::var(MIRROR_ENV).ok()?;
        let value = raw.trim();
        let normalized = value.to_ascii_lowercase();

        if value.is_empty() || matches!(normalized.as_str(), "0" | "false" | "off" | "no") {
            return None;
        }

        let address = if matches!(normalized.as_str(), "1" | "true" | "on" | "yes") {
            DEFAULT_ADDRESS.to_string()
        } else if value.parse::<u16>().is_ok() {
            format!("127.0.0.1:{value}")
        } else {
            value.to_string()
        };

        match Self::start(width, height, &address) {
            Ok(mirror) => {
                eprintln!("GPE OBS mirror: http://{address}/");
                Some(mirror)
            }
            Err(err) => {
                eprintln!("GPE OBS mirror disabled: {err}");
                None
            }
        }
    }

    fn start(width: u32, height: u32, address: &str) -> io::Result<Self> {
        let frame_len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "framebuffer is too large"))?;
        let listener = TcpListener::bind(address)?;
        let latest_rgba = Arc::new(Mutex::new(vec![0; frame_len]));
        let server_frame = Arc::clone(&latest_rgba);

        thread::Builder::new()
            .name("gpe-obs-mirror".to_string())
            .spawn(move || serve(listener, server_frame, width, height))?;

        Ok(Self { latest_rgba })
    }

    pub(crate) fn publish(&self, framebuffer: &Framebuffer) {
        let Ok(mut latest) = self.latest_rgba.try_lock() else {
            return;
        };
        latest.copy_from_slice(framebuffer.as_rgba8());
    }
}

fn serve(listener: TcpListener, latest_rgba: Arc<Mutex<Vec<u8>>>, width: u32, height: u32) {
    for mut stream in listener.incoming().flatten() {
        let _ = serve_connection(&mut stream, &latest_rgba, width, height);
    }
}

fn serve_connection(
    stream: &mut TcpStream,
    latest_rgba: &Mutex<Vec<u8>>,
    width: u32,
    height: u32,
) -> io::Result<()> {
    let mut request = [0_u8; 2048];
    let read = stream.read(&mut request)?;
    if read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&request[..read]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|path| path.split('?').next())
        .unwrap_or("/");

    match path {
        "/" => {
            let page = mirror_page(width, height);
            write_response(stream, "200 OK", "text/html; charset=utf-8", page.as_bytes())
        }
        "/frame.rgba" => {
            let frame = latest_rgba
                .lock()
                .map(|pixels| pixels.clone())
                .unwrap_or_default();
            write_response(stream, "200 OK", "application/octet-stream", &frame)
        }
        "/health" => write_response(stream, "200 OK", "text/plain; charset=utf-8", b"ok\n"),
        _ => write_response(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            b"not found\n",
        ),
    }
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store, no-cache, must-revalidate\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)
}

fn mirror_page(width: u32, height: u32) -> String {
    const TEMPLATE: &str = r#"<!doctype html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<style>
html,body{margin:0;width:100%;height:100%;overflow:hidden;background:#000}
canvas{display:block;width:100vw;height:100vh;image-rendering:pixelated;image-rendering:crisp-edges}
</style>
</head>
<body>
<canvas id="game" width="__WIDTH__" height="__HEIGHT__"></canvas>
<script>
const width=__WIDTH__, height=__HEIGHT__;
const canvas=document.getElementById('game');
const ctx=canvas.getContext('2d',{alpha:false});
let sequence=0;
async function frame(){
  try {
    const response=await fetch('/frame.rgba?'+(++sequence),{cache:'no-store'});
    const bytes=new Uint8ClampedArray(await response.arrayBuffer());
    if(bytes.length===width*height*4){
      ctx.putImageData(new ImageData(bytes,width,height),0,0);
    }
  } catch (_) {}
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
</script>
</body>
</html>
"#;

    TEMPLATE
        .replace("__WIDTH__", &width.to_string())
        .replace("__HEIGHT__", &height.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirror_page_embeds_framebuffer_dimensions() {
        let page = mirror_page(180, 320);
        assert!(page.contains("width=\"180\""));
        assert!(page.contains("height=\"320\""));
        assert!(page.contains("const width=180, height=320"));
    }
}
