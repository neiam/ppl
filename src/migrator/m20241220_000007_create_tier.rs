use sea_orm::sea_query::{ColumnDef, Table};
use sea_orm::{DbErr, DeriveIden};
use sea_orm_migration::{MigrationName, MigrationTrait, SchemaManager};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20241220_000007_create_tier"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tier::Table)
                    .col(
                        ColumnDef::new(Tier::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tier::PplId).integer().not_null())
                    .col(ColumnDef::new(Tier::Name).string().not_null())
                    .col(ColumnDef::new(Tier::Color).string().null())
                    .col(ColumnDef::new(Tier::Symbol).string().null())
                    .col(ColumnDef::new(Tier::DateIns).date().not_null())
                    .col(ColumnDef::new(Tier::DateUp).date().not_null())
                    .to_owned(),
            )
            .await?;

        // let foreign_key = sea_query::ForeignKey::create()
        //     .name("FK_character_font")
        //     .from(Tier::Table, Tier::PplId)
        //     .to(Ppl::Table, Ppl::Id)
        //     .on_delete(ForeignKeyAction::Cascade)
        //     .on_update(ForeignKeyAction::Cascade)
        //     .to_owned();
        // manager.create_foreign_key(foreign_key).await?;

        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tier::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Tier {
    Table,
    Id,
    PplId,
    Name,
    Color,
    Symbol,

    DateIns,
    DateUp,
}
