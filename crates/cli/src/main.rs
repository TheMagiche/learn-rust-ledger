//! `spend` — the command line and the `serve` command that hosts the API.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;

use chrono::Local;
use clap::{Parser, Subcommand};
use spend_api::ServeError;
use spend_core::report::format_report;
use spend_core::{
    month_key, parse_user_date, parse_user_month, Category, Expense, Filter, Ledger, Money,
    NewExpense, SpendError,
};
use spend_store::{JsonFileStore, Store, StoreError};
use thiserror::Error;

#[derive(Debug, Error)]
enum CliError {
    #[error(transparent)]
    Spend(#[from] SpendError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Serve(#[from] ServeError),
    #[error("{0}")]
    Message(String),
}

#[derive(Parser)]
#[command(
    name = "spend",
    version,
    about = "Track expenses from the terminal, or serve the API and teaching UI."
)]
struct Cli {
    /// Ledger file. Defaults to ~/.spend.json.
    #[arg(long, global = true)]
    file: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Record an expense. `450` means 450.00 and `12.50` means 12.50.
    Add {
        amount: String,
        category: String,
        #[arg(long)]
        note: Option<String>,
        #[arg(long)]
        date: Option<String>,
    },
    /// List expenses. Filters are optional and combine.
    List {
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        month: Option<String>,
        #[arg(long)]
        search: Option<String>,
    },
    /// Print the total for a month, or for every expense when `--month` is omitted.
    Total {
        #[arg(long)]
        month: Option<String>,
    },
    /// Per-category totals and budget status. Defaults to the current month.
    Report {
        #[arg(long)]
        month: Option<String>,
    },
    /// Delete one expense by id.
    Remove { id: u32 },
    /// Monthly category caps.
    Budget {
        #[command(subcommand)]
        command: BudgetCommand,
    },
    /// Serve the JSON API. Without `--dev`, also serve the embedded web UI.
    Serve {
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: String,
        /// Allow the Vite dev server (http://localhost:5173) to call the API.
        #[arg(long)]
        dev: bool,
    },
}

#[derive(Subcommand)]
enum BudgetCommand {
    /// `spend budget set food 150.00`
    Set { category: String, amount: String },
}

fn main() -> ExitCode {
    // LEARN(cargo): Cargo compiles this file into the `spend` binary.
    // `cargo run -p spend -- serve --dev` builds the workspace and starts the server.
    // LEARN(result): user mistakes return `Err`. We print `error: ...` and exit 1.
    // Panics are reserved for bugs, not for bad input, so library code does not call `unwrap`.
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    let path = resolve_path(cli.file)?;
    match cli.command {
        Command::Add {
            amount,
            category,
            note,
            date,
        } => cmd_add(&path, amount, category, note, date),
        Command::List {
            category,
            month,
            search,
        } => cmd_list(&path, category, month, search),
        Command::Total { month } => cmd_total(&path, month),
        Command::Report { month } => cmd_report(&path, month),
        Command::Remove { id } => cmd_remove(&path, id),
        Command::Budget { command } => match command {
            BudgetCommand::Set { category, amount } => cmd_budget_set(&path, category, amount),
        },
        Command::Serve { addr, dev } => cmd_serve(&path, &addr, dev),
    }
}

fn resolve_path(flag: Option<PathBuf>) -> Result<PathBuf, CliError> {
    match flag {
        Some(path) => Ok(expand_tilde(path)),
        None => {
            let home = std::env::var_os("HOME")
                .ok_or_else(|| CliError::Message("HOME is not set; pass --file".to_string()))?;
            Ok(PathBuf::from(home).join(".spend.json"))
        }
    }
}

fn expand_tilde(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    path
}

fn load(path: &Path) -> Result<Ledger, CliError> {
    JsonFileStore::new(path).load().map_err(Into::into)
}

fn save(path: &Path, ledger: &Ledger) -> Result<(), CliError> {
    JsonFileStore::new(path).save(ledger).map_err(Into::into)
}

fn cmd_add(
    path: &Path,
    amount: String,
    category: String,
    note: Option<String>,
    date: Option<String>,
) -> Result<(), CliError> {
    // LEARN(shadowing): `amount` starts as the text the user typed. This new binding
    // reuses the name for the parsed `Money`. The right-hand side still sees the text.
    let amount = Money::parse(&amount)?;
    let category = Category::from_str(category.trim())?;
    let date = match date {
        Some(value) => parse_user_date(&value)?,
        None => Local::now().date_naive(),
    };
    let mut ledger = load(path)?;
    let created = {
        let expense = ledger.add(NewExpense {
            amount,
            category,
            note,
            date,
        })?;
        expense.clone()
    };
    save(path, &ledger)?;
    println!("added {}", format_expense(&created));
    Ok(())
}

fn cmd_list(
    path: &Path,
    category: Option<String>,
    month: Option<String>,
    search: Option<String>,
) -> Result<(), CliError> {
    let ledger = load(path)?;
    let filter = make_filter(category, month, search)?;
    let rows = ledger.list(&filter);
    if rows.is_empty() {
        println!("no expenses");
        return Ok(());
    }
    for expense in rows {
        println!("{}", format_expense(expense));
    }
    Ok(())
}

fn cmd_total(path: &Path, month: Option<String>) -> Result<(), CliError> {
    let ledger = load(path)?;
    let filter = make_filter(None, month, None)?;
    println!("{}", ledger.total(&filter));
    Ok(())
}

fn cmd_report(path: &Path, month: Option<String>) -> Result<(), CliError> {
    let ledger = load(path)?;
    let month = match month {
        Some(value) => parse_user_month(&value)?,
        None => month_key(Local::now().date_naive()),
    };
    let filter = Filter {
        month: Some(month.clone()),
        ..Filter::default()
    };
    let text = format_report(
        &month,
        &ledger.by_category(Some(&month)),
        &ledger.budget_status(&month),
        ledger.total(&filter),
    );
    println!("{text}");
    Ok(())
}

fn cmd_remove(path: &Path, id: u32) -> Result<(), CliError> {
    let mut ledger = load(path)?;
    let removed = ledger.remove(id)?;
    save(path, &ledger)?;
    println!("removed {}", format_expense(&removed));
    Ok(())
}

fn cmd_budget_set(path: &Path, category: String, amount: String) -> Result<(), CliError> {
    let category = Category::from_str(category.trim())?;
    let amount = Money::parse(&amount)?;
    let mut ledger = load(path)?;
    ledger.set_budget(category, amount);
    save(path, &ledger)?;
    let month = month_key(Local::now().date_naive());
    let status = ledger
        .budget_status(&month)
        .into_iter()
        .find(|row| row.category == category);
    match status {
        Some(status) => {
            let flag = if status.over { "over" } else { "ok" };
            println!(
                "{category} budget {} spent {} {flag}",
                status.budget, status.spent
            );
        }
        None => println!("{category} budget {amount}"),
    }
    Ok(())
}

fn cmd_serve(path: &Path, addr: &str, dev: bool) -> Result<(), CliError> {
    let store = JsonFileStore::new(path);
    store.load()?;
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
    if dev {
        println!("spend API on http://{addr}");
        println!("dev mode: start the UI with `npm --prefix web run dev` (it proxies /api here)");
    } else {
        println!("spend UI and API on http://{addr}");
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| CliError::Message(err.to_string()))?;
    runtime.block_on(spend_api::serve(addr, Box::new(store), dev))?;
    Ok(())
}

fn make_filter(
    category: Option<String>,
    month: Option<String>,
    search: Option<String>,
) -> Result<Filter, CliError> {
    let category = match category {
        Some(value) => Some(Category::from_str(value.trim())?),
        None => None,
    };
    let month = match month {
        Some(value) => Some(parse_user_month(&value)?),
        None => None,
    };
    Ok(Filter {
        category,
        month,
        search,
    })
}

fn format_expense(expense: &Expense) -> String {
    match &expense.note {
        Some(note) => format!(
            "{:<4} {}  {:<10} {:>10}  {note}",
            expense.id, expense.date, expense.category, expense.amount
        ),
        None => format!(
            "{:<4} {}  {:<10} {:>10}",
            expense.id, expense.date, expense.category, expense.amount
        ),
    }
}
