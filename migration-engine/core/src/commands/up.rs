use super::MigrationCommand;
use crate::parse_datamodel;
use migration_connector::ImperativeMigration;
use migration_connector::{DatabaseMigrationMarker, MigrationConnector};
use serde::{Deserialize, Serialize};

pub struct UpCommand<'a> {
    input: &'a UpInput,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpInput {
    pub target_schema: String,
    pub migrations: Vec<ImperativeMigration>,
    pub migration_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpOutput {
    pub warnings: Vec<String>,
    pub unexecutable: Vec<String>,
    pub migration: ImperativeMigration,
}
