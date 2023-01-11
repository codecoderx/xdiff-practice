use clap::{Parser, Args, Subcommand};
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct DiffArgs {
    #[command(subcommand)]
    pub action: Action
}

#[derive(Subcommand, Debug)]
#[non_exhaustive]
pub enum Action {
    Run(RunArgs),
    Parse
}

#[derive(Args, Debug)]
pub struct RunArgs {
    #[arg(short, long)]
    pub profile: String,

    #[arg(short, long)]
    pub config: Option<String>,

    /// -e @a=b -e %c=d -e d=e
    #[arg(short, long, value_parser = key_val_parse)]
    pub extra_params: Vec<KeyVal>
}

#[derive(Clone, Debug)]
pub enum KeyValType {
    Header,
    Body,
    Query
}

#[derive(Clone, Debug)]
pub struct KeyVal {
    pub key_type: KeyValType,
    pub key: String,
    pub val: String
}

fn key_val_parse(data: &str) -> Result<KeyVal> {
    let mut args = data.splitn(2, "=");
    let key = args.next().unwrap();
    let val = args.next().unwrap();

    let (key_type, key) = match key.chars().next() {
        Some('%') => (KeyValType::Header, key[1..].to_string()),
        Some('@') => (KeyValType::Body, key[1..].to_string()),
        Some(v) if v.is_ascii_alphabetic() => (KeyValType::Query, key.to_string()),
        _ => return Err(anyhow::anyhow!("Invalid key value pair"))
    };
    
    Ok(KeyVal{
        key_type,
        key,
        val: val.to_string()
    })
}
