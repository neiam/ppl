use crate::migrator::m20241219_000001_create_ppl::Ppl;
use crate::migrator::m20241220_000002_create_contact::Contact;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20241220_000003_create_entity"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Entity::Table)
                    .col(
                        ColumnDef::new(Entity::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Entity::Type).string().not_null())
                    .col(ColumnDef::new(Entity::PplId).integer().not_null())
                    .col(ColumnDef::new(Entity::Name).string().not_null())
                    .col(ColumnDef::new(Entity::DateAcq).date().not_null())
                    .col(ColumnDef::new(Entity::DateIns).date().not_null())
                    .col(ColumnDef::new(Entity::DateUp).date().not_null())
                    .to_owned(),
            )
            .await?;
        // let foreign_key = sea_query::ForeignKey::create()
        //     .name("FK_character_font")
        //     .from(Entity::Table, Entity::PplId)
        //     .to(Ppl::Table, Ppl::Id)
        //     .on_delete(ForeignKeyAction::Cascade)
        //     .on_update(ForeignKeyAction::Cascade)
        //     .to_owned();
        // manager.create_foreign_key(foreign_key).await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Entity::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Entity {
    // A house, a pet
    Table,
    Id,
    PplId,
    Name,
    Type,
    DateAcq,

    DateIns,
    DateUp,
}
