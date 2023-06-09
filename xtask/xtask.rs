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
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
        io::{BufRead, BufReader},
        path::PathBuf,
        sync::atomic::{AtomicBool, Ordering},
        time,
    };

    use anyhow::{anyhow, Result};
    use bip39::Mnemonic;
    use nanorand::{Rng, WyRand};
    use referrals_cw::{
        DappResponse, ExecuteMsg, InstantiateMsg, QueryMsg, ReferralCodeResponse, WithReferralCode,
    };
    use serde::{de::DeserializeOwned, Serialize};
    use serde_json::{
        from_slice as from_json_bytes, from_str as from_json_str, Value as JsonValue,
    };
    use xshell::{cmd, Cmd, Shell};

    pub const IMAGE_NAME: &str = "archwayd-xtask";
    pub const CONTAINER_NAME: &str = "local_archwayd_xtask";

    static VERBOSE: AtomicBool = AtomicBool::new(false);

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

    pub fn archwayd_local_seed() -> String {
        dotenv::var("ARCHWAY_LOCAL_SEED").unwrap_or_else(|_| "v26-solutions".to_owned())
    }

    pub fn archwayd_local_n_accounts() -> usize {
        dotenv::var("ARCHWAY_LOCAL_N_ACCOUNTS")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(10)
            .max(1) // always at least one account
    }

    pub fn generate_n_mnemonics(seed: &str, n: usize) -> Vec<String> {
        let mut hasher = DefaultHasher::default();
        seed.hash(&mut hasher);
        let seed = hasher.finish();

        let mut rng = WyRand::new_seed(seed);

        let mut bytes = [0u8; 16];

        (0..n)
            .map(|_| {
                rng.fill_bytes(&mut bytes);
                Mnemonic::from_entropy(&bytes).unwrap().to_string()
            })
            .collect()
    }

    pub fn absolute_path(sh: &Shell, f: impl FnOnce() -> String) -> PathBuf {
        let mut cwd = sh.current_dir();
        cwd.push(f());
        cwd
    }

    pub fn container_cmd<'a>(sh: &'a Shell, entrypoint: &str) -> Cmd<'a> {
        let mut artifacts_dir = sh.current_dir();
        artifacts_dir.push(crate::artifacts_dir());

        let home_dir = absolute_path(sh, archwayd_home_dir);

        cmd!(
            sh,
            "docker
            run
            --interactive
            --rm
            --volume {artifacts_dir}:/artifacts
            --volume {home_dir}:/root
            --entrypoint {entrypoint}
            {IMAGE_NAME}:latest"
        )
    }

    pub fn sh_cmd(sh: &Shell) -> Cmd {
        container_cmd(sh, "/bin/sh")
    }

    pub fn archwayd_cmd(sh: &Shell) -> Cmd {
        container_cmd(sh, "archwayd")
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

    pub fn clear_chain(sh: &Shell) -> Result<()> {
        sh_cmd(sh)
            .args(["-c", "rm -rf /root/.archway"])
            .ignore_status()
            .ignore_stderr()
            .run()?;
        Ok(())
    }

    pub fn delete_account(sh: &Shell, account: &str) -> Result<()> {
        archwayd_cmd(sh)
            .args([
                "keys",
                "delete",
                account,
                "--yes",
                "--keyring-backend",
                "test",
            ])
            .ignore_status()
            .ignore_stdout()
            .ignore_stderr()
            .quiet()
            .run()?;
        Ok(())
    }

    pub fn add_account(sh: &Shell, account: &str, mnemonic: &str) -> Result<()> {
        archwayd_cmd(sh)
            .args([
                "keys",
                "add",
                account,
                "--recover",
                "--keyring-backend",
                "test",
            ])
            .stdin(format!("{mnemonic}\n"))
            .ignore_stdout()
            .ignore_stderr()
            .quiet()
            .run()?;
        Ok(())
    }

    pub fn account_address(sh: &Shell, account: &str) -> Result<String> {
        let out = archwayd_cmd(sh)
            .args([
                "keys",
                "show",
                account,
                "--keyring-backend",
                "test",
                "--output",
                "json",
            ])
            .output()?;

        let json: JsonValue = from_json_bytes(&out.stdout)?;

        json.as_object()
            .and_then(|o| o.get("address"))
            .and_then(JsonValue::as_str)
            .ok_or_else(|| anyhow!("expected address field"))
            .map(String::from)
    }

    pub fn print_mnemonics() -> Result<()> {
        let archwayd_local_seed = archwayd_local_seed();
        let archwayd_local_n_accounts = archwayd_local_n_accounts();
        let mnemonics = generate_n_mnemonics(&archwayd_local_seed, archwayd_local_n_accounts);
        for m in mnemonics {
            println!("{m}");
        }
        Ok(())
    }

    pub fn init_local(sh: &Shell) -> Result<()> {
        let archwayd_repo_dir = archwayd_repo_dir();
        let archwayd_local_seed = archwayd_local_seed();
        let archwayd_local_n_accounts = archwayd_local_n_accounts();

        if !sh.path_exists(archwayd_repo_dir) {
            clone_archwayd_repo(sh)?;
            build_archwayd_docker(sh)?;
        }

        clear_chain(sh)?;

        archwayd_cmd(sh)
            .args(["init", "archway-id", "--chain-id", "localnet"])
            .ignore_stderr()
            .ignore_stdout()
            .run()?;

        let mnemonics = generate_n_mnemonics(&archwayd_local_seed, archwayd_local_n_accounts);

        for (i, mnemonic) in mnemonics.iter().enumerate() {
            let account = format!("test_{i}");

            println!("\nAdding key {account}: {mnemonic}");
            add_account(sh, &account, mnemonic)?;

            let address = account_address(sh, &account)?;
            println!("{account} address: {address}");

            archwayd_cmd(sh)
                .args([
                    "add-genesis-account",
                    &account,
                    "1000000000000stake",
                    "--keyring-backend",
                    "test",
                ])
                .ignore_stderr()
                .ignore_stdout()
                .quiet()
                .run()?;
        }

        archwayd_cmd(sh)
            .args([
                "gentx",
                "test_0",
                "100000000stake",
                "--chain-id",
                "localnet",
                "--keyring-backend",
                "test",
            ])
            .ignore_stderr()
            .ignore_stdout()
            .run()?;

        archwayd_cmd(sh)
            .arg("collect-gentxs")
            .ignore_stderr()
            .ignore_stdout()
            .run()?;

        archwayd_cmd(sh)
            .arg("validate-genesis")
            .ignore_stderr()
            .ignore_stdout()
            .run()?;

        sh_cmd(sh)
            .args([
                "-c",
                "sed -i 's/127.0.0.1/0.0.0.0/g' /root/.archway/config/config.toml",
            ])
            .ignore_status()
            .ignore_stderr()
            .run()?;

        sh_cmd(sh)
            .args([
                "-c",
                r#"sed -i 's/cors_allowed_origins = \[\]/cors_allowed_origins = \["*"\]/g' target/chains/.archway/config/config.toml"#,
            ])
            .ignore_status()
            .ignore_stderr()
            .run()?;

        Ok(())
    }

    pub fn start_local(sh: &Shell) -> Result<()> {
        let node_cmd = duct::cmd!(
            "docker",
            "run",
            "--name",
            CONTAINER_NAME,
            "--rm",
            "--volume",
            format!("{}:/root", absolute_path(sh, archwayd_home_dir).display()),
            "--publish",
            "9090:9090",
            "--publish",
            "26657:26657",
            "--entrypoint",
            "archwayd",
            format!("{IMAGE_NAME}:latest"),
            "start"
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

    pub fn archwayd_node_cmd(sh: &Shell) -> Result<Cmd> {
        let ip = local_node_ip(sh)?;

        let cmd = archwayd_cmd(sh).args(["--node", &format!("tcp://{ip}:26657")]);

        Ok(cmd)
    }

    pub fn account_balance(sh: &Shell, address: &str) -> Result<u128> {
        let out = archwayd_node_cmd(sh)?
            .args(["query", "bank", "balances", address, "--output", "json"])
            .output()?;

        if !out.status.success() {
            let err = std::str::from_utf8(&out.stderr)?;
            return Err(anyhow!("{err}"));
        }

        let json: JsonValue = from_json_bytes(&out.stdout)?;

        let balance = json
            .as_object()
            .and_then(|o| o.get("balances"))
            .and_then(JsonValue::as_array)
            .and_then(|v| v.first())
            .and_then(JsonValue::as_object)
            .and_then(|o| o.get("amount"))
            .and_then(JsonValue::as_str)
            .ok_or_else(|| anyhow!("Expected at least one balance amount"))?
            .parse()?;

        Ok(balance)
    }

    pub fn run_cmd(cmd: Cmd) -> Result<JsonValue> {
        if VERBOSE.load(Ordering::Relaxed) {
            eprintln!("$ {cmd}");
        }

        let out = cmd.ignore_status().output()?;

        if !out.status.success() {
            let err = std::str::from_utf8(&out.stderr)?;
            return Err(anyhow!("{err}"));
        }

        let json = from_json_bytes(&out.stdout)?;

        Ok(json)
    }

    pub fn send_tx(cmd: Cmd, from: &str, gas: Option<u64>) -> Result<String> {
        let gas = gas.map_or_else(|| "auto".to_owned(), |g| g.to_string());

        let cmd = cmd.arg("--gas").arg(gas).args([
            "--from",
            from,
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
        let cmd = archwayd_node_cmd(sh)?.args(["query", "tx", hash, "--output", "json"]);

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
    pub fn execute_tx(sh: &Shell, cmd: Cmd, from: &str, gas: Option<u64>) -> Result<JsonValue> {
        let tx_hash = send_tx(cmd, from, gas)?;
        loop {
            let Some(json) = query_tx(sh, &tx_hash)? else {
                    std::thread::sleep(time::Duration::from_secs(1));
                    continue;
                };

            let tx_query = json
                .as_object()
                .ok_or_else(|| anyhow!("expected json object"))?;

            let status_code = tx_query
                .get("code")
                .and_then(JsonValue::as_u64)
                .ok_or_else(|| anyhow!("expected status code"))?;

            if status_code > 0 {
                let err = tx_query
                    .get("raw_log")
                    .and_then(JsonValue::as_str)
                    .ok_or_else(|| anyhow!("expected raw log"))?;

                return Err(anyhow!("Tx failed: {err}"));
            }

            return Ok(json);
        }
    }

    pub fn store_contract(sh: &Shell, from: &str, path: &str) -> Result<u64> {
        let cmd = archwayd_node_cmd(sh)?.args(["tx", "wasm", "store", path]);
        let json = execute_tx(sh, cmd, from, None)?;

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

    pub fn query_code_info(sh: &Shell, code_id: u64) -> Result<JsonValue> {
        let out = archwayd_node_cmd(sh)?
            .args([
                "query",
                "wasm",
                "code-info",
                &code_id.to_string(),
                "--output",
                "json",
            ])
            .ignore_status()
            .output()?;

        if !out.status.success() {
            let err = std::str::from_utf8(&out.stderr)?;
            return Err(anyhow!("{err}"));
        }

        let json = serde_json::from_slice(&out.stdout)?;

        Ok(json)
    }

    pub fn build_contract_addr(
        sh: &Shell,
        from: &str,
        code_id: u64,
        label: &str,
    ) -> Result<(String, String)> {
        let code_hash = query_code_info(sh, code_id)?
            .as_object()
            .and_then(|o| o.get("data_hash"))
            .and_then(JsonValue::as_str)
            .map(ToOwned::to_owned)
            .ok_or_else(|| anyhow!("expected json object with 'data_hash' field"))?;

        let from_address = account_address(sh, from)?;

        let salt = hex::encode(label);

        let out = archwayd_cmd(sh)
            .args([
                "query",
                "wasm",
                "build-address",
                &code_hash,
                &from_address,
                &salt,
            ])
            .ignore_status()
            .output()?;

        if !out.status.success() {
            let err = std::str::from_utf8(&out.stderr)?;
            return Err(anyhow!("{err}"));
        }

        let address = String::from_utf8(out.stdout)?
            .split_ascii_whitespace()
            .next()
            .unwrap()
            .to_owned();

        Ok((address, salt))
    }

    pub fn init_contract<Msg>(
        sh: &Shell,
        from: &str,
        code_id: u64,
        name: &str,
        msg: Msg,
    ) -> Result<String>
    where
        Msg: Serialize,
    {
        let timestamp = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)?
            .as_secs();

        let label = format!("{name}:{timestamp}");

        let (determined_address, salt) = build_contract_addr(sh, from, code_id, &label)?;

        let msg = serde_json::to_string(&msg)?;

        let cmd = archwayd_node_cmd(sh)?.args([
            "tx",
            "wasm",
            "instantiate2",
            &code_id.to_string(),
            msg.as_str(),
            salt.as_str(),
            "--label",
            label.as_str(),
            "--admin",
            determined_address.as_str(),
        ]);

        let json = execute_tx(sh, cmd, from, None)?;

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

        assert_eq!(addr, determined_address);

        Ok(addr)
    }

    pub fn exec_contract<Msg>(
        sh: &Shell,
        from: &str,
        address: &str,
        msg: Msg,
        gas: Option<u64>,
        premium: u128,
    ) -> Result<JsonValue>
    where
        Msg: Serialize,
    {
        let msg = serde_json::to_string(&msg)?;

        let cmd = archwayd_node_cmd(sh)?.args([
            "tx",
            "wasm",
            "execute",
            address,
            msg.as_str(),
            "--fees",
            &format!("{premium}stake"),
        ]);

        execute_tx(sh, cmd, from, gas)
    }

    pub fn query_contract<Msg, Response>(sh: &Shell, address: &str, msg: Msg) -> Result<Response>
    where
        Msg: Serialize,
        Response: DeserializeOwned,
    {
        let msg = serde_json::to_string(&msg)?;

        let out = archwayd_node_cmd(sh)?
            .args([
                "query",
                "wasm",
                "contract-state",
                "smart",
                address,
                msg.as_str(),
                "--output",
                "json",
            ])
            .ignore_status()
            .output()?;

        if !out.status.success() {
            let err = std::str::from_utf8(&out.stderr)?;
            return Err(anyhow!("{err}"));
        }

        let json: JsonValue = serde_json::from_slice(&out.stdout)?;

        let data = json
            .as_object()
            .and_then(|o| o.get("data"))
            .ok_or_else(|| anyhow!("expected json object with 'data' field"))?
            .to_owned();

        let res = serde_json::from_value(data)?;

        Ok(res)
    }

    pub fn deploy_local(sh: &Shell, verbose: bool) -> Result<()> {
        VERBOSE.store(verbose, Ordering::Relaxed);

        println!("Storing contracts...");

        let hub_code_id = store_contract(sh, "test_0", "/artifacts/archway_referrals_hub.wasm")?;
        let pot_code_id = store_contract(
            sh,
            "test_0",
            "/artifacts/archway_referrals_rewards_pot.wasm",
        )?;

        println!("Instantiating Referrals Hub...");

        let hub_addr = init_contract(
            sh,
            "test_0",
            hub_code_id,
            "referrals_hub",
            InstantiateMsg {
                rewards_pot_code_id: pot_code_id,
                contract_premium: 1000u128.into(),
            },
        )?;

        println!("Referrals Hub Deployed at: {hub_addr}");

        let test_0_address = account_address(sh, "test_0")?;

        println!("Referrals Hub Owner/Collector: {test_0_address}");

        println!("Registering 1st referral code...");

        exec_contract(
            sh,
            "test_1",
            &hub_addr,
            ExecuteMsg::RegisterReferrer {},
            Some(200_000),
            1000,
        )?;

        let test_1_address = account_address(sh, "test_1")?;

        let test_1_referral_code: ReferralCodeResponse = query_contract(
            sh,
            &hub_addr,
            QueryMsg::RefferalCode {
                referrer: test_1_address.clone(),
            },
        )?;

        let test_1_referral_code = test_1_referral_code.code;

        println!("Referral Code Registered: {test_1_address} => {test_1_referral_code}",);

        println!("Registering 2nd referral code (referrer set to {test_1_referral_code})...",);

        exec_contract(
            sh,
            "test_2",
            &hub_addr,
            WithReferralCode {
                referral_code: Some(test_1_referral_code),
                msg: ExecuteMsg::RegisterReferrer {},
            },
            Some(500_000),
            1000,
        )?;

        let test_2_address = account_address(sh, "test_2")?;

        let test_2_referral_code: ReferralCodeResponse = query_contract(
            sh,
            &hub_addr,
            QueryMsg::RefferalCode {
                referrer: test_2_address.clone(),
            },
        )?;

        let test_2_referral_code = test_2_referral_code.code;

        println!("Referral Code Registered: {test_2_address} => {test_2_referral_code}");

        let hub: DappResponse = query_contract(
            sh,
            &hub_addr,
            QueryMsg::Dapp {
                dapp: hub_addr.clone(),
            },
        )?;

        println!("Hub Status:");
        println!("\tFee: {}", hub.fee.unwrap());
        println!("\tFee Split Percent: {}%", hub.percent);
        println!("\tTotal Invocations: {}", hub.total_invocations);
        println!("\tDiscrete Referrers: {}", hub.discrete_referrers);
        println!(
            "\tTotal Contributions (to Referrers): {}",
            hub.total_contributions
        );
        println!("\tTotal Rewards: {}", hub.total_rewards);

        let test_1_balance = account_balance(sh, &test_1_address)?;

        println!(
            "{} (owner of referral code {}) balance: {}",
            test_1_address, test_1_referral_code, test_1_balance
        );

        println!("Collecting earnings for code: {test_1_referral_code} (costs referrer 1000 in contract premium)...");

        exec_contract(
            sh,
            "test_1",
            &hub_addr,
            ExecuteMsg::CollectReferrer {
                code: test_1_referral_code,
                dapp: hub_addr.clone(),
            },
            Some(500_000),
            1000,
        )?;

        let test_1_balance = account_balance(sh, &test_1_address)?;

        println!(
            "{} (owner of referral code {}) balance: {}",
            test_1_address, test_1_referral_code, test_1_balance
        );

        let test_0_balance = account_balance(sh, &test_0_address)?;

        println!(
            "{} (owner/collector of Referrals Hub) balance: {}",
            test_0_address, test_0_balance
        );

        let hub: DappResponse = query_contract(
            sh,
            &hub_addr,
            QueryMsg::Dapp {
                dapp: hub_addr.clone(),
            },
        )?;

        println!("Hub Status:");
        println!("\tFee: {}", hub.fee.unwrap());
        println!("\tFee Split Percent: {}%", hub.percent);
        println!("\tTotal Invocations: {}", hub.total_invocations);
        println!("\tDiscrete Referrers: {}", hub.discrete_referrers);
        println!(
            "\tTotal Contributions (to Referrers): {}",
            hub.total_contributions
        );
        println!("\tTotal Rewards: {}", hub.total_rewards);

        println!("Collecting earnings for Hub owner: {test_0_address} (costs dApp collector 1000 in contract premium)...");

        exec_contract(
            sh,
            "test_0",
            &hub_addr,
            ExecuteMsg::CollectDapp {
                dapp: hub_addr.clone(),
            },
            Some(500_000),
            1000,
        )?;

        let test_0_balance = account_balance(sh, &test_0_address)?;

        println!(
            "{} (owner/collector of Referrals Hub) balance: {}",
            test_0_address, test_0_balance
        );

        Ok(())
    }

    pub fn clean(sh: &Shell) -> Result<()> {
        let dir = archwayd_repo_dir();
        sh.remove_path(dir)?;
        Ok(())
    }
}
