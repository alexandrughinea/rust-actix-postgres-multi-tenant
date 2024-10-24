use sqlx::migrate::MigrateError;
use sqlx::PgPool;

pub async fn run_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await?;

    // In release mode or when we need dynamic paths
    #[cfg(not(debug_assertions))]
    {
        use crate::configurations::get_current_environment;
        use sqlx::migrate::MigrateError;
        use std::path::PathBuf;

        let environment = get_current_environment();
        let migrations_directory = format!("./migrations/{}", environment.as_str());
        let migrations_directory_path = PathBuf::from(&migrations_directory);

        let migrator = sqlx::migrate::Migrator::new(migrations_directory_path)
            .await
            .expect("Failed to create migrator");

        migrator.run(pool).await?;
    }

    Ok(())
}
