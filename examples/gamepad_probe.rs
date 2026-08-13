#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use std::thread;
    use std::time::Duration;

    use gilrs::{Event, Gilrs};

    println!("Gotoo Pixel Engine - gamepad probe");
    println!("Initialising gilrs...");

    let mut gilrs = match Gilrs::new() {
        Ok(gilrs) => gilrs,
        Err(err) => {
            eprintln!("ERROR: gilrs initialisation failed: {err}");
            std::process::exit(1);
        }
    };

    let connected = gilrs
        .gamepads()
        .map(|(id, gamepad)| (format!("{id:?}"), gamepad.name().to_owned()))
        .collect::<Vec<_>>();

    if connected.is_empty() {
        println!("No connected gamepad detected by gilrs.");
    } else {
        println!("Detected {} gamepad(s):", connected.len());
        for (id, name) in connected {
            println!("  {id}: {name}");
        }
    }

    println!("Move sticks / D-pad and press buttons. Ctrl-C to stop.");

    loop {
        while let Some(Event { id, event, .. }) = gilrs.next_event() {
            println!("{id:?}: {event:?}");
        }
        thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    compile_error!("gamepad_probe is a native-only diagnostic example");
}
