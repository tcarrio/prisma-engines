use super::MigrationCommand;
use crate::parse_datamodel;
use migration_connector::ImperativeMigration;
use migration_connector::{DatabaseMigrationMarker, Migration, MigrationConnector, MigrationStatus, NewMigration};
use serde::{Deserialize, Serialize};

pub struct UpCommand<'a> {
    input: &'a UpInput,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpInput {
    pub migrations: Vec<ImperativeMigration>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpOutput {}

#[async_trait::async_trait]
impl<'a> MigrationCommand for UpCommand<'a> {
    type Input = UpInput;
    type Output = UpOutput;

    async fn execute<C, D>(
        input: &Self::Input,
        engine: &crate::migration_engine::MigrationEngine<C, D>,
    ) -> super::CommandResult<Self::Output>
    where
        C: MigrationConnector<DatabaseMigration = D>,
        D: DatabaseMigrationMarker + Send + Sync + 'static,
    {
        let connector = engine.connector();
        let persistence = connector.migration_persistence();
        let last_applied_migration = persistence.last().await?;

        let unapplied_migrations: Vec<&ImperativeMigration> =
            if let Some(last_applied_migration) = last_applied_migration {
                let mut migrations = input
                    .migrations
                    .iter()
                    .skip_while(|saved_migration| saved_migration.name != last_applied_migration.name);

                migrations.next(); // skip the last applied migration

                migrations.collect()
            } else {
                input.migrations.iter().collect()
            };

        for migration in unapplied_migrations {
            connector.apply_imperative_migration(migration).await?;

            let mut persisted_migration = Migration::new(NewMigration {
                datamodel_string: migration.prisma_schema.clone(),
                database_migration: serde_json::to_value(&migration.steps).unwrap(),
                name: migration.name.clone(),
                datamodel_steps: Vec::new(),
            });

            persisted_migration.status = MigrationStatus::MigrationSuccess;

            persistence.create(persisted_migration).await?;

            tracing::info!(
                migration_name = migration.name.as_str(),
                steps_count = migration.steps.len(),
                "Successfully applied `{}`",
                migration.name
            );
        }

        Ok(UpOutput {})
    }
}
