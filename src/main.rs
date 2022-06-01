use std::path::PathBuf;
use std::process::{exit, id as pid, Command, ExitStatus};
use std::{env, fs, io};

const EXTRA_CONFIG: &str = r###"
xplr.config.modes.builtin.default.key_bindings.on_key.enter = {
  help = "browse",
  messages = {
    {
      BashExecSilently = [===[
        basename=$(basename "$XPLR_FOCUS_PATH")
        if [ -e "$basename" ]; then
            gh browse "$basename"
            url=$(gh browse -n "$basename")
            echo "LogSuccess: $url" >> "${XPLR_PIPE_MSG_IN}"
        else
            gh browse .
            url=$(gh browse -n .)
            echo "LogSuccess: $url" >> "${XPLR_PIPE_MSG_IN}"
        fi
      ]===]
    },
  },
}
"###;

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

fn explore(repo: PathBuf, extra_config_path: PathBuf) -> i32 {
    let mut cli = xplr::cli::Cli::default();
    cli.paths.push(repo.into());
    cli.extra_config.push(extra_config_path.into());

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
    let tmpdir = env::temp_dir().join("gh-xplr").join(pid().to_string());

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
        let extra_config_path = tmpdir.join(".git").join("xplr.lua");
        if let Err(e) = fs::write(&extra_config_path, EXTRA_CONFIG) {
            rc = 2;
            eprintln!("error: {}", e.to_string());
        } else {
            rc = explore(tmpdir.clone(), extra_config_path);
        }
    }

    if let Err(e) = fs::remove_dir_all(tmpdir) {
        rc = 2;
        eprintln!("error: {}", e.to_string());
    }

    exit(rc);
}
