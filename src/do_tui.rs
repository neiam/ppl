use crate::data::{
    ContactOps, PplOps, RelationOps, SigDateOps, TierDefaultOps, TierOps, TraitDefaultOps, TraitOps,
};
use crate::entities::ppl::Model;
use crate::entities::prelude::{Ppl, TierDefaults, TraitDefaults};
use crate::entities::{
    contact, ppl, relation, sig_date, tier, tier_defaults, trait_defaults, traits,
};
use crate::PplError;
use chrono::NaiveDate;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use enum_iterator::{next, Sequence};
use ratatui::prelude::{Constraint, Direction, Layout, Line, Modifier, Span, Style, Stylize, Text};
use ratatui::style::palette::material::AMBER;
use ratatui::style::palette::tailwind::{GREEN, ORANGE, SLATE, WHITE};
use ratatui::style::{Color, Styled};
use ratatui::widgets::{Block, Borders, List, ListDirection, ListItem, ListState, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use sea_orm::DatabaseConnection;
use std::ops::Index;
use tokio::task::id;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

struct Tui {
    current_tab: Tabs,
    sigdate_list: Vec<sig_date::Model>,
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

fn key_in_tier(key: String, tiers: &Vec<tier_defaults::Model>) -> bool {
    let t = tiers
        .iter()
        .map(|t| t.key.to_string())
        .collect::<Vec<String>>();
    t.contains(&key)
}

fn get_key_in_tier(key: String, tiers: &Vec<tier_defaults::Model>) -> bool {
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

pub async fn run_tui(
    mut terminal: DefaultTerminal,
    db: DatabaseConnection,
) -> Result<(), PplError> {
    let mut app = Tui::default();
    app.reload(&db).await?;
    loop {
        terminal.draw(|f| render(f, &mut app));
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
                            app.ppl_field_idx = app.ppl_field_idx + 1;
                        }
                        false => {
                            app.current_tab = next(&app.current_tab).unwrap_or_else(|| Tabs::Ppl)
                        }
                    },
                    KeyCode::Esc => break Ok(()),
                    KeyCode::Char('e') => match app.current_tab {
                        Tabs::Ppl => match app.ppl_field_editing {
                            true => {
                                default_editing_handler(&mut app, &key_event);
                            }
                            false => {
                                app.ppl_editing = !app.ppl_editing;
                            }
                        },
                        _ => {}
                    },
                    KeyCode::Char('f') => {
                        default_editing_handler(&mut app, &key_event);
                    }
                    KeyCode::Enter => match app.current_tab {
                        Tabs::Ppl => match app.ppl_editing {
                            true => {
                                if app.ppl_detail_state.selected().is_some() {
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
                                            second: Option::from(
                                                app.ppl_input_b.value().to_string(),
                                            ),
                                            third: Option::from(
                                                app.ppl_input_c.value().to_string(),
                                            ),
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
                            false => {}
                        },
                        _ => {}
                    },
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
    match app.ppl_editing {
        true => {
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
            match app.ppl_field_idx {
                _ => {}
            }
        }
        false => {}
    }
}

fn render(f: &mut Frame, app: &mut Tui) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(1), Constraint::Length(15)].as_ref())
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
            let ppl = List::new(&app.ppl_list)
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
                        .filter(|t| t.is_date == false && t.is_contact == false)
                        .collect::<Vec<&trait_defaults::Model>>();
                    for t in &curr_traits {
                        editables.push(Editable {
                            tgt: Editablez::Trait,
                            id: t.id.clone(),
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

                    let curr_tiers = app
                        .tier_list
                        .iter()
                        .filter(|t| t.ppl_id == curr.id)
                        .map(|t| t.to_owned())
                        .collect::<Vec<tier::Model>>();

                    for t in &curr_tiers {
                        let othersym = app.def_tier_list.iter().find(|dt| dt.key == t.name);
                        editables.push(Editable {
                            id: t.id.clone(),
                            tgt: Editablez::Tier,
                            first: t.name.clone(),
                            second: Option::from(t.color.clone()),
                            third: Option::from(t.symbol.clone()),
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
                            (Some(main), Some(other)) => msgs.push(Line::from(vec![
                                Span::styled(
                                    format!("{} {}: ", main, t.name.clone()),
                                    Style::default().add_modifier(Modifier::BOLD).fg(WHITE),
                                ),
                                Span::raw(t.name.clone()),
                            ])),
                        }
                    }

                    let curr_rels = app
                        .rel_list
                        .iter()
                        .filter(|t| t.ppl_id_b == curr.id)
                        .map(|t| t.to_owned())
                        .collect::<Vec<relation::Model>>();
                    for t in &curr_rels {
                        let i2 = match t.date_entered {
                            None => None,
                            Some(d) => Some(d.to_string()),
                        };
                        let i3 = match t.date_ended {
                            None => None,
                            Some(d) => Some(d.to_string()),
                        };
                        editables.push(Editable {
                            id: t.id.clone(),
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

                    let curr_dates = app
                        .sigdate_list
                        .iter()
                        .filter(|t| t.ppl_id == curr.id)
                        .map(|t| t.to_owned())
                        .collect::<Vec<sig_date::Model>>();
                    let date_traits = app
                        .def_trait_list
                        .iter()
                        .filter(|t| t.is_date == true && t.enabled == true)
                        .collect::<Vec<&trait_defaults::Model>>();
                    for t in &curr_dates {
                        editables.push(Editable {
                            id: t.id.clone(),
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

                    let contacts = app
                        .contact_list
                        .iter()
                        .filter(|t| t.ppl_id == curr.id)
                        .map(|t| t.to_owned())
                        .collect::<Vec<contact::Model>>();
                    let contact_traits = app
                        .def_trait_list
                        .iter()
                        .filter(|t| t.is_contact == true && t.enabled == true)
                        .collect::<Vec<&trait_defaults::Model>>();
                    for t in &contacts {
                        editables.push(Editable {
                            id: t.id.clone(),
                            tgt: Editablez::Contact,
                            first: t.r#type.clone(),
                            second: Option::from(t.designator.clone().unwrap_or("".to_string())),
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
                    let block = match app.ppl_editing {
                        true => match app.ppl_field_editing {
                            true => {
                                let ced = editables
                                    .get(app.ppl_detail_state.selected().unwrap())
                                    .unwrap();
                                match { ced.tgt.clone() } {
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

                                let block = Block::new()
                                    .borders(Borders::LEFT)
                                    .title(format!("Editing {}", curr.name.clone()))
                                    .style(Style::default().fg(ORANGE.c500));
                                f.render_widget(paragraph.clone().block(block), layout_edit[0])
                            }
                            false => {
                                let block = Block::new()
                                    .borders(Borders::LEFT)
                                    .title(format!("Editing {}", curr.name.clone()))
                                    .style(Style::default().fg(AMBER.c500));
                                f.render_stateful_widget(
                                    list_contact.block(block),
                                    layout_ppl[1],
                                    &mut app.ppl_detail_state,
                                );
                            }
                        },
                        false => {
                            let block = Block::new()
                                .borders(Borders::ALL)
                                .title(format!("{}", curr.name.clone()));
                            f.render_widget(paragraph.clone().block(block), layout_ppl[1]);
                        }
                    };
                }
            }
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
        _ => {}
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
            true => Line::styled(format!(" ✓ {:?} {}", value.symbol, value.key), WHITE),
            false => Line::styled(format!(" ☐ {:?} {}", value.symbol, value.key), SLATE.c500),
        };

        ListItem::new(line)
    }
}
impl From<&trait_defaults::Model> for ListItem<'_> {
    fn from(value: &trait_defaults::Model) -> Self {
        let line = match value.enabled {
            true => Line::styled(format!(" ✓ {} {}", value.symbol, value.key), WHITE),
            false => Line::styled(format!(" ☐ {} {}", value.symbol, value.key), SLATE.c500),
        };

        ListItem::new(line)
    }
}
