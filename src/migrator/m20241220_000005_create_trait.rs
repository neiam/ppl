use crate::migrator::m20241219_000001_create_ppl::Ppl;
use crate::migrator::m20241220_000004_create_sig_date::SigDate;
use sea_orm::sea_query::{ColumnDef, Table};
use sea_orm::{sea_query, DbErr, DeriveIden, ForeignKeyAction};
use sea_orm_migration::{MigrationName, MigrationTrait, SchemaManager};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20241220_000005_create_trait"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Traits::Table)
                    .col(
                        ColumnDef::new(Traits::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Traits::PplId).integer().not_null())
                    .col(ColumnDef::new(Traits::Key).string().not_null())
                    .col(ColumnDef::new(Traits::Value).string().not_null())
                    .col(ColumnDef::new(Traits::Hidden).boolean().not_null())
                    .col(ColumnDef::new(Traits::DateIns).date().not_null())
                    .col(ColumnDef::new(Traits::DateUp).date().not_null())
                    .to_owned(),
            )
            .await?;

        // let foreign_key = sea_query::ForeignKey::create()
        //     .name("FK_character_font")
        //     .from(Trait::Table, Trait::PplId)
        //     .to(Ppl::Table, Ppl::Id)
        //     .on_delete(ForeignKeyAction::Cascade)
        //     .on_update(ForeignKeyAction::Cascade)
        //     .to_owned();
        // manager.create_foreign_key(foreign_key).await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Traits::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Traits {
    // height / favorite color / other stats
    Table,
    Id,
    PplId,
    Key,
    Value,
    Hidden,

    DateIns,
    DateUp,
}
