use std::process::{exit, id as pid, Command, ExitStatus};
use std::{env, fs, io};

const EXTRA_CONFIG: &str = r###"
xplr.config.modes.builtin.default.key_bindings.on_key.enter = {
  help = "browse",
  messages = {
    {
      BashExecSilently = [===[
        while read -r path; do

          dirname=$(dirname "$path")
          basename=$(basename "$path")

          cd "${dirname:?}"

          if [ -e "$basename" ]; then
              gh browse "$basename"
              url=$(gh browse -n "$basename")
              echo "LogSuccess: $url" >> "${XPLR_PIPE_MSG_IN}"
          else
              gh browse .
              url=$(gh browse -n .)
              echo "LogSuccess: $url" >> "${XPLR_PIPE_MSG_IN}"
          fi
        done < "$XPLR_PIPE_RESULT_OUT"
      ]===]
    },
    "ClearSelection",
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
            let status = Command::new("xplr")
                .arg(&tmpdir)
                .arg("--extra-config")
                .arg(extra_config_path)
                .status();

            rc = returncode(status)
        }
    }

    if let Err(e) = fs::remove_dir_all(tmpdir) {
        rc = 2;
        eprintln!("error: {}", e.to_string());
    }

    exit(rc);
}
