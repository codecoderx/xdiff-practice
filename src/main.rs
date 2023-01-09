use xdiff::cli::{Action, DiffArgs, RunArgs};
use clap::Parser;
use anyhow::Result;
use xdiff::config::DiffConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let args: DiffArgs =  DiffArgs::parse();

    match args.action {
        Action::Run(run_args) => run(run_args).await?,
        _ => panic!("Not Implemented") 
    }

    Ok(())
}

async fn run(args: RunArgs) -> Result<()> {
    let config = args.config.unwrap_or_else(|| "fixtures/config.yml".to_string());
    let diff_config = DiffConfig::load_yaml(&config).await?;

    let profile = diff_config.get_profile(&args.profile).ok_or_else(|| anyhow::anyhow!("Profile {} not found in config file {}", args.profile, config))?;
    let extra_params =  args.extra_params.into();
    
    profile.diff(extra_params).await?;
    Ok(())
}