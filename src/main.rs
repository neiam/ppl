use clap::{Parser, Subcommand};
use color_eyre::eyre::ErrReport;
use interim::DateError;
use log::{debug, info};
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
    /// Traits
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
        Some(Commands::MOTD) => {
            motd(&db).await?;
        }
        Some(Commands::Init) => match is_init {
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
        Some(Commands::Show) => match is_init {
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

enum MotdRemindEnum {
    OnlyOnce,     // Default, Only the first time the motd is run that day
    OnceHourly,   // Only once per hour, resetting at the top of the hour
    Randomly(u8), // Will show this motd person randomly u8% every run
    Always,       // Will always show this person's upcoming
    Never,        // Never show this person's upcoming
}

async fn motd(db: &DatabaseConnection) -> Result<(), PplError> {
    use chrono::{Datelike, Local, NaiveDate};

    let today = Local::now().date_naive();

    debug!("DEBUG: Today is {}", today);

    // Get all people
    let people = Ppl::find().all(db).await?;
    debug!("DEBUG: Found {} people", people.len());

    for person in people {
        // Get all sigdates for this person
        let sigdates = SigDate::find()
            .filter(entities::sig_date::Column::PplId.eq(person.id))
            .all(db)
            .await?;

        debug!(
            "DEBUG: Person '{}' has {} sigdates",
            person.name,
            sigdates.len()
        );

        // Get all tiers for this person
        let person_tiers = Tier::find()
            .filter(entities::tier::Column::PplId.eq(person.id))
            .all(db)
            .await?;

        debug!(
            "DEBUG: Person '{}' has {} tiers",
            person.name,
            person_tiers.len()
        );

        // Determine the remind enum for this person (default to OnlyOnce if null)
        // Also get the tier color and symbol for display
        let mut remind_enum_value: Option<String> = None;
        let mut tier_color: Option<String> = None;
        let mut tier_symbol: Option<String> = None;

        for tier in &person_tiers {
            if tier.sig_remind_enum.is_some() {
                remind_enum_value = tier.sig_remind_enum.clone();
            }
            if tier_color.is_none() {
                tier_color = tier.color.clone();
            }
            if tier_symbol.is_none() {
                tier_symbol = tier.symbol.clone();
            }
            // Break once we have all we need
            if remind_enum_value.is_some() && tier_color.is_some() && tier_symbol.is_some() {
                break;
            }
        }

        // Check if we should show this person based on remind_enum (default OnlyOnce)
        // If remind_enum is null or "OnlyOnce", check if we've shown them today
        let should_show =
            if remind_enum_value.is_none() || remind_enum_value.as_deref() == Some("OnlyOnce") {
                // Check meta field for last_reminded date
                if let Some(meta_json) = &person.meta {
                    if let Some(last_reminded) = meta_json.get("last_reminded") {
                        if let Some(last_reminded_str) = last_reminded.as_str() {
                            if let Ok(last_reminded_date) =
                                NaiveDate::parse_from_str(last_reminded_str, "%Y-%m-%d")
                            {
                                debug!(
                                    "DEBUG: Person '{}' was last reminded on {}",
                                    person.name, last_reminded_date
                                );
                                // Only show if last reminded was before today
                                last_reminded_date < today
                            } else {
                                true // Invalid date format, show anyway
                            }
                        } else {
                            true // Not a string, show anyway
                        }
                    } else {
                        true // No last_reminded field, show
                    }
                } else {
                    true // No meta, show
                }
            } else {
                true // Other remind modes not yet implemented, show for now
            };

        if !should_show {
            debug!(
                "DEBUG: Skipping person '{}' - already shown today",
                person.name
            );
            continue;
        }

        // Track if we showed any sigdates for this person
        let mut showed_sigdate = false;

        // Get all tier defaults
        let tier_defaults = TierDefaults::find().all(db).await?;
        debug!("DEBUG: Found {} tier defaults", tier_defaults.len());

        // Get all trait defaults for date matching
        let trait_defaults = TraitDefaults::find()
            .filter(entities::trait_defaults::Column::IsDate.eq(true))
            .all(db)
            .await?;
        debug!("DEBUG: Found {} date trait defaults", trait_defaults.len());

        for sigdate in sigdates {
            debug!(
                "DEBUG: Processing sigdate: {} on {}",
                sigdate.event, sigdate.date
            );
            let mut tier_delta: Option<u32> = None;

            // First check person's assigned tiers
            for tier in &person_tiers {
                debug!(
                    "DEBUG: Checking tier '{}' with delta {:?}",
                    tier.name, tier.sig_date_delta
                );
                if let Some(delta) = tier.sig_date_delta {
                    tier_delta = Some(delta);
                    break;
                }
            }

            // If no tier delta found, check tier_defaults
            if tier_delta.is_none() {
                for tier_default in &tier_defaults {
                    debug!(
                        "DEBUG: Checking tier_default '{}' with delta {:?}",
                        tier_default.key, tier_default.sig_date_delta
                    );
                    if let Some(delta) = tier_default.sig_date_delta {
                        tier_delta = Some(delta);
                        break;
                    }
                }
            }

            // If we have a tier delta, check if we're within the reminder window for this year's anniversary
            if let Some(delta) = tier_delta {
                // Calculate this year's anniversary
                let this_year_anniversary =
                    NaiveDate::from_ymd_opt(today.year(), sigdate.date.month(), sigdate.date.day());

                if let Some(anniversary) = this_year_anniversary {
                    let days_until = (anniversary - today).num_days();
                    debug!(
                        "DEBUG: This year's anniversary is {}, days until: {}",
                        anniversary, days_until
                    );

                    // Show if the anniversary is upcoming and within the delta window
                    if days_until >= 0 && days_until <= delta as i64 {
                        let years_since = today.year() - sigdate.date.year();
                        let display_name = person.nick.as_ref().unwrap_or(&person.name);

                        // Format the person's name with tier symbol and color if available
                        let formatted_name = if let Some(symbol) = &tier_symbol {
                            if let Some(color_str) = &tier_color {
                                use colored::Colorize;
                                let colored_name = match color_str.to_uppercase().as_str() {
                                    "PINK" => display_name.magenta(),
                                    "GREEN" => display_name.green(),
                                    "CYAN" => display_name.cyan(),
                                    "RED" => display_name.red(),
                                    "BLUE" => display_name.blue(),
                                    "YELLOW" => display_name.yellow(),
                                    "PURPLE" => display_name.purple(),
                                    "WHITE" => display_name.white(),
                                    "BLACK" => display_name.black(),
                                    "BRIGHT_RED" => display_name.bright_red(),
                                    "BRIGHT_GREEN" => display_name.bright_green(),
                                    "BRIGHT_YELLOW" => display_name.bright_yellow(),
                                    "BRIGHT_BLUE" => display_name.bright_blue(),
                                    "BRIGHT_MAGENTA" => display_name.bright_magenta(),
                                    "BRIGHT_CYAN" => display_name.bright_cyan(),
                                    "BRIGHT_WHITE" => display_name.bright_white(),
                                    _ => display_name.normal(), // default to no color
                                };
                                format!("{} {}", symbol, colored_name)
                            } else {
                                format!("{} {}", symbol, display_name)
                            }
                        } else {
                            display_name.to_string()
                        };

                        // Find matching trait_default for this sigdate event
                        let trait_info = trait_defaults.iter().find(|t| t.key == sigdate.event);

                        if let Some(trait_def) = trait_info {
                            println!(
                                "{} {} days until {} for {} ({}{})",
                                trait_def.symbol,
                                days_until,
                                sigdate.event,
                                formatted_name,
                                years_since,
                                if years_since == 1 { " year" } else { " years" }
                            );
                        } else {
                            println!(
                                "{} days until {} for {} ({}{})",
                                days_until,
                                sigdate.event,
                                formatted_name,
                                years_since,
                                if years_since == 1 { " year" } else { " years" }
                            );
                        }
                        showed_sigdate = true;
                    } else if days_until < 0 {
                        // Check next year's anniversary
                        let next_year_anniversary = NaiveDate::from_ymd_opt(
                            today.year() + 1,
                            sigdate.date.month(),
                            sigdate.date.day(),
                        );
                        if let Some(next_anniversary) = next_year_anniversary {
                            let days_until_next = (next_anniversary - today).num_days();
                            debug!(
                                "DEBUG: Next year's anniversary is {}, days until: {}",
                                next_anniversary, days_until_next
                            );

                            if days_until_next >= 0 && days_until_next <= delta as i64 {
                                let years_since = (today.year() + 1) - sigdate.date.year();
                                let display_name = person.nick.as_ref().unwrap_or(&person.name);

                                // Format the person's name with tier symbol and color if available
                                let formatted_name = if let Some(symbol) = &tier_symbol {
                                    if let Some(color_str) = &tier_color {
                                        use colored::Colorize;
                                        let colored_name = match color_str.to_uppercase().as_str() {
                                            "PINK" => display_name.magenta(),
                                            "GREEN" => display_name.green(),
                                            "CYAN" => display_name.cyan(),
                                            "RED" => display_name.red(),
                                            "BLUE" => display_name.blue(),
                                            "YELLOW" => display_name.yellow(),
                                            "PURPLE" => display_name.purple(),
                                            "WHITE" => display_name.white(),
                                            "BLACK" => display_name.black(),
                                            "BRIGHT_RED" => display_name.bright_red(),
                                            "BRIGHT_GREEN" => display_name.bright_green(),
                                            "BRIGHT_YELLOW" => display_name.bright_yellow(),
                                            "BRIGHT_BLUE" => display_name.bright_blue(),
                                            "BRIGHT_MAGENTA" => display_name.bright_magenta(),
                                            "BRIGHT_CYAN" => display_name.bright_cyan(),
                                            "BRIGHT_WHITE" => display_name.bright_white(),
                                            _ => display_name.normal(), // default to no color
                                        };
                                        format!("{} {}", symbol, colored_name)
                                    } else {
                                        format!("{} {}", symbol, display_name)
                                    }
                                } else {
                                    display_name.to_string()
                                };

                                // Find matching trait_default for this sigdate event
                                let trait_info =
                                    trait_defaults.iter().find(|t| t.key == sigdate.event);

                                if let Some(trait_def) = trait_info {
                                    println!(
                                        "{} {} days until {} for {} ({}{})",
                                        trait_def.symbol,
                                        days_until_next,
                                        sigdate.event,
                                        formatted_name,
                                        years_since,
                                        if years_since == 1 { " year" } else { " years" }
                                    );
                                } else {
                                    println!(
                                        "{} days until {} for {} ({}{})",
                                        days_until_next,
                                        sigdate.event,
                                        formatted_name,
                                        years_since,
                                        if years_since == 1 { " year" } else { " years" }
                                    );
                                }
                                showed_sigdate = true;
                            } else {
                                debug!("DEBUG: Next year's anniversary days until ({}) is outside delta window (0 to {})", days_until_next, delta);
                            }
                        }
                    } else {
                        debug!(
                            "DEBUG: Days until this year ({}) is outside delta window (0 to {})",
                            days_until, delta
                        );
                    }
                } else {
                    debug!(
                        "DEBUG: Could not calculate anniversary for {}",
                        sigdate.date
                    );
                }
            } else {
                debug!("DEBUG: No tier delta found for this sigdate");
            }
        }

        // Update person's meta field with last_reminded if we showed them and remind_enum is OnlyOnce
        if showed_sigdate
            && (remind_enum_value.is_none() || remind_enum_value.as_deref() == Some("OnlyOnce"))
        {
            use sea_orm::ActiveValue::Set;

            let today_str = today.format("%Y-%m-%d").to_string();
            let meta_value = serde_json::json!({
                "last_reminded": today_str
            });

            let mut person_active: entities::ppl::ActiveModel = person.into();
            person_active.meta = Set(Some(meta_value));

            if let Err(e) = person_active.update(db).await {
                debug!("DEBUG: Failed to update person meta: {}", e);
            } else {
                debug!("DEBUG: Updated last_reminded for person to {}", today);
            }
        }
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
