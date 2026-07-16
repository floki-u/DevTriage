use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use devtriage_core::{OutputBudget, Pipeline};
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BudgetArg {
    Compact,
    Standard,
    Detailed,
}

impl From<BudgetArg> for OutputBudget {
    fn from(value: BudgetArg) -> Self {
        match value {
            BudgetArg::Compact => Self::Compact,
            BudgetArg::Standard => Self::Standard,
            BudgetArg::Detailed => Self::Detailed,
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "devtriage",
    about = "Compile local debug evidence for humans and AI"
)]
struct Args {
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
    #[arg(long)]
    json: bool,
    #[arg(long, value_enum, default_value_t = BudgetArg::Standard)]
    budget: BudgetArg,
}

fn read_input(file: Option<PathBuf>) -> Result<String> {
    match file {
        Some(path) => std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display())),
        None => {
            let mut input = String::new();
            io::stdin()
                .read_to_string(&mut input)
                .context("failed to read stdin")?;
            Ok(input)
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = read_input(args.file)?;
    let context = Pipeline::default().analyze(&input, args.budget.into());
    if args.json {
        println!("{}", serde_json::to_string_pretty(&context)?);
    } else {
        println!("{}", context.output.text);
    }
    Ok(())
}
