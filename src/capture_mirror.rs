use std::io::{self, Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::Framebuffer;
use crate::platform::{Frame, Game, GameResult};

const MIRROR_ENV: &str = "GPE_OBS_MIRROR";
const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:7878";

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
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "framebuffer is too large"))?;
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
#status{position:fixed;left:8px;top:8px;padding:5px 7px;background:#250018;color:#fff;font:12px monospace;z-index:2}
</style>
</head>
<body>
<div id="status">GPE OBS MIRROR: WAITING FOR FRAME</div>
<canvas id="game" width="__WIDTH__" height="__HEIGHT__"></canvas>
<script>
const width=__WIDTH__, height=__HEIGHT__;
const canvas=document.getElementById('game');
const ctx=canvas.getContext('2d',{alpha:false});
const status=document.getElementById('status');
let sequence=0;
let received=false;
async function frame(){
  try {
    const response=await fetch('/frame.rgba?'+(++sequence),{cache:'no-store'});
    if(!response.ok) throw new Error('HTTP '+response.status);
    const bytes=new Uint8ClampedArray(await response.arrayBuffer());
    if(bytes.length===width*height*4){
      ctx.putImageData(new ImageData(bytes,width,height),0,0);
      if(!received){
        received=true;
        status.style.display='none';
      }
    } else {
      status.textContent='GPE OBS MIRROR: BAD FRAME SIZE '+bytes.length;
    }
  } catch (error) {
    status.style.display='block';
    status.textContent='GPE OBS MIRROR: FRAME LINK FAILED';
  }
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

    #[test]
    fn default_bind_accepts_windows_to_wsl_connections() {
        assert_eq!(DEFAULT_BIND_ADDRESS, "0.0.0.0:7878");
    }
}