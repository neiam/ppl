use crate::migrator::m20241219_000001_create_ppl::Ppl;
use sea_orm::sea_query::{ColumnDef, Table};
use sea_orm::{sea_query, DbErr, DeriveIden, ForeignKeyAction};
use sea_orm_migration::{MigrationName, MigrationTrait, SchemaManager};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20241220_000004_create_sig_date"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SigDate::Table)
                    .col(
                        ColumnDef::new(SigDate::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SigDate::PplId).integer().not_null())
                    .col(ColumnDef::new(SigDate::Date).date().not_null())
                    .col(ColumnDef::new(SigDate::Event).string().not_null())
                    .col(ColumnDef::new(SigDate::DoRemind).boolean().not_null())
                    .col(ColumnDef::new(SigDate::WithPpl).json().not_null())
                    .col(ColumnDef::new(SigDate::DateIns).date().not_null())
                    .col(ColumnDef::new(SigDate::DateUp).date().not_null())
                    .to_owned(),
            )
            .await?;
        // let foreign_key = sea_query::ForeignKey::create()
        //     .name("FK_character_font")
        //     .from(SigDate::Table, SigDate::PplId)
        //     .to(Ppl::Table, Ppl::Id)
        //     .on_delete(ForeignKeyAction::Cascade)
        //     .on_update(ForeignKeyAction::Cascade)
        //     .to_owned();
        // manager.create_foreign_key(foreign_key).await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SigDate::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum SigDate {
    // meeting date / birthdays / weddings / anniversaries
    Table,
    Id,
    PplId,
    Date,
    Event,
    DoRemind, // show in motd
    WithPpl,

    DateIns,
    DateUp,
}
