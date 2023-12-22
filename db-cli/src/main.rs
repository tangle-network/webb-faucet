use std::collections::HashSet;

use argh::FromArgs;
use color_eyre::eyre::Result;
use webb_auth_sled::{ClaimsData, SledAuthDb};
use webb_proposals::TypedChainId;

/// Webb Faucet Database CLI
#[derive(Debug, Clone, FromArgs)]
struct Args {
    /// sled database path
    #[argh(option, short = 'd')]
    db: std::path::PathBuf,

    /// control verbosity level
    #[argh(option, short = 'v', default = "0")]
    verbosity: u8,
    /// output file for evm addresses
    #[argh(option, short = 'e')]
    evm_output: Option<std::path::PathBuf>,
    /// output file for substrate addresses
    #[argh(option, short = 's')]
    substrate_output: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args: Args = argh::from_env();
    setup_logger(args.verbosity, "db_cli")?;
    tracing::info!("opening db at {:?}", args.db);
    let db = SledAuthDb::open(args.db)?;
    let chains = [
        // Tangle
        TypedChainId::Substrate(1081),
        // Tangle Local
        TypedChainId::Substrate(1082),
        // Athena
        TypedChainId::Evm(3884533461),
        // Demeter
        TypedChainId::Evm(3884533463),
        // Hermes
        TypedChainId::Evm(3884533462),
        // Tangle EVM Testnet
        TypedChainId::Evm(4006),
    ];

    let mut accounts = HashSet::new();

    for chain in chains.iter() {
        tracing::debug!(
            chain = %chain.chain_id(),
            "processing chain claims",
        );

        let chain_accounts = db.claims_tree(*chain)?.iter().flat_map(|kv| {
            kv.ok()
                .and_then(|(_, v)| {
                    serde_json::from_slice::<ClaimsData>(&v).ok()
                })
                .map(|c| c.address)
        });
        accounts.extend(chain_accounts);
        tracing::debug!("Total accounts (so far): {}", accounts.len());
    }
    let evm_accounts = accounts
        .iter()
        .filter_map(|a| a.as_ethereum().map(|v| format!("{:?}", v)))
        .collect::<Vec<_>>();
    let substrate_accounts = accounts
        .iter()
        .filter_map(|a| a.as_substrate().map(|v| v.to_string()))
        .collect::<Vec<_>>();
    if let Some(output) = args.evm_output {
        std::fs::write(output, serde_json::to_string_pretty(&evm_accounts)?)?;
    } else {
        eprintln!("{}", serde_json::to_string_pretty(&evm_accounts)?);
    }
    if let Some(output) = args.substrate_output {
        std::fs::write(
            output,
            serde_json::to_string_pretty(&substrate_accounts)?,
        )?;
    } else {
        eprintln!("{}", serde_json::to_string_pretty(&substrate_accounts)?);
    }
    Ok(())
}

pub fn setup_logger(verbosity: u8, filter: &str) -> Result<()> {
    use tracing::Level;
    let log_level = match verbosity {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let directive_1 = format!("{filter}={log_level}")
        .parse()
        .expect("valid log level");
    let directive_2 = format!("webb_={log_level}")
        .parse()
        .expect("valid log level");
    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(directive_1)
        .add_directive(directive_2);
    let logger = tracing_subscriber::fmt()
        .with_target(true)
        .with_max_level(log_level)
        .with_env_filter(env_filter);
    let logger = logger.pretty();
    logger.init();
    Ok(())
}
