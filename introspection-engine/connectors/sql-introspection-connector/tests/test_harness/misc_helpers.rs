use barrel::Migration;
use pretty_assertions::assert_eq;
use quaint::connector::Queryable;
use std::sync::Arc;

//todo split into barrel and assert???

pub(crate) fn custom_assert(left: &str, right: &str) {
    let parsed_expected = datamodel::parse_datamodel(&right).unwrap();
    let reformatted_expected =
        datamodel::render_datamodel_to_string(&parsed_expected).expect("Datamodel rendering failed");

    assert_eq!(left, reformatted_expected);
}

pub(crate) fn assert_eq_json(a: &str, b: &str) {
    let json_a: serde_json::Value = serde_json::from_str(a).expect("The String a was not valid JSON.");
    let json_b: serde_json::Value = serde_json::from_str(b).expect("The String b was not valid JSON.");

    assert_eq!(json_a, json_b);
}

// barrel

pub struct BarrelMigrationExecutor {
    pub(super) database: Arc<dyn Queryable + Send + Sync>,
    pub(super) sql_variant: barrel::backend::SqlVariant,
    pub(super) schema_name: String,
}

impl BarrelMigrationExecutor {
    pub async fn execute<F>(&self, migration_fn: F)
    where
        F: FnOnce(&mut Migration) -> (),
    {
        self.execute_with_schema(migration_fn, &self.schema_name).await
    }

    pub async fn execute_with_schema<F>(&self, migration_fn: F, schema_name: &str)
    where
        F: FnOnce(&mut Migration) -> (),
    {
        let mut migration = Migration::new().schema(schema_name);
        migration_fn(&mut migration);
        let full_sql = migration.make_from(self.sql_variant);
        run_full_sql(&self.database, &full_sql).await;
    }
}

async fn run_full_sql(database: &Arc<dyn Queryable + Send + Sync>, full_sql: &str) {
    for sql in full_sql.split(";") {
        if sql != "" {
            database.raw_cmd(&sql).await.unwrap();
        }
    }
}
