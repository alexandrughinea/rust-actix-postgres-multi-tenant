use sqlx::migrate::MigrateError;
use sqlx::PgPool;

pub async fn run_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    // Run base migrations first
    sqlx::migrate!("./migrations").run(pool).await?;

    // In release mode or when we need dynamic paths
    #[cfg(not(debug_assertions))]
    {
        use crate::configurations::get_current_environment;
        use std::path::PathBuf;

        let environment = get_current_environment();
        let migrations_directory = format!("./migrations/{}", environment.as_str());
        let migrations_directory_path = PathBuf::from(&migrations_directory);
        let migrations_directory_exists = Path::new(&migrations_directory).exists();

        // Check if environment-specific migrations exist
        if migrations_directory_exists {
            match sqlx::migrate::Migrator::new(migrations_directory_path).await {
                Ok(migrator) => {
                    if let Err(e) = migrator.run(pool).await {
                        eprintln!(
                            "Warning: Failed to run environment-specific migrations: {}",
                            e
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Could not create migrator for {} environment: {}",
                        environment.as_str(),
                        e
                    );
                }
            }
        } else {
            eprintln!(
                "Notice: No environment-specific migrations found for {}",
                environment.as_str()
            );
        }
    }

    Ok(())
}
