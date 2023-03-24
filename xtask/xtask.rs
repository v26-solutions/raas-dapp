use anyhow::Result;
use xshell::{cmd, Shell};

pub fn coverage(sh: &Shell) -> Result<()> {
    cmd!(sh, "cargo llvm-cov --html").run()?;

    Ok(())
}

pub fn test(sh: &Shell, update: bool, backtrace: bool) -> Result<()> {
    let mut cmd = cmd!(sh, "cargo test --package it");

    if update {
        cmd = cmd.env("UPDATE_EXPECT", "1");
    }

    if backtrace {
        cmd = cmd.env("RUST_BACKTRACE", "1");
    }

    cmd.run()?;

    Ok(())
}

pub fn dist(sh: &Shell) -> Result<()> {
    let cwd = sh.current_dir();
    let cwd_name = cwd.file_stem().unwrap();
    let cwd_path = cwd.as_path();

    cmd!(
        sh,
        "docker run --rm -v {cwd_path}:/code
          --mount type=volume,source={cwd_name}_cache,target=/code/target
          --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry
          cosmwasm/workspace-optimizer:0.12.10"
    )
    .run()?;

    Ok(())
}

pub fn dev(sh: &Shell, update: bool) -> Result<()> {
    let update = update.then_some("--update");

    cmd!(
        sh,
        "cargo watch -- cargo xtask test {update...} --backtrace"
    )
    .run()?;

    Ok(())
}

pub fn install(sh: &Shell) -> Result<()> {
    cmd!(sh, "rustup component add llvm-tools-preview").run()?;

    cmd!(
        sh,
        "cargo install
            cargo-watch
            cargo-llvm-cov"
    )
    .run()?;

    Ok(())
}

pub fn artifacts_dir() -> String {
    dotenv::var("ARTIFACTS_DIR").unwrap_or_else(|_| "artifacts".to_owned())
}

pub mod archway {
    use std::{
        io::{BufRead, BufReader},
        path::PathBuf,
        time,
    };

    use anyhow::{anyhow, Result};
    use referrals_cw::InstantiateMsg;
    use serde::Serialize;
    use serde_json::{
        from_slice as from_json_bytes, from_str as from_json_str, Value as JsonValue,
    };
    use xshell::{cmd, Cmd, Shell};

    pub const IMAGE_NAME: &str = "archwayd-xtask";
    pub const CONTAINER_NAME: &str = "local_archwayd_xtask";

    pub fn archwayd_repo_url() -> String {
        dotenv::var("ARCHWAY_REPO_URL")
            .unwrap_or_else(|_| "https://github.com/archway-network/archway".to_owned())
    }

    pub fn archwayd_repo_branch() -> String {
        dotenv::var("ARCHWAY_REPO_BRANCH").unwrap_or_else(|_| "main".to_owned())
    }

    pub fn archwayd_repo_dir() -> String {
        dotenv::var("ARCHWAY_REPO_DIR").unwrap_or_else(|_| "target/chains/archwayd".to_owned())
    }

    pub fn archwayd_home_dir() -> String {
        dotenv::var("ARCHWAY_HOME_DIR").unwrap_or_else(|_| "target/chains".to_owned())
    }

    pub fn absolute_path(sh: &Shell, f: impl FnOnce() -> String) -> PathBuf {
        let mut cwd = sh.current_dir();
        cwd.push(f());
        cwd
    }

    pub fn clone_archwayd_repo(sh: &Shell) -> Result<()> {
        let url = archwayd_repo_url();
        let branch = archwayd_repo_branch();
        let dir = archwayd_repo_dir();

        cmd!(sh, "git clone {url} --depth 1 --branch {branch} {dir}").run()?;

        Ok(())
    }

    pub fn build_archwayd_docker(sh: &Shell) -> Result<()> {
        let dir = archwayd_repo_dir();

        cmd!(sh, "docker build {dir} --tag {IMAGE_NAME}:latest").run()?;

        Ok(())
    }

    pub fn start_local(sh: &Shell) -> Result<()> {
        let archwayd_repo_dir = archwayd_repo_dir();

        if !sh.path_exists(&archwayd_repo_dir) {
            clone_archwayd_repo(sh)?;
        }

        build_archwayd_docker(sh)?;

        let mut localnet_dir = absolute_path(sh, || archwayd_repo_dir);
        localnet_dir.push("contrib/localnet");

        let localnet_dir = localnet_dir.display();

        let node_cmd = duct::cmd!(
            "docker",
            "run",
            "--name",
            CONTAINER_NAME,
            "--rm",
            "--volume",
            format!("{localnet_dir}:/opt"),
            "--volume",
            format!("{}:/root", absolute_path(sh, archwayd_home_dir).display()),
            "--publish",
            "9090:9090",
            "--publish",
            "26657:26657",
            "--entrypoint",
            "/opt/localnet.sh",
            format!("{IMAGE_NAME}:latest"),
        );

        let node_handle = node_cmd.stdout_to_stderr().unchecked().reader()?;

        let node_output_lines = BufReader::new(node_handle).lines();

        ctrlc::set_handler(|| {
            let sh = Shell::new().unwrap();
            cmd!(sh, "docker kill {CONTAINER_NAME}")
                .quiet()
                .ignore_stdout()
                .ignore_stderr()
                .run()
                .unwrap();
        })?;

        for line in node_output_lines {
            println!("{}", line?);
        }

        Ok(())
    }

    pub fn local_node_ip(sh: &Shell) -> Result<String> {
        let json_string = cmd!(sh, "docker inspect {CONTAINER_NAME}")
            .ignore_status()
            .ignore_stderr()
            .read()?;

        let json_value: JsonValue = from_json_str(&json_string)?;

        json_value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(JsonValue::as_object)
            .and_then(|obj| obj.get("NetworkSettings"))
            .and_then(JsonValue::as_object)
            .and_then(|obj| obj.get("IPAddress"))
            .and_then(JsonValue::as_str)
            .map(ToOwned::to_owned)
            .ok_or_else(|| anyhow!("Failed to find local node IP address"))
    }

    pub fn archwayd_cmd(sh: &Shell) -> Result<Cmd> {
        let mut artifacts_dir = sh.current_dir();
        artifacts_dir.push(crate::artifacts_dir());

        let home_dir = absolute_path(sh, archwayd_home_dir);

        let ip = local_node_ip(sh)?;

        Ok(cmd!(
            sh,
            "docker
            run
            --interactive
            --rm
            --volume {artifacts_dir}:/artifacts
            --volume {home_dir}:/root
            --entrypoint archwayd
            {IMAGE_NAME}:latest
            --node tcp://{ip}:26657"
        ))
    }

    pub fn run_cmd(cmd: Cmd) -> Result<JsonValue> {
        let out = cmd.ignore_status().output()?;

        if !out.status.success() {
            let err = std::str::from_utf8(&out.stderr)?;
            return Err(anyhow!("{err}"));
        }

        let json = from_json_bytes(&out.stdout)?;

        Ok(json)
    }

    pub fn send_tx(cmd: Cmd, gas: Option<u64>) -> Result<String> {
        let gas = gas.map_or_else(|| "auto".to_owned(), |g| g.to_string());

        let cmd = cmd.arg("--gas").arg(gas).args([
            "--from",
            "fd",
            "--yes",
            "--keyring-backend",
            "test",
            "--chain-id",
            "localnet",
            "--output",
            "json",
        ]);

        let tx_res_obj = run_cmd(cmd)?
            .as_object()
            .ok_or_else(|| anyhow!("expected json object"))?
            .to_owned();

        let code = tx_res_obj
            .get("code")
            .and_then(JsonValue::as_u64)
            .ok_or_else(|| anyhow!("code field missing in send tx json response"))?;

        if code > 0 {
            let raw_log = tx_res_obj
                .get("raw_log")
                .and_then(JsonValue::as_str)
                .ok_or_else(|| anyhow!("raw_log field missing in send tx json response"))?;

            return Err(anyhow!("Sending TX failed: {raw_log}"));
        }

        let tx_hash = tx_res_obj
            .get("txhash")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| anyhow!("txhash field missing in send tx json response"))?;

        Ok(tx_hash.to_owned())
    }

    pub fn query_tx(sh: &Shell, hash: &str) -> Result<Option<JsonValue>> {
        let cmd = archwayd_cmd(sh)?.args(["query", "tx", hash, "--output", "json"]);

        match run_cmd(cmd) {
            Ok(json) => Ok(Some(json)),
            Err(err) => {
                if err.to_string().contains("not found") {
                    return Ok(None);
                }

                Err(err)
            }
        }
    }

    // round-trip
    pub fn execute_tx(sh: &Shell, cmd: Cmd, gas: Option<u64>) -> Result<JsonValue> {
        let tx_hash = send_tx(cmd, gas)?;
        loop {
            let Some(json) = query_tx(sh, &tx_hash)? else {
                    std::thread::sleep(time::Duration::from_secs(1));
                    continue;
                };

            return Ok(json);
        }
    }

    pub fn store_contract(sh: &Shell, path: &str) -> Result<u64> {
        let cmd = archwayd_cmd(sh)?.args(["tx", "wasm", "store", path]);
        let json = execute_tx(sh, cmd, None)?;

        let code_id = json
            .as_object()
            .and_then(|o| o.get("logs"))
            .and_then(JsonValue::as_array)
            .and_then(|arr| arr.first())
            .and_then(JsonValue::as_object)
            .and_then(|o| o.get("events"))
            .and_then(JsonValue::as_array)
            .into_iter()
            .flatten()
            .filter_map(JsonValue::as_object)
            .filter_map(|o| o.get("attributes"))
            .flat_map(JsonValue::as_array)
            .flatten()
            .filter_map(JsonValue::as_object)
            .filter(|o| matches!(o.get("key").and_then(JsonValue::as_str), Some("code_id")))
            .find_map(|o| o.get("value").and_then(JsonValue::as_str))
            .ok_or_else(|| anyhow!("expected code_id attribute"))?
            .parse()?;

        Ok(code_id)
    }

    pub fn init_contract<Msg>(sh: &Shell, code_id: u64, name: &str, msg: Msg) -> Result<String>
    where
        Msg: Serialize,
    {
        let timestamp = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)?
            .as_secs();

        let label = format!("{name}:{timestamp}");

        let msg = serde_json::to_string(&msg)?;

        let cmd = archwayd_cmd(sh)?.args([
            "tx",
            "wasm",
            "init",
            &code_id.to_string(),
            msg.as_str(),
            "--label",
            label.as_str(),
            "--no-admin",
        ]);

        let json = execute_tx(sh, cmd, None)?;

        let addr = json
            .as_object()
            .and_then(|o| o.get("logs"))
            .and_then(JsonValue::as_array)
            .and_then(|arr| arr.first())
            .and_then(JsonValue::as_object)
            .and_then(|o| o.get("events"))
            .and_then(JsonValue::as_array)
            .into_iter()
            .flatten()
            .filter_map(JsonValue::as_object)
            .filter(|o| {
                matches!(
                    o.get("type").and_then(JsonValue::as_str),
                    Some("instantiate")
                )
            })
            .filter_map(|o| o.get("attributes"))
            .flat_map(JsonValue::as_array)
            .flatten()
            .filter_map(JsonValue::as_object)
            .filter(|o| {
                matches!(
                    o.get("key").and_then(JsonValue::as_str),
                    Some("_contract_address")
                )
            })
            .find_map(|o| o.get("value").and_then(JsonValue::as_str))
            .ok_or_else(|| anyhow!("expected _contract_address attribute"))?
            .to_owned();

        Ok(addr)
    }

    pub fn deploy_local(sh: &Shell) -> Result<()> {
        let hub_code_id = store_contract(sh, "/artifacts/archway_referrals_hub.wasm")?;
        let pot_code_id = store_contract(sh, "/artifacts/archway_referrals_rewards_pot.wasm")?;

        let hub_addr = init_contract(
            sh,
            hub_code_id,
            "referrals_hub",
            InstantiateMsg {
                rewards_pot_code_id: pot_code_id,
            },
        )?;

        println!("Referrals Hub Deployed at: {hub_addr}");

        Ok(())
    }

    pub fn clean(sh: &Shell) -> Result<()> {
        let dir = archwayd_repo_dir();
        sh.remove_path(dir)?;
        Ok(())
    }
}
