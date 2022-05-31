use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::ExitStatus;
use std::process::{exit, Command};
use std::time::{SystemTime, UNIX_EPOCH};

fn returncode(status: io::Result<ExitStatus>) -> i32 {
    match status {
        Ok(status) => {
            if let Some(code) = status.code() {
                code
            } else {
                eprintln!("error: inexpected error");
                130
            }
        }
        Err(e) => {
            eprintln!("error: {}", e.to_string());
            1
        }
    }
}

fn explore(repo: &Path) -> i32 {
    let mut cli = xplr::cli::Cli::default();
    cli.paths.push(repo.into());

    match xplr::runner::from_cli(cli).and_then(|app| app.run()) {
        Ok(Some(out)) => {
            print!("{}", out);
            0
        }
        Ok(None) => 0,
        Err(err) => {
            if !err.to_string().is_empty() {
                eprintln!("error: {}", err);
            };
            1
        }
    }
}

fn main() {
    let not_rand = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();

    let tmpname = format!("gh_xplr_{}", not_rand);
    let tmpdir = env::temp_dir().join(tmpname);

    let args = env::args().skip(1);
    let status = Command::new("gh")
        .arg("repo")
        .arg("clone")
        .args(args)
        .arg(&tmpdir)
        .arg("--")
        .arg("--depth")
        .arg("1")
        .status();

    let mut rc = returncode(status);

    if rc == 0 {
        rc = explore(&tmpdir);
    }

    if let Err(e) = fs::remove_dir_all(tmpdir) {
        rc = 2;
        eprintln!("error: {}", e.to_string());
    }

    exit(rc);
}
