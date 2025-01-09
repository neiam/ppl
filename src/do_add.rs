use crate::data::{ContactOps, PplOps, RelationOps, SigDateOps, TierOps, TraitOps};
use crate::entities::ppl::Column::Me;
use crate::entities::prelude::Ppl;
use crate::PplError;
use chrono::Local;
use clap::Parser;
use color_eyre::owo_colors::OwoColorize;
use interim::{parse_date_string, Dialect};
use log::warn;
use sea_orm::ColumnTrait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use tracing::info;

// RUST_LOG=warn cargo run -- add Anjali Budday "+1 5088089658" avenkatesh.wpi@gmail.com 1993-08-28 2018-12-20 Wife 2012-01-01 WPI
// RUST_LOG=warn cargo run -- add Ajax Budday "+1 5088675309" me@me.io

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct AddArgs {
    name: String,
    tier: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    bday: Option<String>,
    wedding: Option<String>,
    relation: Option<String>,
    met: Option<String>,
    from: Option<String>,
    address: Option<String>,
}

pub async fn do_add(args: &AddArgs, db: DatabaseConnection) -> Result<(), PplError> {
    info!(args = ?args, "Adding arguments");

    let me = Ppl::find().filter(Me.eq(true)).one(&db).await?.unwrap();

    let pp = PplOps::create(&db, args.name.clone()).await?;

    match &args.tier {
        None => Ok(()),
        Some(tier) => TierOps::create(&db, pp.id, tier.clone()).await,
    }?;
    match &args.phone {
        None => Ok(()),
        Some(phone) => {
            ContactOps::create(
                &db,
                pp.id,
                "phone".to_string(),
                "primary".to_string(),
                phone.to_string(),
            )
            .await
        }
    }?;
    match &args.email {
        None => Ok(()),
        Some(phone) => {
            ContactOps::create(
                &db,
                pp.id,
                "email".to_string(),
                "primary".to_string(),
                phone.to_string(),
            )
            .await
        }
    }?;
    match &args.bday {
        None => Ok(()),
        Some(bday) => {
            let date = parse_date_string(bday, Local::now(), Dialect::Us);
            match date {
                Ok(parsed) => {
                    SigDateOps::create(
                        &db,
                        pp.id,
                        parsed.date_naive(),
                        "birthday".to_string(),
                        true,
                    )
                    .await
                }
                Err(e) => {
                    warn!("failed to parse bday {:?}", e.red());
                    Err(e.into())
                }
            }
        }
    }?;

    match &args.wedding {
        None => Ok(()),
        Some(wed) => {
            let date = parse_date_string(wed, Local::now(), Dialect::Us);
            match date {
                Ok(parsed) => {
                    SigDateOps::create(
                        &db,
                        pp.id,
                        parsed.date_naive(),
                        "wedding".to_string(),
                        false,
                    )
                    .await
                }
                Err(e) => {
                    warn!("failed to parse wedding {:?}", e.red());
                    Err(e.into())
                }
            }
        }
    }?;

    match &args.relation {
        None => Ok(()),
        Some(relation) => {
            RelationOps::create(
                &db,
                me.id,
                pp.id,
                relation.to_owned(),
                false,
                None,
                None,
            )
            .await
        }
    }?;

    match &args.met {
        None => Ok(()),
        Some(met) => {
            let date = parse_date_string(met, Local::now(), Dialect::Us);
            match date {
                Ok(parsed) => {
                    SigDateOps::create(
                        &db,
                        pp.id,
                        parsed.date_naive(),
                        "met".to_string(),
                        false,
                    )
                    .await
                }
                Err(e) => {
                    warn!("failed to parse metdate {:?}", e.red());
                    Err(e.into())
                }
            }
        }
    }?;

    match &args.from {
        None => Ok(()),
        Some(from) => {
            TraitOps::create(
                &db,
                pp.id,
                "from".to_string(),
                from.to_string(),
                false,
            )
            .await
        }
    }?;

    match &args.address {
        None => Ok(()),
        Some(addr) => {
            ContactOps::create(
                &db,
                pp.id,
                "address".to_string(),
                "primary".to_string(),
                addr.to_string(),
            )
            .await
        }
    }?;

    Ok(())
}
