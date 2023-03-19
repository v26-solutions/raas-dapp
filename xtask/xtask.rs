use anyhow::Result;
use clap::{Parser, Subcommand};
use xshell::{cmd, Shell};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "run test coverage")]
    Coverage,
    #[command(about = "run tests")]
    Test {
        #[arg(short, long, help = "update expected results")]
        update: bool,
        #[arg(short, long, help = "show backtrace")]
        backtrace: bool,
    },
    #[command(about = "compile contracts for distribution")]
    Dist,
    #[command(
        about = "watch source files and re-run 'xtask test --update --backtrace' when saving files"
    )]
    Dev,
    #[command(about = "install used cargo plugins (if not using Nix)")]
    Install,
}

fn coverage(sh: &Shell) -> Result<()> {
    cmd!(sh, "cargo llvm-cov --html").run()?;

    Ok(())
}

fn test(sh: &Shell, update: bool, backtrace: bool) -> Result<()> {
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

fn dist(sh: &Shell) -> Result<()> {
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

fn dev(sh: &Shell) -> Result<()> {
    cmd!(sh, "cargo watch -x 'xtask test --update --backtrace'").run()?;
    Ok(())
}

fn install(sh: &Shell) -> Result<()> {
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

pub fn main() -> Result<()> {
    let cli = Cli::parse();

    let sh = Shell::new()?;

    match cli.command {
        Command::Coverage => coverage(&sh),
        Command::Test { update, backtrace } => test(&sh, update, backtrace),
        Command::Dist => dist(&sh),
        Command::Dev => dev(&sh),
        Command::Install => install(&sh),
    }
}
