use crate::color::colo;
use crate::do_tui::Editable;
use crate::entities::ppl::Model;
use crate::entities::prelude::Tier;
use crate::entities::prelude::Traits;
use crate::entities::prelude::{Contact, Ppl, Relation, SigDate, TierDefaults, TraitDefaults};
use crate::entities::{
    contact, ppl, relation, sig_date, tier, tier_defaults, trait_defaults, traits,
};
use crate::PplError;
use chrono::{Local, NaiveDate};
use color_eyre::owo_colors::OwoColorize;
use log::warn;
use ratatui::style::palette::tailwind::PURPLE;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, NotSet};

fn bool_from_string(input: Option<String>) -> bool {
    if input.is_none() {
        return false;
    }

    let input = input.unwrap_or("false".to_string());
    // Check if the input is an empty string
    if input.is_empty() {
        return false;
    }

    // Convert the input to lowercase for case-insensitive comparison
    let input = input.to_lowercase();

    // Check if the input matches any of the bool-like values
    match input.as_str() {
        "true" | "yes" | "t" | "y" => true,
        "false" | "no" | "f" | "n" => false,
        _ => false,
    }
}

fn option_naive_date_from_option_string(input: Option<String>) -> Option<NaiveDate> {
    match input {
        Some(s) => chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok(),
        None => None,
    }
}

pub struct ContactOps {}

impl ContactOps {
    pub async fn create(
        db: &DatabaseConnection,
        ppl_id: i32,
        typ: String,
        designator: String,
        value: String,
    ) -> Result<(), PplError> {
        let p = contact::ActiveModel {
            id: Default::default(),
            ppl_id: Set(ppl_id),
            r#type: Set(typ),
            designator: Set(Option::from(designator)),
            value: Set(value),
            date_acq: Set(Some(Local::now().date_naive())),
            date_ins: Set(Local::now().date_naive()),
            date_up: Set(Local::now().date_naive()),
        };

        let pp = p.insert(db).await;
        match pp {
            Ok(item) => {
                let (r, g, b) = colo(PURPLE.c500);
                warn!("contact {:?}", item.id.truecolor(r, g, b));
                Ok(())
            }
            Err(err) => {
                warn!("contact {}", err.red());
                Err(err.into())
            }
        }
    }

    pub async fn list(db: &DatabaseConnection) -> Result<Vec<contact::Model>, DbErr> {
        Contact::find().all(db).await
    }

    pub async fn updatee(db: &DatabaseConnection, editable: Editable) {
        Self::update(
            db,
            editable.id as u8,
            editable.first,
            editable.second,
            editable.third.unwrap_or("".to_string()),
            None,
        )
        .await
        .expect("TODO: panic message");
    }

    pub async fn update(
        db: &DatabaseConnection,
        model_id: u8,
        typ: String,
        designator: Option<String>,
        value: String,
        date_acq: Option<String>,
    ) -> Result<(), PplError> {
        let loaded_model = Contact::find_by_id(model_id).one(db).await?;
        match loaded_model {
            None => Ok(()),
            Some(instance) => {
                let mut am: contact::ActiveModel = instance.into();
                match date_acq {
                    None => {}
                    Some(d) => {
                        am.date_acq = Set(option_naive_date_from_option_string(Option::from(d)))
                    }
                }
                am.designator = Set(designator);
                am.r#type = Set(typ);
                am.value = Set(value);
                am.date_up = Set(Local::now().date_naive());
                am.update(db).await?;
                Ok(())
            }
        }
    }
}

pub struct PplOps {}

impl PplOps {
    pub async fn create_me(db: &DatabaseConnection, name: String) -> Result<ppl::Model, PplError> {
        let p = ppl::ActiveModel {
            id: Default::default(),
            name: Set(name),
            nick: NotSet,
            me: Set(true),
            date_ins: Set(Local::now().date_naive()),
            date_up: Set(Local::now().date_naive()),
            meta: NotSet,
        };
        let pp = p.insert(db).await;
        match pp {
            Ok(item) => {
                warn!("ppl {:?}", item.id);
                Ok(item)
            }
            Err(err) => {
                warn!("ppl {}", err.red());
                Err(err.into())
            }
        }
    }
    pub async fn create(db: &DatabaseConnection, name: String) -> Result<ppl::Model, PplError> {
        let p = ppl::ActiveModel {
            id: Default::default(),
            name: Set(name),
            me: Set(false),
            nick: NotSet,
            date_ins: Set(Local::now().date_naive()),
            date_up: Set(Local::now().date_naive()),
            meta: NotSet,
        };
        let pp = p.insert(db).await;
        match pp {
            Ok(item) => {
                warn!("ppl {:?}", item.id);
                Ok(item)
            }
            Err(err) => {
                warn!("ppl {}", err.red());
                Err(err.into())
            }
        }
    }

    pub async fn list(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
        Ppl::find().all(db).await
    }

    pub async fn update(
        db: &DatabaseConnection,
        id: u8,
        name: Option<String>,
    ) -> Result<(), PplError> {
        let loaded_model = Ppl::find_by_id(id).one(db).await?;
        match loaded_model {
            None => Ok(()),
            Some(instance) => {
                let mut am: ppl::ActiveModel = instance.into();
                match name {
                    None => {}
                    Some(i) => am.name = Set(i),
                }
                am.date_up = Set(Local::now().date_naive());
                am.update(db).await?;
                Ok(())
            }
        }
    }
}

pub struct RelationOps {}

impl RelationOps {
    pub async fn create(
        db: &DatabaseConnection,
        a: i32,
        b: i32,
        typ: String,
        superseded: bool,
        entered: Option<NaiveDate>,
        ended: Option<NaiveDate>,
    ) -> Result<(), PplError> {
        let p = relation::ActiveModel {
            id: Default::default(),
            ppl_id_a: Set(a),
            ppl_id_b: Set(b),
            r#type: Set(typ),
            date_entered: Set(entered),
            date_ended: Set(ended),
            date_ins: Set(Local::now().date_naive()),
            date_up: Set(Local::now().date_naive()),
            superseded: Set(superseded),
        };
        let pp = p.insert(db).await;
        match pp {
            Ok(item) => {
                warn!("relation {:?}", item.id);
                Ok(())
            }
            Err(err) => {
                warn!("relation {}", err.red());
                Err(err.into())
            }
        }
    }

    pub async fn list(db: &DatabaseConnection) -> Result<Vec<relation::Model>, DbErr> {
        Relation::find().all(db).await
    }

    pub async fn updatee(db: &DatabaseConnection, editable: Editable) {
        Self::update(
            db,
            editable.id as u8,
            None,
            None,
            editable.first,
            None,
            option_naive_date_from_option_string(editable.second),
            option_naive_date_from_option_string(editable.third),
        )
        .await
        .expect("huh");
    }

    pub async fn update(
        db: &DatabaseConnection,
        model_id: u8,
        a: Option<i32>,
        b: Option<i32>,
        typ: String,
        superseded: Option<bool>,
        entered: Option<NaiveDate>,
        ended: Option<NaiveDate>,
    ) -> Result<(), PplError> {
        let loaded_model = Relation::find_by_id(model_id).one(db).await?;
        match loaded_model {
            None => Ok(()),
            Some(instance) => {
                let mut am: relation::ActiveModel = instance.into();
                match a {
                    None => {}
                    Some(i) => am.ppl_id_a = Set(i),
                }
                match b {
                    None => {}
                    Some(j) => am.ppl_id_b = Set(j),
                }
                match superseded {
                    None => {}
                    Some(bool_val) => am.superseded = Set(bool_val),
                }
                match entered {
                    None => {}
                    Some(date) => am.date_entered = Set(Option::from(date)),
                }
                match ended {
                    None => {}
                    Some(date) => am.date_ended = Set(Option::from(date)),
                }
                am.r#type = Set(typ);
                am.date_up = Set(Local::now().date_naive());
                am.update(db).await?;
                Ok(())
            }
        }
    }
}

pub struct SigDateOps {}

impl SigDateOps {
    pub async fn create(
        db: &DatabaseConnection,
        ppl_id: i32,
        date: NaiveDate,
        event: String,
        do_remind: bool,
    ) -> Result<(), PplError> {
        let p = sig_date::ActiveModel {
            id: Default::default(),
            ppl_id: Set(ppl_id),
            date: Set(date),
            event: Set(event),
            do_remind: Set(do_remind),
            with_ppl: Default::default(),
            date_ins: Set(Local::now().date_naive()),
            date_up: Set(Local::now().date_naive()),
        };

        let pp = p.insert(db).await;
        match pp {
            Ok(item) => {
                let (r, g, b) = colo(PURPLE.c500);
                warn!("sigDate {:?}", item.id.truecolor(r, g, b));
                Ok(())
            }
            Err(err) => {
                warn!("sigDate {}", err.red());
                Err(err.into())
            }
        }
    }

    pub async fn list(db: &DatabaseConnection) -> Result<Vec<sig_date::Model>, DbErr> {
        SigDate::find().all(db).await
    }

    pub async fn updatee(db: &DatabaseConnection, editable: Editable) {
        Self::update(
            db,
            editable.id as u8,
            editable.first,
            option_naive_date_from_option_string(editable.second),
            bool_from_string(editable.third),
            None,
        )
        .await
        .expect("TODO: panic message");
    }

    pub async fn update(
        db: &DatabaseConnection,
        id: u8,
        event: String,
        date: Option<NaiveDate>,
        do_remind: bool,
        _with_ppl: Option<String>,
    ) -> Result<(), PplError> {
        let loaded_model = SigDate::find_by_id(id).one(db).await?;
        match loaded_model {
            None => Ok(()),
            Some(instance) => {
                let mut am: sig_date::ActiveModel = instance.into();

                am.event = Set(event);
                am.date = Set(date.unwrap_or_default());
                am.do_remind = Set(do_remind);
                am.date_up = Set(Local::now().date_naive());
                am.update(db).await?;
                Ok(())
            }
        }
    }
}

pub struct TierOps {}

impl TierOps {
    pub async fn create(
        db: &DatabaseConnection,
        ppl_id: i32,
        tier: String,
        sig_date_delta: Option<u32>
    ) -> Result<(), PplError> {
        let t = tier::ActiveModel {
            id: Default::default(),
            ppl_id: Set(ppl_id),
            name: Set(tier.to_string()),
            color: Default::default(),
            symbol: Default::default(),
            date_ins: Set(Local::now().date_naive()),
            date_up: Set(Local::now().date_naive()),
            sig_date_delta: Set(sig_date_delta),
            sig_remind_enum: NotSet,
        };
        let tt = t.insert(db).await;
        match tt {
            Ok(item) => {
                warn!("tier {:?}", item.id);
                Ok(())
            }
            Err(err) => {
                warn!("tier {}", err.red());
                Err(err.into())
            }
        }
    }

    pub async fn list(db: &DatabaseConnection) -> Result<Vec<tier::Model>, DbErr> {
        Tier::find().all(db).await
    }

    pub async fn updatee(db: &DatabaseConnection, editable: Editable) {
        Self::update(
            db,
            editable.id as u8,
            editable.first,
            editable.second,
            editable.third,
        )
        .await
        .expect("TODO: panic message");
    }

    pub async fn update(
        db: &DatabaseConnection,
        model_id: u8,
        name: String,
        color: Option<String>,
        symbol: Option<String>,
    ) -> Result<(), PplError> {
        let loaded_model = Tier::find_by_id(model_id).one(db).await?;
        match loaded_model {
            None => Ok(()),
            Some(instance) => {
                let mut am: tier::ActiveModel = instance.into();

                am.name = Set(name);
                am.color = Set(color);
                am.symbol = Set(symbol);
                am.date_up = Set(Local::now().date_naive());
                am.update(db).await?;
                Ok(())
            }
        }
    }
}

pub struct TraitOps {}

impl TraitOps {
    pub async fn create(
        db: &DatabaseConnection,
        ppl_id: i32,
        key: String,
        value: String,
        hidden: bool,
    ) -> Result<(), PplError> {
        let t = traits::ActiveModel {
            id: Default::default(),
            ppl_id: Set(ppl_id),
            key: Set(key.to_string()),
            value: Set(value),
            hidden: Set(hidden),
            date_ins: Set(Local::now().date_naive()),
            date_up: Set(Local::now().date_naive()),
        };
        let tt = t.insert(db).await;
        match tt {
            Ok(item) => {
                warn!("trait {:?}", item.id);
                Ok(())
            }
            Err(err) => {
                warn!("trait {}", err.red());
                Err(err.into())
            }
        }
    }
    pub async fn list(db: &DatabaseConnection) -> Result<Vec<traits::Model>, DbErr> {
        Traits::find().all(db).await
    }

    pub async fn updatee(db: &DatabaseConnection, editable: Editable) {
        Self::update(
            db,
            editable.id as u8,
            Option::from(editable.first),
            editable.second,
            bool_from_string(editable.third),
        )
        .await
        .expect("TODO: panic message");
    }

    pub async fn update(
        db: &DatabaseConnection,
        model_id: u8,
        key: Option<String>,
        value: Option<String>,
        hidden: bool,
    ) -> Result<(), PplError> {
        let loaded_model = Traits::find_by_id(model_id).one(db).await?;
        match loaded_model {
            None => Ok(()),
            Some(instance) => {
                let mut am: traits::ActiveModel = instance.into();
                match key {
                    None => {}
                    Some(i) => am.key = Set(i),
                }
                match value {
                    None => {}
                    Some(j) => am.value = Set(j),
                }
                am.hidden = Set(hidden);
                am.date_up = Set(Local::now().date_naive());
                am.update(db).await?;
                Ok(())
            }
        }
    }
}

pub struct TierDefaultOps {}

impl TierDefaultOps {
    pub async fn create(
        db: &DatabaseConnection,
        key: String,
        default: bool,
        enabled: bool,
        color: String,
        symbol: String,
        sig_date_delta: Option<u32>
    ) -> Result<(), PplError> {
        let t = tier_defaults::ActiveModel {
            id: Default::default(),
            key: Set(key),
            default: Set(default),
            enabled: Set(enabled),
            color: Set(Option::from(color)),
            symbol: Set(Option::from(symbol)),
            date_ins: Set(Local::now().date_naive()),
            date_up: Set(Local::now().date_naive()),
            sig_date_delta: Set(sig_date_delta),
            sig_remind_enum: NotSet,
        };
        let tt = t.insert(db).await;
        match tt {
            Ok(item) => {
                warn!("tierDefault {:?}", item.id);
                Ok(())
            }
            Err(err) => {
                warn!("tierDefault {}", err.red());
                Err(err.into())
            }
        }
    }

    pub async fn list(db: &DatabaseConnection) -> Result<Vec<tier_defaults::Model>, DbErr> {
        TierDefaults::find().all(db).await
    }
}

pub struct TraitDefaultOps {}

impl TraitDefaultOps {
    pub async fn create(
        db: &DatabaseConnection,
        key: String,
        color: String,
        symbol: String,
        default: bool,
        enabled: bool,
        is_date: bool,
        is_contact: bool,
    ) -> Result<(), PplError> {
        let t = trait_defaults::ActiveModel {
            id: Default::default(),
            key: Set(key.to_string()),
            default: Set(default),
            enabled: Set(enabled),
            is_date: Set(is_date),
            is_contact: Set(is_contact),
            color: Set(color),
            symbol: Set(symbol),
            date_ins: Set(Local::now().date_naive()),
            date_up: Set(Local::now().date_naive()),
        };
        let tt = t.insert(db).await;
        match tt {
            Ok(item) => {
                warn!("traitDefault {:?}", item.id);
                Ok(())
            }
            Err(err) => {
                warn!("traitDefault {}", err.red());
                Err(err.into())
            }
        }
    }
    pub async fn list(db: &DatabaseConnection) -> Result<Vec<trait_defaults::Model>, DbErr> {
        TraitDefaults::find().all(db).await
    }
}
