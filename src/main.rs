use std::process::{exit, Command, ExitStatus};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs, io};

const EXTRA_CONFIG: &str = include_str!("init.lua");

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

fn main() {
    let not_rand = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let tmpname = format!("gh-xplr.{}", not_rand);
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
    let mut version = "".to_string();

    if rc == 0 {
        let output = Command::new("xplr").arg("--version").output();
        if let Ok(out) = output {
            version = out
                .stdout
                .into_iter()
                .map(char::from)
                .collect::<String>()
                .replace("xplr ", "")
                .trim()
                .to_string();

            rc = returncode(io::Result::Ok(out.status));
        };
    }

    if rc == 0 {
        let extra_config_path = tmpdir.join(".git").join("xplr.lua");
        let extra_config = format!(r#"version = "{}"{}"#, version, EXTRA_CONFIG);

        if let Err(e) = fs::write(&extra_config_path, extra_config) {
            rc = 2;
            eprintln!("error: {}", e.to_string());
        } else {
            let status = Command::new("xplr")
                .arg("--extra-config")
                .arg(extra_config_path)
                .arg("--vroot")
                .arg(&tmpdir)
                .arg("--")
                .arg(&tmpdir)
                .status();

            rc = returncode(status)
        }

        if let Err(e) = fs::remove_dir_all(tmpdir) {
            rc = 2;
            eprintln!("error: {}", e.to_string());
        }
    }

    exit(rc);
}
