use sea_orm::sea_query::{ColumnDef, Table};
use sea_orm::{sea_query, DbErr, DeriveIden, ForeignKeyAction};
use sea_orm_migration::{MigrationName, MigrationTrait, SchemaManager};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20241222_000008_create_traitdefaults"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TraitDefaults::Table)
                    .col(
                        ColumnDef::new(TraitDefaults::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TraitDefaults::Key).string().not_null())
                    .col(ColumnDef::new(TraitDefaults::Default).boolean().not_null())
                    .col(ColumnDef::new(TraitDefaults::Enabled).boolean().not_null())
                    .col(ColumnDef::new(TraitDefaults::IsDate).boolean().not_null())
                    .col(ColumnDef::new(TraitDefaults::IsContact).boolean().not_null())
                    .col(ColumnDef::new(TraitDefaults::Color).string().not_null())
                    .col(ColumnDef::new(TraitDefaults::Symbol).string().not_null())
                    .col(ColumnDef::new(TraitDefaults::DateIns).date().not_null())
                    .col(ColumnDef::new(TraitDefaults::DateUp).date().not_null())
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
            .drop_table(Table::drop().table(TraitDefaults::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TraitDefaults {
    Table,
    Id,
    Key,
    Default, // show in new forms / as-missing
    Enabled, // actually-show-in-new-forms-or-not
    IsDate, // map to a sig-date in the as-missings
    IsContact, // map to a contact in the as-missings
    Color,
    Symbol,

    DateIns,
    DateUp,
}
