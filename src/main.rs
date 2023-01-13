use xdiff::ExtraArgs;
use xdiff::cli::{Action, DiffArgs, RunArgs};
use clap::Parser;
use anyhow::Result;
use xdiff::config::{DiffConfig, DiffProfile, ResponseProfile};
use xdiff::req::RequestProfile;
use std::io::Write;
use dialoguer::{Input, MultiSelect};
use dialoguer::theme::ColorfulTheme;
use xdiff::utils::highlight_text;
use xdiff::{LoadConfig, Validate};

#[tokio::main]
async fn main() -> Result<()> {
    let args: DiffArgs =  DiffArgs::parse();

    match args.action {
        Action::Run(run_args) => run(run_args).await?,
        Action::Parse => parse().await?,
        _ => panic!("Not Implemented") 
    }

    Ok(())
}

async fn run(args: RunArgs) -> Result<()> {
    let config = args.config.unwrap_or_else(|| "fixtures/config.yml".to_string());
    let diff_config = DiffConfig::load_yaml(&config).await?;

    let profile = diff_config.get_profile(&args.profile).ok_or_else(|| anyhow::anyhow!("Profile {} not found in config file {}", args.profile, config))?;
    let extra_params =  args.extra_params.into();

    let output = profile.diff(extra_params).await?;
    
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    write!(stdout, "{}", output)?;

    Ok(())
}

async fn parse()-> Result<()> {
    let theme = ColorfulTheme::default();
    let url1 : String = Input::with_theme(&theme)
        .with_prompt("Url1")
        .interact_text()?;
    let req1: RequestProfile = url1.parse()?;
    req1.validate()?;

    let url2 : String = Input::with_theme(&theme)
        .with_prompt("Url2")
        .interact_text()?;
    let req2: RequestProfile = url2.parse()?;
    req1.validate()?;

    let res = req1.send(&ExtraArgs::default()).await?;
    
    let headers = res.get_header_keys()?;
    let chosen : Vec<usize> = MultiSelect::new()
        .with_prompt("Select skip headers")
        .items(&headers)
        .interact()?;

    let skip_headers: Vec<String> = chosen.iter().map(|i| headers[*i].to_string()).collect();
    let response_profile = ResponseProfile::new(skip_headers, Vec::new());
    let diff_profile = DiffProfile::new(req1, req2, response_profile);

    let profile : String = Input::with_theme(&theme)
        .with_prompt("Profile")
        .interact_text()?;

    let config = DiffConfig::new([(profile, diff_profile)].into());
    let text = &serde_yaml::to_string(&config)?;

    let stdout  = std::io::stdout();
    let mut output = stdout.lock();

    // 兼容管道或重定向
    if atty::is(atty::Stream::Stdout) { 
        write!(output, "{}", highlight_text(&text, "yaml")?)?;
    } else {
        write!(output, "{}", text)?;
    }

    Ok(())
}