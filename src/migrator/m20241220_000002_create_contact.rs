use crate::migrator::m20241219_000001_create_ppl::Ppl;
use crate::migrator::m20241220_000003_create_entity::Entitys;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20241220_000002_create_contact"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Contact::Table)
                    .col(
                        ColumnDef::new(Contact::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Contact::PplId).integer().not_null())
                    .col(ColumnDef::new(Contact::Type).string().not_null())
                    .col(ColumnDef::new(Contact::Designator).string().null())
                    .col(ColumnDef::new(Contact::Value).string().not_null())
                    .col(ColumnDef::new(Contact::DateAcq).date().not_null())
                    .col(ColumnDef::new(Contact::DateIns).date().not_null())
                    .col(ColumnDef::new(Contact::DateUp).date().not_null())
                    .to_owned(),
            )
            .await?;

        // let foreign_key = sea_query::ForeignKey::create()
        //     .name("FK_character_font")
        //     .from(Contact::Table, Contact::PplId)
        //     .to(Ppl::Table, Ppl::Id)
        //     .on_delete(ForeignKeyAction::Cascade)
        //     .on_update(ForeignKeyAction::Cascade)
        //     .to_owned();
        // manager.create_foreign_key(foreign_key).await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Contact::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Contact {
    // phones / emails / mailing
    Table,
    Id,
    PplId,
    Type,
    Designator, // todo remember what this was for
    Value,
    DateAcq,

    DateIns,
    DateUp,
}
