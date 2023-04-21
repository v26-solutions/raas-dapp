use anyhow::Result;
use clap::{Parser, Subcommand};
use xshell::Shell;

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
    #[command(about = "watch source files and run tests on changes")]
    Dev {
        #[arg(short, long, help = "update expect test results")]
        update: bool,
    },
    #[command(about = "install used cargo plugins (if not using Nix)")]
    Install,
    #[command(subcommand, about = "web client tasks")]
    WebClient(WebClient),
    #[command(subcommand, about = "archway deployment tasks")]
    Archway(Archway),
}

#[derive(Subcommand)]
enum WebClient {
    #[command(about = "start the development server")]
    Dev {
        #[arg(long, help = "expose server on 0.0.0.0")]
        host: bool,
    },
}

#[derive(Subcommand)]
enum Archway {
    #[command(about = "initialize a local node")]
    InitLocal,
    #[command(about = "start a local node")]
    StartLocal,
    #[command(about = "deploy contracts to a local node")]
    DeployLocal,
    #[command(about = "remove local node directory")]
    Clean,
    #[command(about = "print mnemonics of all test accounts")]
    PrintMnemonics,
}

pub fn main() -> Result<()> {
    let cli = Cli::parse();

    dotenv::dotenv()?;

    let sh = Shell::new()?;

    match cli.command {
        Command::Coverage => xtask::coverage(&sh),
        Command::Test { update, backtrace } => xtask::test(&sh, update, backtrace),
        Command::Dist => xtask::dist(&sh),
        Command::Dev { update } => xtask::dev(&sh, update),
        Command::Install => xtask::install(&sh),
        Command::WebClient(cmd) => {
            use xtask::web_client;

            match cmd {
                WebClient::Dev { host } => web_client::dev(&sh, host),
            }
        }
        Command::Archway(cmd) => {
            use xtask::archway;

            match cmd {
                Archway::InitLocal => archway::init_local(&sh),
                Archway::StartLocal => archway::start_local(&sh),
                Archway::DeployLocal => archway::deploy_local(&sh),
                Archway::Clean => archway::clean(&sh),
                Archway::PrintMnemonics => archway::print_mnemonics(),
            }
        }
    }
}
