use crate::migrator::Migrator;
use crate::PplError;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigratorTrait;

pub async fn check_migrations(db: &DatabaseConnection) -> Result<(), PplError> {
    // let be = db.get_database_backend();
    // let schema_manager = SchemaManager::new(db);
    // Migrator::refresh(db).await?;
    Migrator::up(db, None).await?;
    Ok(())
}
