use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20241219_000001_create_ppl"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Bakery table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Ppl::Table)
                    .col(
                        ColumnDef::new(Ppl::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Ppl::Name).string().not_null())
                    .col(ColumnDef::new(Ppl::Me).boolean().not_null())
                    .col(ColumnDef::new(Ppl::Nick).string().not_null())
                    .col(ColumnDef::new(Ppl::DateIns).date().not_null())
                    .col(ColumnDef::new(Ppl::DateUp).date().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Ppl::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Ppl {
    Table,
    Id,
    Name,
    Me,   // boolean for self to wish you a happy bday
    Nick, // default-alias
    DateIns,
    DateUp,
}
