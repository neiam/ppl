use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use futures::executor::block_on;
use log::info;
use ratatui::{prelude::*, widgets::*, DefaultTerminal};
use sea_orm::*;
use sea_orm_migration::SchemaManager;
use std::io::{self, stdout};
use color_eyre::eyre::ErrReport;
use Event::Key;

mod entities;
mod migrator;
mod db;
mod do_init;

use crate::entities::ppl::Model;
use crate::migrator::Migrator;
use entities::{prelude::*, *};

#[derive(Debug)]
enum PplError {
    DbError(DbErr),
    EyreError(ErrReport),
    Std(io::Error),

}

impl From<io::Error> for PplError {
    fn from(value: io::Error) -> Self {
        PplError::Std(value)
    }
}
impl From<DbErr> for PplError {
    fn from(value: DbErr) -> Self {
        PplError::DbError(value)
    }
}

impl From<ErrReport> for PplError {
    fn from(value: ErrReport) -> Self {
        PplError::EyreError(value)
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize ppl
    Init,
    /// TUI
    Tui,
    /// Show MOTD Version
    MOTD,
    /// Add ppl
    Add,
    /// Edit ppl
    Edit { name: Option<String> },
    /// Show ppl
    Show,
    /// Tiers
    Tiers,
    /// Stats
    Stats,
}


const DATABASE_URL: &str = "sqlite://database.sqlite?mode=rwc";
#[tokio::main]
async fn main() -> Result<(), PplError> {
    env_logger::init();
    let db = Database::connect(DATABASE_URL).await?;
    db::check_migrations(&db).await?;



    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Edit { name }) => {
            println!("'edit: {:?}", name)
        }
        Some(Commands::Show {}) => {
            println!("showing but selected");
            // show().expect("failed2draw")
        }
        None => {
            println!("Showing PPL");
            // show().expect("failed2draw")
        }
        Some(Commands::MOTD { .. }) => {}
        Some(Commands::Init { .. }) => {
            match Ppl::find()
                .filter(ppl::Column::Me.eq(true))
                .one(&db)
                .await?
            {
                None => {
                    info!("Uninitialized");
                    color_eyre::install()?;
                    let terminal = ratatui::init();
                    let result = do_init::run_init(terminal);
                    ratatui::restore();
                    result;
                    info!("Init complete");
                }
                Some(_) => {
                    info!("ppl has been initialized already");
                }
            }
        }
        Some(Commands::Add { .. }) => {}
        Some(Commands::Tui) => {}
        Some(Commands::Tiers) => {}
        Some(Commands::Stats) => {}
    }
    Ok(())
}

//
// fn show() -> io::Result<()> {
//     enable_raw_mode()?;
//     stdout().execute(EnterAlternateScreen)?;
//     let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
//
//     let mut should_quit = false;
//     while !should_quit {
//         terminal.draw(ui)?;
//         should_quit = handle_events()?;
//     }
//
//     disable_raw_mode()?;
//     // stdout().execute(LeaveAlternateScreen)?;
//     Ok(())
// }
//
// fn handle_events() -> io::Result<bool> {
//     if event::poll(std::time::Duration::from_millis(50))? {
//         if let Event::Key(key) = event::read()? {
//             if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
//                 return Ok(true);
//             }
//         }
//     }
//     Ok(false)
// }
//
// fn ui(frame: &mut Frame) {
//     frame.render_widget(
//         Paragraph::new("Hello World!")
//             .block(Block::default().title("Greeting").borders(Borders::ALL)),
//         frame.size(),
//     );
// }
//
//
// fn main() {
//     let cli = Cli::parse();
//
//     // You can check for the existence of subcommands, and if found use their
//     // matches just as you would the top level cmd
//     match &cli.command {
//         Some(Commands::Edit { name }) => {
//             println!("'myapp add' was used, name is: {:?}", name)
//         }
//         Some(Commands::Show {}) => {
//             println!("showing but selected");
//             show().expect("failed2draw")
//         }
//         None => {
//             println!("Showing PPL");
//             show().expect("failed2draw")
//         }
//         Some(Commands::MOTD { .. }) => {}
//     }
// }
