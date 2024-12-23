use crate::migrator::m20241219_000001_create_ppl::Ppl;
use crate::migrator::m20241220_000004_create_sig_date::SigDate;
use sea_orm::sea_query::{ColumnDef, Table};
use sea_orm::{sea_query, DbErr, DeriveIden, ForeignKeyAction};
use sea_orm_migration::{MigrationName, MigrationTrait, SchemaManager};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20241220_000006_create_relation"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Relation::Table)
                    .col(
                        ColumnDef::new(Relation::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Relation::PplIdA).integer().not_null())
                    .col(ColumnDef::new(Relation::PplIdB).integer().not_null())
                    .col(ColumnDef::new(Relation::Type).string().not_null())
                    .col(ColumnDef::new(Relation::DateEntered).date().not_null())
                    .col(ColumnDef::new(Relation::DateEnded).date().not_null())
                    .col(ColumnDef::new(Relation::Superseded).boolean().not_null())
                    .col(ColumnDef::new(Relation::DateIns).date().not_null())
                    .col(ColumnDef::new(Relation::DateUp).date().not_null())
                    .to_owned(),
            )
            .await?;

        // let foreign_keya = sea_query::ForeignKey::create()
        //     .name("FK_character_font")
        //     .from(Relation::Table, Relation::PplIdA)
        //     .to(Ppl::Table, Ppl::Id)
        //     .on_delete(ForeignKeyAction::Cascade)
        //     .on_update(ForeignKeyAction::Cascade)
        //     .to_owned();
        // manager.create_foreign_key(foreign_keya).await?;
        // let foreign_keyb = sea_query::ForeignKey::create()
        //     .name("FK_character_font")
        //     .from(Relation::Table, Relation::PplIdB)
        //     .to(Ppl::Table, Ppl::Id)
        //     .on_delete(ForeignKeyAction::Cascade)
        //     .on_update(ForeignKeyAction::Cascade)
        //     .to_owned();
        // manager.create_foreign_key(foreign_keyb).await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Relation::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Relation {
    // Person 2 Person
    Table,
    Id,
    PplIdA,
    PplIdB,
    Type,
    DateEntered,
    DateEnded,
    Superseded, // fiance -> married etc

    DateIns,
    DateUp,
}
