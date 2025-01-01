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
                    .table(Entitys::Table)
                    .col(
                        ColumnDef::new(Entitys::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Entitys::Type).string().not_null())
                    .col(ColumnDef::new(Entitys::PplId).integer().not_null())
                    .col(ColumnDef::new(Entitys::Name).string().not_null())
                    .col(ColumnDef::new(Entitys::DateAcq).date().not_null())
                    .col(ColumnDef::new(Entitys::DateIns).date().not_null())
                    .col(ColumnDef::new(Entitys::DateUp).date().not_null())
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
            .drop_table(Table::drop().table(Entitys::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Entitys {
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
