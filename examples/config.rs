use xdiff::config::DiffConfig;
use xdiff::LoadConfig;
use anyhow::Result;

fn main() -> Result<()> {
    let data = include_str!("../fixtures/config.yml");
    let config =  DiffConfig::from_yaml(data)?;
    print!("{:#?}", config);
    Ok(())
}