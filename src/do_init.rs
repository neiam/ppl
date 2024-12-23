use crate::Commands::Tiers;
use crate::PplError;
use chrono::NaiveDate;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use dateparser::{parse, DateTimeUtc};
use enum_iterator::{all, cardinality, first, last, next, previous, reverse_all, Sequence};
use interim::{parse_date_string, Dialect};
use ratatui::{
    prelude::*,
    style::{palette::tailwind::*, Color, Modifier, Style, Stylize},
    widgets::*,
    DefaultTerminal,
};
use sea_orm::sqlx::types::chrono;
use sea_orm::sqlx::types::chrono::Local;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::string::String;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Debug, PartialEq, Sequence, PartialOrd, Ord, Eq, Copy, Clone)]
enum Selection {
    Selected,
    NotSelected,
}

#[derive(Debug, Clone)]
struct TierSelect {
    selection: Selection,
    name: String,
}

#[derive(Debug, Clone)]
struct TraitSelect {
    selection: Selection,
    symbol: String,
    color: String,
    name: String,
    is_date: bool,
    is_contact: bool,
}
#[derive(Debug, Clone)]
struct Init {
    step: Steps,
    /// Input box value
    input: Input,
    /// Name
    name: String,
    /// Nicks
    nicks: Vec<String>,
    /// bday
    bday: NaiveDate,
    /// bday-parse-error
    bday_parse: String,
    /// bplace
    birthplace: String,
    /// of-parents
    of_ppl: Vec<String>,
    /// list state for the tier
    tier_state: ListState,
    /// tier storage
    tier_list: Vec<TierSelect>,
    /// editing/not
    tier_editing: Selection,
    /// list state for the traits
    trait_state: ListState,
    /// list-of-traits
    trait_list: Vec<TraitSelect>,
    /// editing/not
    trait_editing: Selection,
}

impl Default for Init {
    fn default() -> Init {
        Init {
            step: Steps::Welcome,
            input: Input::default(),
            nicks: Vec::<String>::default(),
            name: "".to_string(),
            bday: NaiveDate::default(),
            bday_parse: "".to_string(),
            birthplace: "".to_string(),
            of_ppl: Vec::<String>::default(),
            tier_state: ListState::default(),
            tier_list: all::<DefaultTiers>()
                .map(|e| TierSelect {
                    selection: Selection::Selected,
                    name: e.to_string(),
                })
                .collect::<Vec<TierSelect>>(),
            tier_editing: Selection::NotSelected,

            trait_state: ListState::default(),
            trait_editing: Selection::NotSelected,
            trait_list: vec![
                TraitSelect {
                    selection: Selection::Selected,
                    symbol: "🏷️".to_string(),
                    color: "VIOLET".to_string(),
                    name: "Nickname".to_string(),
                    is_date: false,
                    is_contact: false,
                },
                TraitSelect {
                    selection: Selection::Selected,
                    symbol: "🎂".to_string(),
                    color: "GOLD".to_string(),
                    name: "Birthday".to_string(),
                    is_date: true,
                    is_contact: false,
                },
                TraitSelect {
                    selection: Selection::Selected,
                    symbol: "💒".to_string(),
                    color: "WHITE".to_string(),
                    name: "Wedding".to_string(),
                    is_date: true,
                    is_contact: false,
                },
                TraitSelect {
                    selection: Selection::Selected,
                    symbol: "🤝".to_string(),
                    color: "PINK".to_string(),
                    name: "Met".to_string(),
                    is_date: true,
                    is_contact: false,
                },
                TraitSelect {
                    selection: Selection::Selected,
                    symbol: "📞".to_string(),
                    color: "TEAL".to_string(),
                    name: "Phone".to_string(),
                    is_date: false,
                    is_contact: true,
                },
                TraitSelect {
                    selection: Selection::Selected,
                    symbol: "📬".to_string(),
                    color: "RED".to_string(),
                    name: "Mailing".to_string(),
                    is_date: false,
                    is_contact: true,
                },
                TraitSelect {
                    selection: Selection::Selected,
                    symbol: "📧".to_string(),
                    color: "GREEN".to_string(),
                    name: "eMail".to_string(),
                    is_date: false,
                    is_contact: true,
                },
            ],
        }
    }
}

impl Init {
    fn toggle_tier(&mut self) {
        let selection = self.tier_state.selected();
        match selection {
            Some(s) => {
                self.tier_list[s].selection = match self.tier_list[s].selection {
                    Selection::Selected => Selection::NotSelected,
                    Selection::NotSelected => Selection::Selected,
                }
            }
            None => {}
        }
    }

    fn toggle_tier_editing(&mut self) {
        self.tier_editing = match self.tier_editing {
            Selection::Selected => Selection::NotSelected,
            Selection::NotSelected => Selection::Selected,
        }
    }

    fn tier_add(&mut self, tier: String) {
        let new_ts = TierSelect {
            selection: Selection::Selected,
            name: tier,
        };
        self.tier_list.push(new_ts);
    }

    fn tier_count_string(&self) -> String {
        let total = self.tier_list.len();
        let selected = self
            .tier_list
            .iter()
            .filter(|e| e.selection == Selection::Selected)
            .count();
        format!("{}/{} Selected", selected, total)
    }

    fn toggle_trait(&mut self) {
        let selection = self.trait_state.selected();
        match selection {
            Some(s) => {
                self.trait_list[s].selection = match self.trait_list[s].selection {
                    Selection::Selected => Selection::NotSelected,
                    Selection::NotSelected => Selection::Selected,
                }
            }
            None => {}
        }
    }

    fn trait_count_string(&self) -> String {
        let total = self.trait_list.len();
        let selected = self
            .trait_list
            .iter()
            .filter(|e| e.selection == Selection::Selected)
            .count();
        format!("{}/{} Selected", selected, total)
    }
}

#[derive(Debug, PartialEq, Sequence, Clone)]
enum Steps {
    Welcome,
    Name,
    Birthday,
    Birthplace,
    Of,
    Tiers,
    Traits,
    Review,
}

#[derive(Debug, PartialEq, Sequence)]
enum DefaultTiers {
    Family,
    Bests,
    Friends,
    Acquaintances,
    CoWorkers,
}

impl fmt::Display for DefaultTiers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn handle_key_event(key_event: KeyEvent) {}

pub fn run_init(mut terminal: DefaultTerminal) -> Result<(), PplError> {
    let mut app = Init::default();
    loop {
        terminal.draw(|f| render(f, &mut app))?;
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    // KeyCode::Char('Esc') => break Ok(()),
                    KeyCode::Esc => break Ok(()),
                    KeyCode::Enter => match &app.step {
                        Steps::Welcome => {
                            app.input.reset();
                            app.step = next(&app.step).unwrap();
                        }
                        Steps::Name => {
                            if app.input.value() != "".to_string() {
                                let all_text = app.input.value();
                                let mut split = all_text.split(",").collect::<Vec<&str>>();
                                match split.len() {
                                    1 => {
                                        app.name = all_text.parse().unwrap();
                                    }
                                    _ => {
                                        split.reverse();
                                        app.name = split.pop().unwrap().to_string();
                                        split.reverse();
                                        app.nicks = split
                                            .iter()
                                            .map(|s| s.to_string())
                                            .collect::<Vec<String>>();
                                    }
                                }

                                app.input.reset();
                                app.step = next(&app.step).unwrap();
                            }
                        }
                        Steps::Birthday => {
                            if app.input.value() != "".to_string() {
                                let datestr = app.input.value();
                                let parsed = parse_date_string(datestr, Local::now(), Dialect::Us);
                                match parsed {
                                    Ok(parsed) => {
                                        app.bday = parsed.date_naive();
                                        app.input.reset();
                                        app.bday_parse = "".to_string();
                                        app.step = next(&app.step).unwrap();
                                    }
                                    Err(e) => {
                                        app.bday_parse = e.to_string();
                                    }
                                }
                            }
                        }
                        Steps::Birthplace => {
                            if app.input.value() != "".to_string() {
                                app.birthplace = app.input.value().into();
                                app.input.reset();
                                app.step = next(&app.step).unwrap();
                            }
                        }
                        Steps::Of => {
                            if app.input.value() != "".to_string() {
                                app.of_ppl = app
                                    .input
                                    .value()
                                    .to_string()
                                    .split(",")
                                    .map(|s| s.trim().to_string())
                                    .collect::<Vec<String>>();
                                app.input.reset();
                                app.step = next(&app.step).unwrap();
                            }
                        }
                        Steps::Tiers => match app.tier_editing {
                            Selection::Selected => {
                                app.tier_add(app.input.value().to_string());
                                app.input.reset();
                            }
                            Selection::NotSelected => {
                                app.input.reset();
                                app.step = next(&app.step).unwrap();
                            }
                        },
                        Steps::Traits => {
                            app.input.reset();
                            app.step = next(&app.step).unwrap();
                        }
                        Steps::Review => {}
                    },
                    KeyCode::Backspace => {
                        if app.input.value() == "" {
                            match &app.step {
                                Steps::Welcome => {}
                                Steps::Name => {
                                    app.step = previous(&app.step).unwrap();
                                }
                                Steps::Birthday => {
                                    app.name = "".to_string();
                                    app.nicks = vec![];
                                    app.input.reset();
                                    app.step = previous(&app.step).unwrap();
                                }
                                Steps::Birthplace => {
                                    app.bday = NaiveDate::default();
                                    app.input.reset();
                                    app.step = previous(&app.step).unwrap();
                                }
                                Steps::Of => {
                                    app.birthplace = "".to_string();
                                    app.input.reset();
                                    app.step = previous(&app.step).unwrap();
                                }
                                Steps::Tiers => {
                                    app.of_ppl = vec![];
                                    app.input.reset();
                                    app.step = previous(&app.step).unwrap();
                                }
                                Steps::Traits => {
                                    app.input.reset();
                                    app.step = previous(&app.step).unwrap();
                                }
                                Steps::Review => {
                                    app.input.reset();
                                    app.step = previous(&app.step).unwrap();
                                }
                            }
                        } else {
                            app.input.handle_event(&Event::Key(key_event));
                        }
                    }
                    KeyCode::Down => match &app.step {
                        Steps::Tiers => app.tier_state.scroll_up_by(1),
                        Steps::Traits => app.trait_state.scroll_up_by(1),
                        _ => {}
                    },
                    KeyCode::Up => match &app.step {
                        Steps::Tiers => app.tier_state.scroll_down_by(1),
                        Steps::Traits => app.trait_state.scroll_down_by(1),
                        _ => {}
                    },
                    KeyCode::Char(' ') => match &app.step {
                        Steps::Tiers => app.toggle_tier(),
                        Steps::Traits => app.toggle_trait(),
                        _ => {
                            app.input.handle_event(&Event::Key(key_event));
                        }
                    },
                    KeyCode::Tab => match &app.step {
                        Steps::Tiers => app.toggle_tier_editing(),
                        _ => {}
                    },
                    // app.input.handle_event(&Event::Key(key));
                    // KeyCode::Left => self.decrement_counter(),
                    // KeyCode::Right => self.increment_counter(),
                    _ => {
                        app.input.handle_event(&Event::Key(key_event));
                    }
                }
            }
            _ => {}
        }
    }
}

fn render(f: &mut Frame, app: &mut Init) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(7),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.area());

    let msgw = vec![
        Span::raw("welcome to "),
        Span::styled(
            "ppl",
            Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
        ),
        Span::raw(" your local, but everywhere lrm"),
    ];
    let textw = Text::from(Line::from(msgw)).style(Style::default());
    let welcome = Paragraph::new(textw);
    f.render_widget(welcome, chunks[0]);

    let msg = vec![
        Span::raw("press "),
        Span::styled(
            "Esc",
            Style::default().add_modifier(Modifier::BOLD).fg(RED.c500),
        ),
        Span::raw(" to cancel and quit, "),
        Span::styled(
            "Enter",
            Style::default().add_modifier(Modifier::BOLD).fg(BLUE.c500),
        ),
        Span::raw(" to submit and record your responses, "),
        Span::styled(
            "Backspace",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(ORANGE.c500),
        ),
        Span::raw(" previous screen"),
    ];
    let text = Text::from(Line::from(msg)).style(Style::default());
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[1]);

    let width = chunks[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor

    let scroll = app.input.visual_scroll(width as usize);

    let ival = app.input.value().clone();
    let input = Paragraph::new(ival)
        .style(Style::default())
        // .style(match app.input_mode {
        // InputMode::Normal => Style::default(),
        // InputMode::Editing => Style::default().fg(Color::Yellow),
        // })
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Input"));

    let mname: String = app.name.clone();
    let mnicks: String = app.nicks.join(", ");
    let mbday = app.bday.clone();
    let mplace = app.birthplace.clone();
    let mut messages = vec![
        ("name", mname),
        ("nicks", mnicks),
        ("bday", mbday.to_string()),
        ("place", mplace),
    ];

    match app.step {
        Steps::Welcome => {}
        Steps::Name => {}
        Steps::Birthday => {}
        Steps::Birthplace => {}
        Steps::Of => {}
        Steps::Tiers => {
            let oflist = &app.of_ppl.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",");;
            messages.push(("of", oflist.clone()))
        }
        Steps::Traits => {
            let oflist = &app.of_ppl.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",");;
            messages.push(("of", oflist.clone()));
            let tierlist = &app
                .tier_list
                .iter()
                .filter(|t| t.selection == Selection::Selected)
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>()
                .join(",");
            messages.push(("circles", tierlist.clone()))
        }
        Steps::Review => {
            let oflist = &app.of_ppl.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",");;
            messages.push(("of", oflist.clone()));
            let tierlist = &app
                .tier_list
                .iter()
                .filter(|t| t.selection == Selection::Selected)
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>()
                .join(",");
            messages.push(("circles", tierlist.clone()));
            let traitlist = &app
                .trait_list
                .iter()
                .filter(|t| t.selection == Selection::Selected)
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>()
                .join(",");
            messages.push(("traits", traitlist.clone()))
        }
    }

    let messages_v: Vec<ListItem> = messages
        .iter()
        .filter(|(t, i)| **i != "".to_string())
        .enumerate()
        .map(|(i, (t, m))| {
            let content = vec![Line::from(vec![
                Span::styled(*t, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(": {}", m)),
            ])];
            ListItem::new(content)
        })
        .collect();
    let messages =
        List::new(messages_v).block(Block::default().borders(Borders::TOP).title("Responses"));

    // let items = all::<DefaultTiers>().map(|e| e.to_string()).collect::<Vec<_>>();
    // let items = app.tier_list.iter().collect();
    let list_tiers = List::new(&app.tier_list)
        .block(Block::bordered().title("Circles"))
        .style(Style::new().white())
        .highlight_style(Style::new().italic())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::BottomToTop);

    let list_traits = List::new(&app.trait_list)
        .block(Block::bordered().title("Fields"))
        .style(Style::new().white())
        .highlight_style(Style::new().italic())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::BottomToTop);

    match &app.step {
        Steps::Welcome => {
            f.render_widget(
                Span::styled(
                    "welcome",
                    Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                ),
                chunks[2],
            );
        }
        Steps::Name => {
            f.render_widget(
                Span::styled(
                    "> your name? ",
                    Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                ),
                chunks[2],
            );
            f.render_widget(input, chunks[3]);
            f.render_widget(messages, chunks[4]);
        }
        Steps::Birthday => {
            let mut msg = vec![Span::styled(
                "> your bday?",
                Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
            )];
            match app.bday_parse.len() {
                0 => {}
                _ => {
                    msg.push(Span::styled(
                        "> invalid",
                        Style::default().add_modifier(Modifier::BOLD).fg(RED.c500),
                    ));
                    msg.push(Span::raw(&app.bday_parse));
                }
            }
            let text = Text::from(Line::from(msg)).style(Style::default());
            f.render_widget(text, chunks[2]);
            f.render_widget(input, chunks[3]);
            f.render_widget(messages, chunks[4]);
        }
        Steps::Birthplace => {
            f.render_widget(
                Span::styled(
                    "> born where?",
                    Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                ),
                chunks[2],
            );
            f.render_widget(input, chunks[3]);
            f.render_widget(messages, chunks[4]);
        }
        Steps::Of => {
            f.render_widget(
                Span::styled(
                    "> born of? (comma seperated)",
                    Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                ),
                chunks[2],
            );
            f.render_widget(input, chunks[3]);
            f.render_widget(messages, chunks[4]);
        }
        Steps::Tiers => {
            let tcs = app.tier_count_string();
            let msg = vec![
                Span::raw("which of these groupings do you want? "),
                Span::styled("space", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to select/deselect, "),
                Span::styled("tab", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to add new ones, "),
                Span::styled(tcs, Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ];
            let text = Text::from(Line::from(msg)).style(Style::default());
            let help_message = Paragraph::new(text);
            match app.tier_editing {
                Selection::NotSelected => {
                    f.render_widget(help_message, chunks[2]);
                    f.render_stateful_widget(list_tiers, chunks[3], &mut app.tier_state);
                    f.render_widget(messages, chunks[4]);
                }
                Selection::Selected => {
                    f.render_widget(help_message, chunks[2]);
                    f.render_widget(input, chunks[3]);
                    f.render_widget(messages, chunks[4]);
                }
            }
        }
        Steps::Traits => {
            let tcs = app.trait_count_string();
            let msg = vec![
                Span::raw("which of these default fields do you want? "),
                Span::styled("space", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to select/deselect, "),
                Span::styled("tab", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to add new ones, "),
                Span::styled(tcs, Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ];
            let text = Text::from(Line::from(msg)).style(Style::default());
            let help_message = Paragraph::new(text);
            match app.trait_editing {
                Selection::NotSelected => {
                    f.render_widget(help_message, chunks[2]);
                    f.render_stateful_widget(list_traits, chunks[3], &mut app.trait_state);
                    f.render_widget(messages, chunks[4]);
                }
                Selection::Selected => {
                    f.render_widget(help_message, chunks[2]);
                    f.render_widget(input, chunks[3]);
                    f.render_widget(messages, chunks[4]);
                }
            }
        }
        Steps::Review => {
            let msga = vec![
                Span::raw("review your responses below "),
                Span::styled(
                    ":)",
                    Style::default()
                        .add_modifier(Modifier::SLOW_BLINK)
                        .fg(YELLOW.c500),
                ),
            ];
            let texta = Line::from(msga).style(Style::default());
            let msgb = vec![
                Span::raw(" if these look correct, press "),
                Span::styled(
                    "^Enter",
                    Style::default()
                        .add_modifier(Modifier::SLOW_BLINK)
                        .fg(YELLOW.c500),
                ),
                Span::raw(" to complete init "),
            ];
            let textb = Line::from(msgb).style(Style::default());
            let help_message = Paragraph::new(vec![texta, textb]);
            f.render_widget(help_message, chunks[3]);
            f.render_widget(messages, chunks[4]);
        }
    }
}
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;
const BLUEBLUE: Color = BLUE.c500;

fn lcolor(input: &String) -> Color {
    match input.to_lowercase().as_str() {
        "red" => RED.c500,
        "gold" => AMBER.c500,
        "amber" => AMBER.c500,
        "green" => GREEN.c500,
        "teal" => TEAL.c500,
        "cyan" => CYAN.c500,
        "blue" => BLUE.c500,
        "violet" => VIOLET.c500,
        "pink" => PINK.c500,
        "slate" => SLATE.c500,
        _ => TEXT_FG_COLOR,
    }
}

impl From<&TierSelect> for ListItem<'_> {
    fn from(value: &TierSelect) -> Self {
        let line = match value.selection {
            Selection::NotSelected => Line::styled(format!(" ☐ {}", value.name), TEXT_FG_COLOR),
            Selection::Selected => {
                Line::styled(format!(" ✓ {}", value.name), COMPLETED_TEXT_FG_COLOR)
            }
        };
        ListItem::new(line)
    }
}

impl From<&TraitSelect> for ListItem<'_> {
    fn from(value: &TraitSelect) -> Self {
        let line = match value.selection {
            Selection::NotSelected => {
                Line::styled(format!(" ☐ {} {}", value.symbol, value.name), SLATE.c500)
            }
            Selection::Selected => Line::styled(
                format!(" ✓ {} {}", value.symbol, value.name),
                lcolor(&value.color),
            ),
        };
        ListItem::new(line)
    }
}
