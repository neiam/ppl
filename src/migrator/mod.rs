use sea_orm_migration::prelude::*;

use sea_orm_migration::prelude::*;

mod m20241219_000001_create_ppl;
mod m20241220_000002_create_contact;
mod m20241220_000003_create_entity;
mod m20241220_000004_create_sig_date;
mod m20241220_000005_create_trait;
mod m20241220_000006_create_relation;
mod m20241220_000007_create_tier;
mod m20241222_000008_create_traitsettings;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241219_000001_create_ppl::Migration),
            Box::new(m20241220_000002_create_contact::Migration),
            Box::new(m20241220_000003_create_entity::Migration),
            Box::new(m20241220_000004_create_sig_date::Migration),
            Box::new(m20241220_000005_create_trait::Migration),
            Box::new(m20241220_000006_create_relation::Migration),
            Box::new(m20241220_000007_create_tier::Migration),
            Box::new(m20241222_000008_create_traitsettings::Migration),
        ]
    }
}
