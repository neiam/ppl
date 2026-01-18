use crate::data::{
    ContactOps, PplOps, RelationOps, SigDateOps, TierDefaultOps, TierOps, TraitDefaultOps, TraitOps,
};
use crate::do_init::lcolor;
use crate::entities::ppl::Model;
use crate::entities::{
    contact, ppl, relation, sig_date, tier, tier_defaults, trait_defaults, traits,
};
use crate::PplError;
use chrono::{Datelike, NaiveDate};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use enum_iterator::{next, Sequence};
use itertools::Itertools;
use ratatui::prelude::{Constraint, Direction, Layout, Line, Modifier, Span, Style, Stylize, Text};
use ratatui::style::palette::material::AMBER;
use ratatui::style::palette::tailwind::{GREEN, ORANGE, PINK, SLATE, WHITE};
use ratatui::style::Color;
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::widgets::{Block, Borders, List, ListDirection, ListItem, ListState, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use sea_orm::DatabaseConnection;
use std::ops::Index;
use time::{Date, Month, OffsetDateTime};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

struct Tui {
    current_tab: Tabs,
    sigdate_list: Vec<sig_date::Model>,
    sigdate_cal_state: ListState,
    trait_list: Vec<traits::Model>,
    tier_list: Vec<tier::Model>,
    rel_list: Vec<relation::Model>,
    contact_list: Vec<contact::Model>,
    ppl_list: Vec<Model>,
    ppl_state: ListState,
    ppl_editing: bool,
    ppl_field_editing: bool,
    ppl_field_idx: u8,
    ppl_editables: Vec<Editable>,
    ppl_input_a: Input,
    ppl_input_b: Input,
    ppl_input_c: Input,
    ppl_detail_state: ListState,
    ppl_show_contacts: bool,
    ppl_show_dates: bool,
    ppl_show_traits: bool,
    ppl_show_tiers: bool,
    ppl_show_relations: bool,
    def_tier_list: Vec<tier_defaults::Model>,
    def_tier_state: ListState,
    def_trait_list: Vec<trait_defaults::Model>,
    def_trait_state: ListState,
}

#[derive(Debug, PartialEq, Sequence, Clone)]
enum Tabs {
    Ppl,
    Calendar,
    TierSettings,
    TraitSettings,
}

#[derive(Clone, Debug)]
pub enum Editablez {
    Trait,
    Tier,
    Contact,
    Relation,
    SigDate,
}

#[derive(Clone, Debug)]
pub struct Editable {
    tgt: Editablez,
    pub id: i32,
    pub first: String,
    pub second: Option<String>,
    pub third: Option<String>,
}

fn key_in_tier(key: String, tiers: &[tier_defaults::Model]) -> bool {
    let t = tiers
        .iter()
        .map(|t| t.key.to_string())
        .collect::<Vec<String>>();
    t.contains(&key)
}

fn get_key_in_tier(key: String, tiers: &[tier_defaults::Model]) -> bool {
    let t = tiers
        .iter()
        .map(|t| t.key.to_string())
        .collect::<Vec<String>>();

    tiers.iter().find(|t| t.key == key);
    t.contains(&key)
}

impl Tui {
    async fn reload(&mut self, db: &DatabaseConnection) -> Result<(), PplError> {
        self.ppl_list = PplOps::list(db).await?;
        self.def_trait_list = TraitDefaultOps::list(db).await?;
        self.def_tier_list = TierDefaultOps::list(db).await?;
        self.tier_list = TierOps::list(db).await?;
        self.trait_list = TraitOps::list(db).await?;
        self.rel_list = RelationOps::list(db).await?;
        self.contact_list = ContactOps::list(db).await?;
        self.sigdate_list = SigDateOps::list(db).await?;

        Ok(())
    }
}

impl Default for Tui {
    fn default() -> Self {
        Tui {
            current_tab: Tabs::Ppl,
            sigdate_list: vec![],
            sigdate_cal_state: ListState::default(),
            trait_list: vec![],
            tier_list: vec![],
            rel_list: vec![],
            contact_list: vec![],
            ppl_list: vec![],
            ppl_state: ListState::default(),
            ppl_editing: false,
            ppl_field_editing: false,
            ppl_field_idx: 0,
            ppl_editables: vec![],
            ppl_input_a: Input::default(),
            ppl_input_b: Input::default(),
            ppl_input_c: Input::default(),
            ppl_detail_state: ListState::default(),
            ppl_show_contacts: true,
            ppl_show_dates: true,
            ppl_show_traits: true,
            ppl_show_tiers: true,
            ppl_show_relations: true,
            def_tier_list: vec![],
            def_tier_state: ListState::default(),
            def_trait_list: vec![],
            def_trait_state: ListState::default(),
        }
    }
}

fn get_style(mine: u8, cursor: u8, total: u8) -> Color {
    if cursor % total == mine {
        WHITE
    } else {
        SLATE.c500
    }
}

fn get_cal<'a>(
    month: Month,
    year: i32,
    events: &CalendarEventStore,
) -> Monthly<'a, &CalendarEventStore> {
    let default_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(AMBER.c500)
        .bg(SLATE.c800);

    Monthly::new(Date::from_calendar_date(year, month, 1).unwrap(), events)
        .show_month_header(Style::default())
        .default_style(default_style)
}

pub async fn run_tui(
    mut terminal: DefaultTerminal,
    db: DatabaseConnection,
) -> Result<(), PplError> {
    let mut app = Tui::default();
    app.reload(&db).await?;
    loop {
        let _ = terminal.draw(|f| render(f, &mut app));
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Up => match app.current_tab {
                        Tabs::Ppl => match app.ppl_editing {
                            true => match app.ppl_field_editing {
                                true => {}
                                false => app.ppl_detail_state.scroll_up_by(1),
                            },

                            false => app.ppl_state.scroll_up_by(1),
                        },
                        Tabs::TierSettings => app.def_tier_state.scroll_up_by(1),
                        Tabs::TraitSettings => app.def_trait_state.scroll_up_by(1),
                        _ => {}
                    },
                    KeyCode::Down => match app.current_tab {
                        Tabs::Ppl => match app.ppl_editing {
                            true => match app.ppl_field_editing {
                                true => {}
                                false => app.ppl_detail_state.scroll_down_by(1),
                            },
                            false => app.ppl_state.scroll_down_by(1),
                        },
                        Tabs::TierSettings => app.def_tier_state.scroll_down_by(1),
                        Tabs::TraitSettings => app.def_trait_state.scroll_down_by(1),
                        _ => {}
                    },
                    KeyCode::Tab => match app.ppl_field_editing {
                        true => {
                            app.ppl_field_idx += 1;
                        }
                        false => app.current_tab = next(&app.current_tab).unwrap_or(Tabs::Ppl),
                    },
                    KeyCode::Esc => break Ok(()),
                    KeyCode::Char('e') => {
                        if app.current_tab == Tabs::Ppl {
                            match app.ppl_field_editing {
                                true => {
                                    default_editing_handler(&mut app, &key_event);
                                }
                                false => {
                                    app.ppl_editing = !app.ppl_editing;
                                }
                            }
                        }
                    }
                    KeyCode::Char('f') => {
                        default_editing_handler(&mut app, &key_event);
                    }
                    KeyCode::Char('c') => match app.current_tab {
                        Tabs::Ppl => app.ppl_show_contacts = !app.ppl_show_contacts,
                        Tabs::Calendar => {}
                        _ => {}
                    },
                    KeyCode::Char('d') => if app.current_tab == Tabs::Ppl { app.ppl_show_dates = !app.ppl_show_dates },
                    KeyCode::Char('r') => if app.current_tab == Tabs::Ppl { app.ppl_show_traits = !app.ppl_show_traits },
                    KeyCode::Char('i') => if app.current_tab == Tabs::Ppl { app.ppl_show_tiers = !app.ppl_show_tiers },
                    KeyCode::Char('l') => if app.current_tab == Tabs::Ppl { app.ppl_show_relations = !app.ppl_show_relations },
                    KeyCode::Enter => {
                        if app.current_tab == Tabs::Ppl
                            && app.ppl_editing && app.ppl_detail_state.selected().is_some() {
                                let idx = &app.ppl_detail_state.selected().unwrap();
                                let e = app.ppl_editables.get(*idx).unwrap();
                                if !app.ppl_field_editing {
                                    // let k
                                    app.ppl_input_a = e.first.clone().into();
                                    app.ppl_input_b =
                                        e.second.clone().unwrap_or("".to_string()).into();
                                    app.ppl_input_c =
                                        e.third.clone().unwrap_or("".to_string()).into();
                                } else {
                                    let new_editable = Editable {
                                        tgt: e.tgt.clone(),
                                        id: e.id,
                                        first: app.ppl_input_a.value().parse().unwrap(),
                                        second: Option::from(app.ppl_input_b.value().to_string()),
                                        third: Option::from(app.ppl_input_c.value().to_string()),
                                    };

                                    match e.tgt {
                                        Editablez::Trait => {
                                            TraitOps::updatee(&db, new_editable).await;
                                        }
                                        Editablez::Tier => {
                                            TierOps::updatee(&db, new_editable).await;
                                        }
                                        Editablez::Contact => {
                                            ContactOps::updatee(&db, new_editable).await;
                                        }
                                        Editablez::Relation => {
                                            RelationOps::updatee(&db, new_editable).await;
                                        }
                                        Editablez::SigDate => {
                                            SigDateOps::updatee(&db, new_editable).await;
                                        }
                                    };
                                    app.reload(&db).await?;
                                }
                                app.ppl_field_editing = !app.ppl_field_editing;
                            }
                    }
                    _ => {
                        default_editing_handler(&mut app, &key_event);
                    }
                }
            }

            _ => {}
        }
    }
}

fn default_editing_handler(app: &mut Tui, key_event: &KeyEvent) {
    if app.ppl_editing {
        let idx = &app.ppl_detail_state.selected().unwrap();
        let e = app.ppl_editables.get(*idx).unwrap();
        match e.tgt {
            Editablez::Trait => match app.ppl_field_idx % 2 {
                0 => {
                    app.ppl_input_a.handle_event(&Event::Key(*key_event));
                }
                1 => {
                    app.ppl_input_b.handle_event(&Event::Key(*key_event));
                }
                _ => {}
            },
            Editablez::Tier => match app.ppl_field_idx % 3 {
                0 => {
                    app.ppl_input_a.handle_event(&Event::Key(*key_event));
                }
                1 => {
                    app.ppl_input_b.handle_event(&Event::Key(*key_event));
                }
                2 => {
                    app.ppl_input_c.handle_event(&Event::Key(*key_event));
                }
                _ => {}
            },
            Editablez::Contact => match app.ppl_field_idx % 3 {
                0 => {
                    app.ppl_input_a.handle_event(&Event::Key(*key_event));
                }
                1 => {
                    app.ppl_input_b.handle_event(&Event::Key(*key_event));
                }
                2 => {
                    app.ppl_input_c.handle_event(&Event::Key(*key_event));
                }
                _ => {}
            },
            Editablez::Relation => match app.ppl_field_idx % 3 {
                0 => {
                    app.ppl_input_a.handle_event(&Event::Key(*key_event));
                }
                1 => {
                    app.ppl_input_b.handle_event(&Event::Key(*key_event));
                }
                2 => {
                    app.ppl_input_c.handle_event(&Event::Key(*key_event));
                }
                _ => {}
            },
            Editablez::SigDate => match app.ppl_field_idx % 3 {
                0 => {
                    app.ppl_input_a.handle_event(&Event::Key(*key_event));
                }
                1 => {
                    app.ppl_input_b.handle_event(&Event::Key(*key_event));
                }
                2 => {
                    app.ppl_input_c.handle_event(&Event::Key(*key_event));
                }
                _ => {}
            },
        }
        {}
    }
}

fn render(f: &mut Frame, app: &mut Tui) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Percentage(100),
                Constraint::Length(1),
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
        Span::raw(" :)"),
    ];
    let textw = Text::from(Line::from(msgw)).style(Style::default());
    let welcome = Paragraph::new(textw);
    f.render_widget(welcome, chunks[0]);

    match app.current_tab {
        Tabs::Ppl => {
            match app.ppl_editing {
                true => {
                    let msgw = vec![
                        Span::styled(
                            "Enter",
                            Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                        ),
                        Span::raw(" to modify a ppl entry. "),
                        Span::styled(
                            "Shift+Enter",
                            Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                        ),
                        Span::raw(" to add a ppl entry"),
                    ];
                    let textw = Text::from(Line::from(msgw)).style(Style::default());
                    let welcome = Paragraph::new(textw);
                    f.render_widget(welcome, chunks[2]);
                }
                false => {
                    let mut msgw = vec![
                        Span::styled(
                            "Tab",
                            Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                        ),
                        Span::raw(" to switch views. "),
                        Span::styled(
                            "Enter",
                            Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                        ),
                        Span::raw(" to add ppl. "),
                        Span::styled("E", Style::default().add_modifier(Modifier::BOLD).fg(WHITE)),
                        Span::raw(" to edit the selected ppl. "),
                    ];
                    if app.ppl_show_contacts {
                        msgw.push(Span::styled(
                            "C",
                            Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                        ));
                    } else {
                        msgw.push(Span::styled(
                            "C",
                            Style::default().add_modifier(Modifier::BOLD).fg(SLATE.c500),
                        ));
                    }
                    msgw.push(Span::raw("/"));

                    if app.ppl_show_dates {
                        msgw.push(Span::styled(
                            "D",
                            Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                        ));
                    } else {
                        msgw.push(Span::styled(
                            "D",
                            Style::default().add_modifier(Modifier::BOLD).fg(SLATE.c500),
                        ));
                    }
                    msgw.push(Span::raw("/"));

                    if app.ppl_show_traits {
                        msgw.push(Span::styled(
                            "R",
                            Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                        ));
                    } else {
                        msgw.push(Span::styled(
                            "R",
                            Style::default().add_modifier(Modifier::BOLD).fg(SLATE.c500),
                        ));
                    }
                    msgw.push(Span::raw("/"));

                    if app.ppl_show_tiers {
                        msgw.push(Span::styled(
                            "I",
                            Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                        ));
                    } else {
                        msgw.push(Span::styled(
                            "I",
                            Style::default().add_modifier(Modifier::BOLD).fg(SLATE.c500),
                        ));
                    }
                    msgw.push(Span::raw("/"));

                    if app.ppl_show_relations {
                        msgw.push(Span::styled(
                            "L",
                            Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                        ));
                    } else {
                        msgw.push(Span::styled(
                            "L",
                            Style::default().add_modifier(Modifier::BOLD).fg(SLATE.c500),
                        ));
                    }

                    msgw.push(Span::raw(
                        " to toggle Contacts/Dates/tRaits/tIers/reLations",
                    ));

                    let textw = Text::from(Line::from(msgw)).style(Style::default());
                    let welcome = Paragraph::new(textw);
                    f.render_widget(welcome, chunks[2]);
                }
            }

            let layout_ppl = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
                .split(chunks[1]);
            let layout_edit = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .split(layout_ppl[1]);
            let ppls: Vec<PplAndProps> = app
                .ppl_list
                .iter()
                .map(|p| PplAndProps {
                    ppl: p.clone(),
                    tiers: app
                        .tier_list
                        .iter()
                        .filter(|t| t.ppl_id == p.id)
                        .map(|t| t.to_owned())
                        .collect::<Vec<tier::Model>>(),
                    tier_defaults: app.def_tier_list.clone(),
                })
                .collect::<Vec<PplAndProps>>();

            let ppl = List::new(ppls)
                .block(Block::bordered().title("ppl"))
                .style(Style::new().white())
                .highlight_style(Style::new().italic())
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            f.render_stateful_widget(&ppl, layout_ppl[0], &mut app.ppl_state);

            let idx = app.ppl_state.selected();
            match idx {
                None => {}
                Some(val) => {
                    let curr = app.ppl_list.index(val);

                    let mut msgs: Vec<Line> = vec![];
                    let mut editables: Vec<Editable> = vec![];

                    app.trait_list.sort_by_key(|k| {
                        (
                            key_in_tier(k.key.clone(), &app.def_tier_list),
                            k.key.clone(),
                            k.value.clone(),
                        )
                    });
                    let curr_traits = app
                        .trait_list
                        .iter()
                        .filter(|t| t.ppl_id == curr.id)
                        .map(|t| t.to_owned())
                        .collect::<Vec<traits::Model>>();
                    let trait_defaults = app
                        .def_trait_list
                        .iter()
                        .filter(|t| !t.is_date && !t.is_contact)
                        .collect::<Vec<&trait_defaults::Model>>();
                    if app.ppl_show_traits {
                        for t in &curr_traits {
                            editables.push(Editable {
                                tgt: Editablez::Trait,
                                id: t.id,
                                first: t.key.clone(),
                                second: Option::from(t.value.clone()),
                                third: t.hidden.clone().to_string().into(),
                            });
                            match trait_defaults.iter().find(|f| f.key == t.key) {
                                None => msgs.push(Line::from(vec![
                                    Span::styled(
                                        format!("ℹ️ {}: ", t.key.clone()),
                                        Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                    ),
                                    Span::raw(t.value.clone()),
                                ])),
                                Some(td) => msgs.push(Line::from(vec![
                                    Span::styled(
                                        format!("{} {}: ", td.symbol, t.key.clone()),
                                        Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                    ),
                                    Span::raw(t.value.clone()),
                                ])),
                            }
                        }
                    }

                    let curr_tiers = app
                        .tier_list
                        .iter()
                        .filter(|t| t.ppl_id == curr.id)
                        .map(|t| t.to_owned())
                        .collect::<Vec<tier::Model>>();

                    if app.ppl_show_tiers {
                        for t in &curr_tiers {
                            let othersym = app.def_tier_list.iter().find(|dt| dt.key == t.name);
                            editables.push(Editable {
                                id: t.id,
                                tgt: Editablez::Tier,
                                first: t.name.clone(),
                                second: t.color.clone(),
                                third: t.symbol.clone(),
                            });
                            match (&t.symbol, othersym) {
                                (None, Some(other)) => msgs.push(Line::from(vec![
                                    Span::styled(
                                        format!(
                                            "{} {}: ",
                                            &other.symbol.clone().unwrap(),
                                            t.name.clone()
                                        ),
                                        Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                    ),
                                    Span::raw(t.name.clone()),
                                ])),
                                (Some(main), None) => msgs.push(Line::from(vec![
                                    Span::styled(
                                        format!("{} {}: ", main, t.name.clone()),
                                        Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                    ),
                                    Span::raw(t.name.clone()),
                                ])),
                                (None, None) => msgs.push(Line::from(vec![
                                    Span::styled(
                                        format!("ℹ️ {}: ", t.name.clone()),
                                        Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                    ),
                                    Span::raw(t.name.clone()),
                                ])),
                                // _ => {}
                                (Some(main), Some(_other)) => msgs.push(Line::from(vec![
                                    Span::styled(
                                        format!("{} {}: ", main, t.name.clone()),
                                        Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                    ),
                                    Span::raw(t.name.clone()),
                                ])),
                            }
                        }
                    }

                    let curr_rels = app
                        .rel_list
                        .iter()
                        .filter(|t| t.ppl_id_b == curr.id)
                        .map(|t| t.to_owned())
                        .collect::<Vec<relation::Model>>();
                    if app.ppl_show_relations {
                        for t in &curr_rels {
                            let i2 = t.date_entered.map(|d| d.to_string());
                            let i3 = t.date_ended.map(|d| d.to_string());
                            editables.push(Editable {
                                id: t.id,
                                tgt: Editablez::Relation,
                                first: t.r#type.clone(),
                                second: i2,
                                third: i3,
                            });

                            msgs.push(Line::from(vec![
                                Span::styled(
                                    format!("🫂 {}: ", t.r#type.clone()),
                                    Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                ),
                                Span::raw(format!("{:?} - {:?}", t.date_entered, t.date_ended)),
                            ]))
                        }
                    }

                    let curr_dates = app
                        .sigdate_list
                        .iter()
                        .filter(|t| t.ppl_id == curr.id)
                        .map(|t| t.to_owned())
                        .collect::<Vec<sig_date::Model>>();
                    let date_traits = app
                        .def_trait_list
                        .iter()
                        .filter(|t| t.is_date && t.enabled)
                        .collect::<Vec<&trait_defaults::Model>>();
                    if app.ppl_show_dates {
                        for t in &curr_dates {
                            editables.push(Editable {
                                id: t.id,
                                tgt: Editablez::SigDate,
                                first: t.event.clone(),
                                second: Option::from(t.date.to_string()),
                                third: Option::from(t.do_remind.to_string()),
                            });
                            match (date_traits.iter().find(|f| f.key == t.event), t.do_remind) {
                                (Some(dt), _) => msgs.push(Line::from(vec![
                                    Span::styled(
                                        format!("{} {}: ", dt.symbol, t.event.clone()),
                                        Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                    ),
                                    Span::raw(t.date.clone().to_string()),
                                ])),
                                (None, true) => msgs.push(Line::from(vec![
                                    Span::styled(
                                        format!("⏰ {}: ", t.event.clone()),
                                        Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                    ),
                                    Span::raw(t.date.clone().to_string()),
                                ])),
                                (None, false) => msgs.push(Line::from(vec![
                                    Span::styled(
                                        format!("📅 {}: ", t.event.clone()),
                                        Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                    ),
                                    Span::raw(t.date.clone().to_string()),
                                ])),
                            }
                        }
                    }

                    let contacts = app
                        .contact_list
                        .iter()
                        .filter(|t| t.ppl_id == curr.id)
                        .map(|t| t.to_owned())
                        .collect::<Vec<contact::Model>>();
                    let contact_traits = app
                        .def_trait_list
                        .iter()
                        .filter(|t| t.is_contact && t.enabled)
                        .collect::<Vec<&trait_defaults::Model>>();
                    if app.ppl_show_contacts {
                        for t in &contacts {
                            editables.push(Editable {
                                id: t.id,
                                tgt: Editablez::Contact,
                                first: t.r#type.clone(),
                                second: Option::from(
                                    t.designator.clone().unwrap_or("".to_string()),
                                ),
                                third: Option::from(t.value.clone()),
                            });

                            match contact_traits.iter().find(|f| f.key == t.r#type) {
                                Some(dt) => msgs.push(Line::from(vec![
                                    Span::styled(
                                        format!("{} {}: ", dt.symbol, t.r#type.clone()),
                                        Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                    ),
                                    Span::raw(t.value.clone().to_string()),
                                ])),
                                None => msgs.push(Line::from(vec![
                                    Span::styled(
                                        format!("📇 {}: ", t.r#type.clone()),
                                        Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                    ),
                                    Span::raw(t.value.clone().to_string()),
                                ])),
                            }
                        }
                    }

                    app.ppl_editables = editables.clone();
                    let list_contact = List::new(msgs.clone())
                        .block(Block::bordered().title("Circles"))
                        .style(Style::new().white())
                        .highlight_style(Style::new().italic())
                        .highlight_symbol(">>")
                        .repeat_highlight_symbol(true)
                        .direction(ListDirection::TopToBottom);

                    let textw = Text::from(msgs.clone()).style(Style::default());
                    let paragraph = Paragraph::new(textw);
                    match app.ppl_editing {
                        true => match app.ppl_field_editing {
                            true => {
                                let ced = editables
                                    .get(app.ppl_detail_state.selected().unwrap())
                                    .unwrap();
                                match ced.tgt.clone() {
                                    Editablez::Trait => {
                                        let input_a = Paragraph::new(app.ppl_input_a.value())
                                            .block(
                                                Block::default().borders(Borders::ALL).title("key"),
                                            )
                                            .style(get_style(0, app.ppl_field_idx, 2));
                                        f.render_widget(input_a, layout_edit[1]);

                                        let input_b = Paragraph::new(app.ppl_input_b.value())
                                            .block(
                                                Block::default()
                                                    .borders(Borders::ALL)
                                                    .title("value"),
                                            )
                                            .style(get_style(1, app.ppl_field_idx, 2));
                                        f.render_widget(input_b, layout_edit[2]);
                                    }
                                    Editablez::Tier => {
                                        let input_b = Paragraph::new(app.ppl_input_a.value())
                                            .block(
                                                Block::default()
                                                    .borders(Borders::ALL)
                                                    .title("name"),
                                            )
                                            .style(get_style(0, app.ppl_field_idx, 3));
                                        f.render_widget(input_b, layout_edit[1]);
                                        let input_b = Paragraph::new(app.ppl_input_b.value())
                                            .block(
                                                Block::default()
                                                    .borders(Borders::ALL)
                                                    .title("color"),
                                            )
                                            .style(get_style(1, app.ppl_field_idx, 3));
                                        f.render_widget(input_b, layout_edit[2]);

                                        let input_c = Paragraph::new(app.ppl_input_c.value())
                                            .block(
                                                Block::default()
                                                    .borders(Borders::ALL)
                                                    .title("symbol"),
                                            )
                                            .style(get_style(2, app.ppl_field_idx, 3));
                                        f.render_widget(input_c, layout_edit[3]);
                                    }
                                    Editablez::Contact => {
                                        let input_a = Paragraph::new(app.ppl_input_a.value())
                                            .block(
                                                Block::default().borders(Borders::ALL).title("typ"),
                                            )
                                            .style(get_style(0, app.ppl_field_idx, 3));
                                        f.render_widget(input_a, layout_edit[1]);

                                        let input_b = Paragraph::new(app.ppl_input_b.value())
                                            .block(
                                                Block::default()
                                                    .borders(Borders::ALL)
                                                    .title("designator"),
                                            )
                                            .style(get_style(1, app.ppl_field_idx, 3));
                                        f.render_widget(input_b, layout_edit[2]);
                                        let input_c = Paragraph::new(app.ppl_input_c.value())
                                            .block(
                                                Block::default()
                                                    .borders(Borders::ALL)
                                                    .title("value"),
                                            )
                                            .style(get_style(2, app.ppl_field_idx, 3));
                                        f.render_widget(input_c, layout_edit[3]);
                                    }
                                    Editablez::Relation => {
                                        let input_a = Paragraph::new(app.ppl_input_a.value())
                                            .block(
                                                Block::default().borders(Borders::ALL).title("key"),
                                            )
                                            .style(get_style(0, app.ppl_field_idx, 3));
                                        f.render_widget(input_a, layout_edit[1]);

                                        let input_b = Paragraph::new(app.ppl_input_b.value())
                                            .block(
                                                Block::default()
                                                    .borders(Borders::ALL)
                                                    .title("start"),
                                            )
                                            .style(get_style(1, app.ppl_field_idx, 3));
                                        f.render_widget(input_b, layout_edit[2]);

                                        let input_c = Paragraph::new(app.ppl_input_c.value())
                                            .block(
                                                Block::default().borders(Borders::ALL).title("end"),
                                            )
                                            .style(get_style(2, app.ppl_field_idx, 3));
                                        f.render_widget(input_c, layout_edit[3]);
                                    }
                                    Editablez::SigDate => {
                                        let input_a = Paragraph::new(app.ppl_input_a.value())
                                            .block(
                                                Block::default()
                                                    .borders(Borders::ALL)
                                                    .title("date"),
                                            )
                                            .style(get_style(0, app.ppl_field_idx, 3));
                                        f.render_widget(input_a, layout_edit[1]);

                                        let input_b = Paragraph::new(app.ppl_input_b.value())
                                            .block(
                                                Block::default()
                                                    .borders(Borders::ALL)
                                                    .title("date"),
                                            )
                                            .style(get_style(1, app.ppl_field_idx, 3));
                                        f.render_widget(input_b, layout_edit[2]);

                                        let input_c = Paragraph::new(app.ppl_input_c.value())
                                            .block(
                                                Block::default()
                                                    .borders(Borders::ALL)
                                                    .title("do_remind"),
                                            )
                                            .style(get_style(2, app.ppl_field_idx, 3));
                                        f.render_widget(input_c, layout_edit[3]);
                                    }
                                }
                                let title = match &curr.nick {
                                    None => {
                                        format!("Editing {}", curr.name.clone())
                                    }
                                    Some(n) => {
                                        format!("Editing {} ({})", curr.name.clone(), n)
                                    }
                                };
                                let block = Block::new()
                                    .borders(Borders::LEFT)
                                    .title(title)
                                    .style(Style::default().fg(ORANGE.c500));
                                f.render_widget(paragraph.clone().block(block), layout_edit[0])
                            }
                            false => {
                                let title = match &curr.nick {
                                    None => {
                                        format!("Editing {}", curr.name.clone())
                                    }
                                    Some(n) => {
                                        format!("Editing {} ({})", curr.name.clone(), n)
                                    }
                                };
                                let block = Block::new()
                                    .borders(Borders::LEFT)
                                    .title(title)
                                    .style(Style::default().fg(AMBER.c500));
                                f.render_stateful_widget(
                                    list_contact.block(block),
                                    layout_ppl[1],
                                    &mut app.ppl_detail_state,
                                );
                            }
                        },
                        false => {
                            let title = match &curr.nick {
                                None => {
                                    curr.name.clone().to_string()
                                }
                                Some(n) => {
                                    format!("{} ({})", curr.name.clone(), n)
                                }
                            };
                            let block = Block::new().borders(Borders::ALL).title(title);
                            f.render_widget(paragraph.clone().block(block), layout_ppl[1]);
                        }
                    };
                }
            }
        }
        Tabs::Calendar => {
            let layout_cal = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(26), Constraint::Percentage(74)])
                .split(chunks[1]);

            let mut start = OffsetDateTime::now_local()
                .unwrap()
                .date()
                .replace_month(Month::January)
                .unwrap()
                .replace_day(1)
                .unwrap();
            let as_upcoming = &app
                .sigdate_list
                .iter()
                .map(|d| {
                    let mut n = d.clone();
                    n.date = NaiveDate::from_ymd_opt(start.year(), n.date.month(), n.date.day())
                        .unwrap();
                    let p = app
                        .ppl_list
                        .iter()
                        .find(|p| p.id == n.ppl_id)
                        .unwrap()
                        .clone();
                    let tr = app
                        .trait_list
                        .iter()
                        .filter(|p| p.id == n.ppl_id)
                        .map(|t| t.to_owned())
                        .collect::<Vec<traits::Model>>();
                    crate::do_tui::SigDateAndProps {
                        date: n,
                        ppl: p,
                        traits: tr,
                        trait_defaults: app.def_trait_list.clone(),
                    }
                })
                .sorted_by(|a, b| a.date.date.cmp(&b.date.date))
                .collect::<Vec<SigDateAndProps>>();
            let mut event_list = CalendarEventStore::today(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Blue),
            );

            for u in as_upcoming.iter().filter(|d| d.date.do_remind) {
                event_list.add(
                    // Date::from_calendar_date(start.year(), Month::try_from(4 as u8).expect("oops"), 15 as u8).expect("doubleoops"),
                    Date::from_calendar_date(
                        start.year(),
                        Month::try_from(u.date.date.month() as u8).expect("oops"),
                        u.date.date.day() as u8,
                    )
                    .expect("doubleoops"),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(GREEN.c300)
                        .bg(PINK.c500),
                );
            }

            // let _ = as_upcoming.iter().map(|u| {
            //     let _ = event_list.add(
            //         Date::from_calendar_date(start.year(), Month::try_from(4 as u8).expect("oops"), 15 as u8).expect("doubleoops"),
            //         // Date::from_calendar_date(start.year(), Month::try_from(u.date.month() as u8).expect("oops"), u.date.day() as u8).expect("doubleoops"),
            //         Style::default()
            //             .add_modifier(Modifier::BOLD)
            //             .fg(GREEN.c300)
            //             .bg(PINK.c500),
            //     );
            // });

            let rows = Layout::vertical([Constraint::Ratio(1, 2); 2]).split(layout_cal[1]);
            let cols = rows.iter().flat_map(|row| {
                Layout::horizontal([Constraint::Ratio(1, 6); 6])
                    .split(*row)
                    .to_vec()
            });
            for col in cols {
                let cal = get_cal(start.month(), start.year(), &event_list);
                f.render_widget(cal, col);
                start = start.replace_month(start.month().next()).unwrap();
            }

            let dates = List::new(as_upcoming)
                .block(Block::bordered().title("upcoming"))
                .style(Style::new().white())
                .highlight_style(Style::new().italic())
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            f.render_stateful_widget(dates, layout_cal[0], &mut app.sigdate_cal_state);

            // let layout_cal_detail = Layout::default()
            //     .direction(Direction::Horizontal)
            //     .margin(0)
            //     .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            //     .split(chunks[1]);

            let msgw = vec![
                Span::styled(
                    "Tab",
                    Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                ),
                Span::raw(" to switch views. "),
                Span::styled("E", Style::default().add_modifier(Modifier::BOLD).fg(WHITE)),
                Span::raw(" to edit the enabled tiers. "),
                Span::styled("C", Style::default().add_modifier(Modifier::BOLD).fg(WHITE)),
                Span::raw(" to enable/disable the various builtin calendars. "),
            ];
            let textw = Text::from(Line::from(msgw)).style(Style::default());
            let welcome = Paragraph::new(textw);
            f.render_widget(welcome, chunks[2]);
        }
        Tabs::TierSettings => {
            let ppl = List::new(&app.def_tier_list)
                .block(Block::bordered().title("tiers"))
                .style(Style::new().white())
                .highlight_style(Style::new().italic())
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            f.render_stateful_widget(ppl, chunks[1], &mut app.def_tier_state);
        }
        Tabs::TraitSettings => {
            let ppl = List::new(&app.def_trait_list)
                .block(Block::bordered().title("traits"))
                .style(Style::new().white())
                .highlight_style(Style::new().italic())
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            f.render_stateful_widget(ppl, chunks[1], &mut app.def_trait_state);
        }
    }
}

struct PplAndProps {
    ppl: ppl::Model,
    tier_defaults: Vec<tier_defaults::Model>,
    tiers: Vec<tier::Model>,
}

struct SigDateAndProps {
    date: sig_date::Model,
    ppl: ppl::Model,
    traits: Vec<traits::Model>,
    trait_defaults: Vec<trait_defaults::Model>,
}

impl From<PplAndProps> for ListItem<'_> {
    fn from(value: PplAndProps) -> Self {
        // value.date_up;
        let line =
            match value.ppl.me {
                true => Line::styled(
                    format!(" 🌟 {}", value.ppl.nick.unwrap_or(value.ppl.name)),
                    WHITE,
                ),
                false => match value.tiers.len() {
                    0 => Line::styled(
                        format!("   {}", value.ppl.nick.unwrap_or(value.ppl.name)),
                        WHITE,
                    ),
                    _ => {
                        let t = value.tiers.first().unwrap();
                        match value.tier_defaults.iter().find(|q| q.key == t.name) {
                            None => Line::styled(
                                format!("   {}", value.ppl.nick.unwrap_or(value.ppl.name)),
                                WHITE,
                            ),
                            Some(tierd) => {
                                match t.symbol.clone().unwrap_or("".to_string()).as_str() {
                                    "" => Line::styled(
                                        format!(
                                            " {} {}",
                                            tierd.symbol.clone().unwrap_or("".to_string()),
                                            value.ppl.nick.unwrap_or(value.ppl.name)
                                        ),
                                        lcolor(&t.color.clone().unwrap_or(
                                            tierd.color.clone().unwrap_or("".to_string()),
                                        )),
                                    ),
                                    otherwise => Line::styled(
                                        format!(
                                            " {} {}",
                                            otherwise,
                                            value.ppl.nick.unwrap_or(value.ppl.name)
                                        ),
                                        lcolor(&t.color.clone().unwrap_or(
                                            tierd.color.clone().unwrap_or("".to_string()),
                                        )),
                                    ),
                                }
                            }
                        }
                    }
                },
            };
        ListItem::new(line)
    }
}

impl From<&SigDateAndProps> for ListItem<'_> {
    fn from(value: &SigDateAndProps) -> Self {
        let n = value.ppl.nick.clone().unwrap_or(value.ppl.name.clone());
        let default = value
            .trait_defaults
            .iter()
            .find(|q| q.key == value.date.event && q.is_date);
        let line = match (value.date.do_remind, default) {
            (true, Some(t)) => Line::styled(
                format!(" ✓ {} {:6} {}", t.symbol, n, value.date.date),
                WHITE,
            ),
            (true, None) => Line::styled(
                format!(" ✓ 📅 {:6} {} ({})", n, value.date.date, value.date.event),
                WHITE,
            ),
            (false, Some(t)) => Line::styled(
                format!(" ☐ {} {:6} {}", t.symbol, n, value.date.date),
                SLATE.c500,
            ),

            (false, None) => Line::styled(
                format!(" ☐ 📅 {:6} {} ({})", n, value.date.date, value.date.event),
                SLATE.c500,
            ),
        };

        ListItem::new(line)
    }
}

impl From<&ppl::Model> for ListItem<'_> {
    fn from(value: &Model) -> Self {
        let line = match value.me {
            true => Line::styled(format!(" 🌟 {}", value.name), WHITE),
            false => Line::styled(format!("   {}", value.name), WHITE),
        };
        ListItem::new(line)
    }
}

impl From<&tier_defaults::Model> for ListItem<'_> {
    fn from(value: &tier_defaults::Model) -> Self {
        let line = match value.enabled {
            true => Line::styled(
                format!(" ✓ {:?} {}", value.symbol, value.key),
                lcolor(&value.color.clone().unwrap_or("".to_string())),
            ),
            false => Line::styled(format!(" ☐ {:?} {}", value.symbol, value.key), SLATE.c500),
        };

        ListItem::new(line)
    }
}
impl From<&trait_defaults::Model> for ListItem<'_> {
    fn from(value: &trait_defaults::Model) -> Self {
        let line = match value.enabled {
            true => Line::styled(
                format!(" ✓ {} {}", value.symbol, value.key),
                lcolor(&value.color),
            ),
            false => Line::styled(format!(" ☐ {} {}", value.symbol, value.key), SLATE.c500),
        };

        ListItem::new(line)
    }
}

impl From<&sig_date::Model> for ListItem<'_> {
    fn from(value: &sig_date::Model) -> Self {
        let line = match value.do_remind {
            true => Line::styled(
                format!(" ✓ {} {} {}", value.ppl_id, value.event, value.date),
                WHITE,
            ),
            false => Line::styled(
                format!(" ☐ {} {} {}", value.ppl_id, value.event, value.date),
                SLATE.c500,
            ),
        };

        ListItem::new(line)
    }
}
