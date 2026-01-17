use sea_orm_migration::prelude::*;

mod m20241219_000001_create_ppl;
mod m20241220_000002_create_contact;
mod m20241220_000003_create_entity;
mod m20241220_000004_create_sig_date;
mod m20241220_000005_create_trait;
mod m20241220_000006_create_relation;
mod m20241220_000007_create_tier;
mod m20241222_000008_create_traitsettings;
mod m20241222_000009_create_tiersettings;
mod m20260116_000010_add_sig_date_delta_to_tier;
mod m20260116_000011_add_sig_date_delta_to_tier_defaults;

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
            Box::new(m20241222_000009_create_tiersettings::Migration),
            Box::new(m20260116_000010_add_sig_date_delta_to_tier::Migration),
            Box::new(m20260116_000011_add_sig_date_delta_to_tier_defaults::Migration),
        ]
    }
}
