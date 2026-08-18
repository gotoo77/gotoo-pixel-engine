use std::io::{self, Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::Framebuffer;
use crate::platform::{Frame, Game, GameResult};

const MIRROR_ENV: &str = "GPE_OBS_MIRROR";
const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:7878";
const STREAM_FRAME_INTERVAL: Duration = Duration::from_millis(33);

pub struct ObsMirrorGame<G> {
    game: G,
    mirror: Option<CaptureMirror>,
}

impl<G> ObsMirrorGame<G> {
    pub fn from_env(game: G, width: u32, height: u32) -> Self {
        Self {
            game,
            mirror: CaptureMirror::from_env(width, height),
        }
    }
}

impl<G: Game> Game for ObsMirrorGame<G> {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.game.update(frame);
        if let Some(mirror) = &self.mirror {
            mirror.publish(frame.framebuffer);
        }
        result
    }
}

struct CaptureMirror {
    latest_rgba: Arc<Mutex<Vec<u8>>>,
}

impl CaptureMirror {
    fn from_env(width: u32, height: u32) -> Option<Self> {
        let raw = std::env::var(MIRROR_ENV).ok()?;
        let value = raw.trim();
        let normalized = value.to_ascii_lowercase();

        if value.is_empty() || matches!(normalized.as_str(), "0" | "false" | "off" | "no") {
            return None;
        }

        let address = if matches!(normalized.as_str(), "1" | "true" | "on" | "yes") {
            DEFAULT_BIND_ADDRESS.to_string()
        } else if value.parse::<u16>().is_ok() {
            format!("0.0.0.0:{value}")
        } else {
            value.to_string()
        };

        match Self::start(width, height, &address) {
            Ok((mirror, port)) => {
                eprintln!("GPE OBS mirror listening on {address}");
                eprintln!("GPE OBS mirror Windows URL: http://localhost:{port}/");
                if let Some(ip) = guest_ip() {
                    eprintln!("GPE OBS mirror direct WSL URL: http://{ip}:{port}/");
                }
                eprintln!("GPE OBS mirror health: http://localhost:{port}/health");
                Some(mirror)
            }
            Err(err) => {
                eprintln!("GPE OBS mirror disabled: {err}");
                None
            }
        }
    }

    fn start(width: u32, height: u32, address: &str) -> io::Result<(Self, u16)> {
        let frame_len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "framebuffer is too large")
            })?;
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let latest_rgba = Arc::new(Mutex::new(vec![0; frame_len]));
        let server_frame = Arc::clone(&latest_rgba);

        thread::Builder::new()
            .name("gpe-obs-mirror".to_string())
            .spawn(move || serve(listener, server_frame, width, height))?;

        Ok((Self { latest_rgba }, port))
    }

    fn publish(&self, framebuffer: &Framebuffer) {
        let Ok(mut latest) = self.latest_rgba.try_lock() else {
            return;
        };
        latest.copy_from_slice(framebuffer.as_rgba8());
    }
}

fn guest_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("1.1.1.1:80").ok()?;
    let ip = socket.local_addr().ok()?.ip();
    (!ip.is_loopback() && !ip.is_unspecified()).then_some(ip)
}

fn serve(listener: TcpListener, latest_rgba: Arc<Mutex<Vec<u8>>>, width: u32, height: u32) {
    for stream in listener.incoming().flatten() {
        let client_frame = Arc::clone(&latest_rgba);
        let _ = thread::Builder::new()
            .name("gpe-obs-client".to_string())
            .spawn(move || {
                let mut stream = stream;
                let _ = serve_connection(&mut stream, &client_frame, width, height);
            });
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
            write_response(
                stream,
                "200 OK",
                "text/html; charset=utf-8",
                page.as_bytes(),
            )
        }
        "/stream.rgba" => stream_frames(stream, latest_rgba),
        "/frame.rgba" => {
            let frame = snapshot(latest_rgba);
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

fn snapshot(latest_rgba: &Mutex<Vec<u8>>) -> Vec<u8> {
    latest_rgba
        .lock()
        .map(|pixels| pixels.clone())
        .unwrap_or_default()
}

fn stream_frames(stream: &mut TcpStream, latest_rgba: &Mutex<Vec<u8>>) -> io::Result<()> {
    stream.write_all(
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nCache-Control: no-store, no-cache, must-revalidate\r\nConnection: close\r\n\r\n",
    )?;

    loop {
        let frame = snapshot(latest_rgba);
        stream.write_all(&frame)?;
        stream.flush()?;
        thread::sleep(STREAM_FRAME_INTERVAL);
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
body{display:flex;align-items:center;justify-content:center}
#stage{display:flex;align-items:center;justify-content:center;width:100vw;height:100vh;background:#000}
canvas{display:block;image-rendering:pixelated;image-rendering:crisp-edges;background:#000}
#status{position:fixed;left:8px;top:8px;padding:5px 7px;background:#250018;color:#fff;font:12px monospace;z-index:2}
</style>
</head>
<body>
<div id="stage"><canvas id="game" width="__WIDTH__" height="__HEIGHT__"></canvas></div>
<div id="status">GPE OBS MIRROR: CONNECTING</div>
<script>
const width=__WIDTH__, height=__HEIGHT__;
const frameSize=width*height*4;
const canvas=document.getElementById('game');
const ctx=canvas.getContext('2d',{alpha:false});
const status=document.getElementById('status');

function fitCanvas(){
  const fit=Math.min(window.innerWidth/width,window.innerHeight/height);
  const scale=fit>=1?Math.max(1,Math.floor(fit)):fit;
  canvas.style.width=(width*scale)+'px';
  canvas.style.height=(height*scale)+'px';
}

function showStatus(message){
  status.textContent=message;
  status.style.display='block';
}

function hideStatus(){
  status.style.display='none';
}

function drawFrame(bytes){
  ctx.putImageData(new ImageData(new Uint8ClampedArray(bytes),width,height),0,0);
  hideStatus();
}

async function streamFrames(){
  showStatus('GPE OBS MIRROR: CONNECTING');
  try {
    const response=await fetch('/stream.rgba',{cache:'no-store'});
    if(!response.ok||!response.body) throw new Error('stream unavailable');
    const reader=response.body.getReader();
    let pending=new Uint8Array(0);

    for(;;){
      const result=await reader.read();
      if(result.done) throw new Error('stream closed');

      const merged=new Uint8Array(pending.length+result.value.length);
      merged.set(pending,0);
      merged.set(result.value,pending.length);

      let offset=0;
      while(merged.length-offset>=frameSize){
        drawFrame(merged.slice(offset,offset+frameSize));
        offset+=frameSize;
      }
      pending=merged.slice(offset);
    }
  } catch (_) {
    showStatus('GPE OBS MIRROR: RECONNECTING');
    window.setTimeout(streamFrames,500);
  }
}

window.addEventListener('resize',fitCanvas);
fitCanvas();
streamFrames();
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

    #[test]
    fn mirror_page_preserves_aspect_ratio() {
        let page = mirror_page(180, 320);
        assert!(page.contains("Math.min(window.innerWidth/width,window.innerHeight/height)"));
        assert!(page.contains("canvas.style.width=(width*scale)+'px'"));
        assert!(page.contains("canvas.style.height=(height*scale)+'px'"));
    }

    #[test]
    fn mirror_page_uses_single_stream_connection() {
        let page = mirror_page(180, 320);
        assert!(page.contains("fetch('/stream.rgba'"));
        assert!(page.contains("response.body.getReader()"));
        assert!(!page.contains("requestAnimationFrame(frame)"));
    }

    #[test]
    fn default_bind_accepts_windows_to_wsl_connections() {
        assert_eq!(DEFAULT_BIND_ADDRESS, "0.0.0.0:7878");
    }
}
