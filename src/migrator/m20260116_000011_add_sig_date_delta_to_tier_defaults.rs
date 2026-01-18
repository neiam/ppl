use sea_orm::sea_query::{ColumnDef, Table};
use sea_orm::{DbErr, DeriveIden};
use sea_orm_migration::{MigrationName, MigrationTrait, SchemaManager};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20260116_000011_add_sig_date_delta_to_tier_defaults"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TierDefaults::Table)
                    .add_column(ColumnDef::new(TierDefaults::SigDateDelta).unsigned().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(TierDefaults::Table)
                    .add_column(ColumnDef::new(TierDefaults::SigRemindEnum).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TierDefaults::Table)
                    .drop_column(TierDefaults::SigDateDelta)
                    .drop_column(TierDefaults::SigRemindEnum)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum TierDefaults {
    Table,
    SigDateDelta,
    SigRemindEnum,
}
