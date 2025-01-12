use clap::{Parser, Subcommand};
use color_eyre::eyre::ErrReport;
use interim::DateError;
use log::info;
use sea_orm::*;
use std::io::{self};

mod color;
mod data;
mod db;
mod do_add;
mod do_init;
mod do_tui;
mod entities;
mod migrator;

use crate::do_add::{do_add, AddArgs};
use crate::entities::ppl::Column::Me;
use entities::prelude::*;

#[derive(Debug)]
enum PplError {
    DateError(DateError),
    DbError(DbErr),
    EyreError(ErrReport),
    Std(io::Error),
    XDG(xdg::BaseDirectoriesError),
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

impl From<DateError> for PplError {
    fn from(value: DateError) -> Self {
        PplError::DateError(value)
    }
}

impl From<xdg::BaseDirectoriesError> for PplError {
    fn from(value: xdg::BaseDirectoriesError) -> Self {
        PplError::XDG(value)
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
    /// Calendar of upcoming events
    Calendar,
    /// Show MOTD Version
    MOTD,
    /// Add ppl
    Add(AddArgs),
    /// Aug ppl
    Aug,
    /// Edit ppl
    Edit { name: Option<String> },
    /// Show ppl
    Show,
    /// Tiers
    Tiers,
    ///
    Traits,
    /// Stats
    Stats,
}

#[tokio::main]
async fn main() -> Result<(), PplError> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("pplapp")?;
    xdg_dirs.create_data_directory("")?;
    let data_home_path = xdg_dirs.get_data_home();
    let database_url: String = format!(
        "sqlite://{}/db.db?mode=rwc",
        data_home_path.to_str().unwrap()
    );

    env_logger::init();
    color_eyre::install()?;
    let db = Database::connect(database_url).await?;
    db::check_migrations(&db).await?;

    let cli = Cli::parse();

    let is_init = Ppl::find().filter(Me.eq(true)).one(&db).await?.is_some();

    match &cli.command {
        Some(Commands::Edit { name }) => {
            println!("'edit: {:?}", name)
        }
        None => {
            println!("ppl");
            // show().expect("failed2draw")
        }
        Some(Commands::MOTD { .. }) => {}
        Some(Commands::Init { .. }) => match is_init {
            false => {
                info!("Uninitialized");

                let terminal = ratatui::init();
                let result = do_init::run_init(terminal, db).await;
                ratatui::restore();
                drop(result);
                info!("Init complete");
            }
            true => {
                println!("ppl has been initialized already");
            }
        },
        Some(Commands::Add(args)) => {
            let _ = do_add(args, db).await;
        }
        Some(Commands::Aug) => {}
        Some(Commands::Tui) => {
            let terminal = ratatui::init();
            let result = do_tui::run_tui(terminal, db).await;
            ratatui::restore();
            drop(result);
        }
        Some(Commands::Calendar) => {}
        Some(Commands::Show {}) => match is_init {
            true => {
                let ppl = Ppl::find().all(&db).await?;
                for p in ppl {
                    println!("{:?}", p)
                }
            }
            false => {
                info!("pls run init");
            }
        },
        Some(Commands::Tiers) => match is_init {
            true => {
                let tiers = TierDefaults::find().all(&db).await?;
                for t in tiers {
                    println!("{:?}", t)
                }
            }
            false => {
                info!("pls run init");
            }
        },
        Some(Commands::Traits) => match is_init {
            true => {
                let traits = TraitDefaults::find().all(&db).await?;
                for t in traits {
                    println!("{:?}", t)
                }
            }
            false => {
                info!("pls run init");
            }
        },
        Some(Commands::Stats) => match is_init {
            true => {
                let cnt = Ppl::find().all(&db).await?.len();
                println!("{:?} ppl", cnt)
            }
            false => {
                info!("pls run init");
            }
        },
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
