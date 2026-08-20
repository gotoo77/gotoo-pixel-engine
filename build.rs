use std::process::Command;

fn main() {
    let build_id = git_build_id().unwrap_or_else(|| "UNKNOWN".to_string());
    println!("cargo:rustc-env=GPE_BUILD_ID={build_id}");
}

fn git_build_id() -> Option<String> {
    let sha = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()?;
    if !sha.status.success() {
        return None;
    }

    let sha = String::from_utf8(sha.stdout)
        .ok()?
        .trim()
        .to_ascii_uppercase();
    if sha.is_empty() {
        return None;
    }

    let dirty = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .is_some_and(|output| !output.stdout.is_empty());

    Some(if dirty { format!("{sha}*") } else { sha })
}
