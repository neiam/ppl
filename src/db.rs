use sea_orm::DatabaseConnection;
use log::info;
use sea_orm_migration::MigratorTrait;
use crate::migrator::Migrator;
use crate::PplError;

pub async fn check_migrations(db: &DatabaseConnection) -> Result<(), PplError> {

    // let be = db.get_database_backend();
    // let schema_manager = SchemaManager::new(db);
    // Migrator::refresh(db).await?;
    Migrator::up(db, None).await?;
    Ok(())
}