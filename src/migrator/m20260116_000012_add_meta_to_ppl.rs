use sea_orm::sea_query::{ColumnDef, Table};
use sea_orm::{DbErr, DeriveIden};
use sea_orm_migration::{MigrationName, MigrationTrait, SchemaManager};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20260116_000012_add_meta_to_ppl"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Ppl::Table)
                    .add_column(
                        ColumnDef::new(Ppl::Meta)
                            .json_binary()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Ppl::Table)
                    .drop_column(Ppl::Meta)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum Ppl {
    Table,
    Meta,
}
