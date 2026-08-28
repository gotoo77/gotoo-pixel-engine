use gotoo_pixel_engine::diagnostics_fault_probe::{SCENARIOS, run_child};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct Observation {
    status: ExitStatus,
    elapsed: Duration,
    timed_out: bool,
    controller_kill: bool,
    report: String,
    artifact: PathBuf,
    stdout: String,
    stderr: String,
}

fn main() {
    if let Err(error) = real_main() {
        eprintln!("diagnostics_fault_probe: {error}");
        std::process::exit(2);
    }
}

fn real_main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--child") => {
            let scenario = args.get(2).ok_or("missing child scenario")?;
            let artifact = args.get(3).ok_or("missing child artifact")?;
            run_child(scenario, Path::new(artifact))
        }
        Some("--scenario") => {
            let scenario = args.get(2).ok_or("missing scenario")?;
            validate_scenario(scenario)?;
            let output = output_argument(&args, 3)?;
            let observation = run_controller(scenario, &output)?;
            print_observation(scenario, &observation);
            if process_oracle(scenario, &observation) {
                Ok(())
            } else {
                Err(format!("{scenario} process oracle failed"))
            }
        }
        Some("--all") => {
            let output = output_argument(&args, 2)?;
            let mut failed = Vec::new();
            for scenario in SCENARIOS {
                let observation = run_controller(scenario, &output)?;
                print_observation(scenario, &observation);
                if !process_oracle(scenario, &observation) {
                    failed.push(scenario);
                }
            }
            if failed.is_empty() {
                Ok(())
            } else {
                Err(format!("process oracles failed: {}", failed.join(",")))
            }
        }
        _ => Err(
            "usage: diagnostics_fault_probe --scenario Fxx [--output DIR] | --all [--output DIR]"
                .to_string(),
        ),
    }
}

fn output_argument(args: &[String], start: usize) -> Result<PathBuf, String> {
    if args.get(start).map(String::as_str) == Some("--output") {
        return args
            .get(start + 1)
            .map(PathBuf::from)
            .ok_or_else(|| "missing --output directory".to_string());
    }
    Ok(PathBuf::from("target/diagnostics-fault-campaign"))
}

fn validate_scenario(scenario: &str) -> Result<(), String> {
    if SCENARIOS.contains(&scenario) {
        Ok(())
    } else {
        Err(format!("unknown scenario {scenario}"))
    }
}

fn run_controller(scenario: &str, output: &Path) -> Result<Observation, String> {
    fs::create_dir_all(output).map_err(|error| error.to_string())?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let run_dir = output.join(format!("{scenario}-{}-{nonce}", std::process::id()));
    fs::create_dir(&run_dir).map_err(|error| error.to_string())?;
    let artifact = run_dir.join("report.txt");
    let executable = env::current_exe().map_err(|error| error.to_string())?;
    let mut child = Command::new(executable)
        .arg("--child")
        .arg(scenario)
        .arg(&artifact)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let started = Instant::now();
    let timeout = scenario_timeout(scenario);
    let mut timed_out = false;
    let mut controller_kill = false;

    loop {
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            break;
        }
        if scenario == "F08" && artifact.exists() {
            child.kill().map_err(|error| error.to_string())?;
            controller_kill = true;
            break;
        }
        if started.elapsed() >= timeout {
            child.kill().map_err(|error| error.to_string())?;
            timed_out = true;
            controller_kill = true;
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    let output = child
        .wait_with_output()
        .map_err(|error| error.to_string())?;
    let report = fs::read_to_string(&artifact).unwrap_or_default();
    Ok(Observation {
        status: output.status,
        elapsed: started.elapsed(),
        timed_out,
        controller_kill,
        report,
        artifact,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn scenario_timeout(scenario: &str) -> Duration {
    match scenario {
        "F09" | "F10" => Duration::from_secs(1),
        "F17" => Duration::from_secs(15),
        "F18" => Duration::from_secs(10),
        _ => Duration::from_secs(5),
    }
}

fn report_kind(scenario: &str, report: &str) -> &'static str {
    if report.ends_with(&format!("END REPORT {scenario}\n")) {
        "successful"
    } else if report.is_empty() {
        "missing"
    } else {
        "partial"
    }
}

fn process_oracle(scenario: &str, observation: &Observation) -> bool {
    let successful = observation.status.success();
    let report = report_kind(scenario, &observation.report);
    match scenario {
        "F01" => !successful && !observation.timed_out && report == "successful",
        "F02" | "F03" | "F04" | "F11" | "F12" | "F13" | "F14" | "F15" | "F16" | "F17" | "F18"
        | "F19" | "F20" | "F21" | "F22" => {
            successful && !observation.controller_kill && report == "successful"
        }
        "F05" => !successful && !observation.timed_out && report != "successful",
        "F06" => !successful && !observation.timed_out && report == "successful",
        "F07" => !successful && !observation.timed_out && report == "partial",
        "F08" => {
            !successful
                && observation.controller_kill
                && !observation.timed_out
                && report == "partial"
        }
        "F09" | "F10" => {
            !successful
                && observation.controller_kill
                && observation.timed_out
                && report == "partial"
        }
        _ => false,
    }
}

fn print_observation(scenario: &str, observation: &Observation) {
    println!("=== {scenario} ===");
    println!("exit_success={}", observation.status.success());
    println!("exit_code={:?}", observation.status.code());
    println!("elapsed_ms={}", observation.elapsed.as_millis());
    println!("timed_out={}", observation.timed_out);
    println!("controller_kill={}", observation.controller_kill);
    println!("report_kind={}", report_kind(scenario, &observation.report));
    println!("artifact={}", observation.artifact.display());
    println!("process_oracle={}", process_oracle(scenario, observation));
    if !observation.stdout.is_empty() {
        println!("child_stdout={:?}", observation.stdout);
    }
    if !observation.stderr.is_empty() {
        println!("child_stderr={:?}", observation.stderr);
    }
}
