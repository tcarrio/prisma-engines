mod common;
mod sqlite;
mod test_api;

use barrel::{types, Migration};
use common::*;
use pretty_assertions::assert_eq;
use sql_schema_describer::*;
use sqlite::*;
use test_api::{sqlite_test_api, TestApi, TestResult};
use test_macros::test_each_connector;

#[async_std::test]
async fn sqlite_column_types_must_work() {
    let mut migration = Migration::new().schema(SCHEMA);
    migration.create_table("User", move |t| {
        t.inject_custom("int_col int not null");
        t.add_column("int4_col", types::integer());
        t.add_column("text_col", types::text());
        t.add_column("real_col", types::float());
        t.add_column("primary_col", types::primary());
        t.inject_custom("decimal_col decimal (5, 3) not null");
    });

    let full_sql = migration.make::<barrel::backend::Sqlite>();
    let inspector = get_sqlite_describer(&full_sql, "sqlite_column_types_must_work").await;
    let result = inspector.describe(SCHEMA).await.expect("describing");
    let table = result.get_table("User").expect("couldn't get User table");
    let expected_columns = vec![
        Column {
            name: "int_col".to_string(),
            tpe: ColumnType {
                data_type: "int".to_string(),
                full_data_type: "int".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Int,
                arity: ColumnArity::Required,
            },
            default: None,
            auto_increment: false,
        },
        Column {
            name: "int4_col".to_string(),
            tpe: ColumnType {
                data_type: "INTEGER".to_string(),
                full_data_type: "INTEGER".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Int,
                arity: ColumnArity::Required,
            },
            default: None,
            auto_increment: false,
        },
        Column {
            name: "text_col".to_string(),
            tpe: ColumnType {
                data_type: "TEXT".to_string(),
                full_data_type: "TEXT".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::String,
                arity: ColumnArity::Required,
            },
            default: None,
            auto_increment: false,
        },
        Column {
            name: "real_col".to_string(),
            tpe: ColumnType {
                data_type: "REAL".to_string(),
                full_data_type: "REAL".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Float,
                arity: ColumnArity::Required,
            },
            default: None,
            auto_increment: false,
        },
        Column {
            name: "primary_col".to_string(),
            tpe: ColumnType {
                data_type: "INTEGER".to_string(),
                full_data_type: "INTEGER".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Int,
                arity: ColumnArity::Required,
            },
            default: None,
            auto_increment: true,
        },
        Column {
            name: "decimal_col".to_string(),
            tpe: ColumnType {
                data_type: "decimal (5, 3)".to_string(),
                full_data_type: "decimal (5, 3)".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Float,
                arity: ColumnArity::Required,
            },
            default: None,
            auto_increment: false,
        },
    ];

    assert_eq!(
        table,
        &Table {
            name: "User".to_string(),
            columns: expected_columns,
            indices: vec![],
            primary_key: Some(PrimaryKey {
                columns: vec!["primary_col".to_string()],
                sequence: None,
                constraint_name: None,
            }),
            foreign_keys: vec![],
        }
    );
}

#[async_std::test]
async fn sqlite_foreign_key_on_delete_must_be_handled() {
    let sql = format!(
        "CREATE TABLE \"{0}\".City (id INTEGER NOT NULL PRIMARY KEY);
         CREATE TABLE \"{0}\".User (
            id INTEGER NOT NULL PRIMARY KEY,
            city INTEGER REFERENCES City(id) ON DELETE NO ACTION,
            city_cascade INTEGER REFERENCES City(id) ON DELETE CASCADE,
            city_restrict INTEGER REFERENCES City (id) ON DELETE RESTRICT,
            city_set_default INTEGER REFERENCES City(id) ON DELETE SET DEFAULT,
            city_set_null INTEGER REFERENCES City(id) ON DELETE SET NULL
        )",
        SCHEMA
    );
    let inspector = get_sqlite_describer(&sql, "sqlite_foreign_key_on_delete_must_be_handled").await;

    let schema = inspector.describe(SCHEMA).await.expect("describing");
    let mut table = schema.get_table("User").expect("get User table").to_owned();
    table.foreign_keys.sort_unstable_by_key(|fk| fk.columns.clone());

    assert_eq!(
        table,
        Table {
            name: "User".to_string(),
            columns: vec![
                Column {
                    name: "id".to_string(),
                    tpe: ColumnType {
                        data_type: "INTEGER".to_string(),
                        full_data_type: "INTEGER".to_string(),
                        character_maximum_length: None,
                        family: ColumnTypeFamily::Int,
                        arity: ColumnArity::Required,
                    },
                    default: None,
                    auto_increment: true,
                },
                Column {
                    name: "city".to_string(),
                    tpe: ColumnType {
                        data_type: "INTEGER".to_string(),
                        full_data_type: "INTEGER".to_string(),
                        character_maximum_length: None,
                        family: ColumnTypeFamily::Int,
                        arity: ColumnArity::Nullable,
                    },
                    default: None,
                    auto_increment: false,
                },
                Column {
                    name: "city_cascade".to_string(),
                    tpe: ColumnType {
                        data_type: "INTEGER".to_string(),
                        full_data_type: "INTEGER".to_string(),
                        character_maximum_length: None,
                        family: ColumnTypeFamily::Int,
                        arity: ColumnArity::Nullable,
                    },
                    default: None,
                    auto_increment: false,
                },
                Column {
                    name: "city_restrict".to_string(),
                    tpe: ColumnType {
                        data_type: "INTEGER".to_string(),
                        full_data_type: "INTEGER".to_string(),
                        character_maximum_length: None,
                        family: ColumnTypeFamily::Int,
                        arity: ColumnArity::Nullable,
                    },
                    default: None,
                    auto_increment: false,
                },
                Column {
                    name: "city_set_default".to_string(),
                    tpe: ColumnType {
                        data_type: "INTEGER".to_string(),
                        full_data_type: "INTEGER".to_string(),
                        character_maximum_length: None,
                        family: ColumnTypeFamily::Int,
                        arity: ColumnArity::Nullable,
                    },
                    default: None,
                    auto_increment: false,
                },
                Column {
                    name: "city_set_null".to_string(),
                    tpe: ColumnType {
                        data_type: "INTEGER".to_string(),
                        full_data_type: "INTEGER".to_string(),
                        character_maximum_length: None,

                        family: ColumnTypeFamily::Int,
                        arity: ColumnArity::Nullable,
                    },
                    default: None,
                    auto_increment: false,
                },
            ],
            indices: vec![],
            primary_key: Some(PrimaryKey {
                columns: vec!["id".to_string()],
                sequence: None,
                constraint_name: None,
            }),
            foreign_keys: vec![
                ForeignKey {
                    constraint_name: None,
                    columns: vec!["city".to_string()],
                    referenced_columns: vec!["id".to_string()],
                    referenced_table: "City".to_string(),
                    on_delete_action: ForeignKeyAction::NoAction,
                },
                ForeignKey {
                    constraint_name: None,
                    columns: vec!["city_cascade".to_string()],
                    referenced_columns: vec!["id".to_string()],
                    referenced_table: "City".to_string(),
                    on_delete_action: ForeignKeyAction::Cascade,
                },
                ForeignKey {
                    constraint_name: None,
                    columns: vec!["city_restrict".to_string()],
                    referenced_columns: vec!["id".to_string()],
                    referenced_table: "City".to_string(),
                    on_delete_action: ForeignKeyAction::Restrict,
                },
                ForeignKey {
                    constraint_name: None,
                    columns: vec!["city_set_default".to_string()],
                    referenced_columns: vec!["id".to_string()],
                    referenced_table: "City".to_string(),
                    on_delete_action: ForeignKeyAction::SetDefault,
                },
                ForeignKey {
                    constraint_name: None,
                    columns: vec!["city_set_null".to_string()],
                    referenced_columns: vec!["id".to_string()],
                    referenced_table: "City".to_string(),
                    on_delete_action: ForeignKeyAction::SetNull,
                },
            ],
        }
    );
}

#[async_std::test]
async fn sqlite_text_primary_keys_must_be_inferred_on_table_and_not_as_separate_indexes() {
    let mut migration = Migration::new().schema(SCHEMA);
    migration.create_table("User", move |t| {
        t.add_column("int4_col", types::integer());
        t.add_column("text_col", types::text());
        t.add_column("real_col", types::float());
        t.add_column("primary_col", types::text());

        // Simulate how we create primary keys in the migrations engine.
        t.inject_custom("PRIMARY KEY (\"primary_col\")");
    });
    let full_sql = migration.make::<barrel::backend::Sqlite>();

    let inspector = get_sqlite_describer(
        &full_sql,
        "sqlite_text_primary_keys_must_be_inferred_on_table_and_not_as_separate_indexes",
    )
    .await;
    let result = inspector.describe(SCHEMA).await.expect("describing");

    let table = result.get_table("User").expect("couldn't get User table");

    assert!(table.indices.is_empty());

    assert_eq!(
        table.primary_key.as_ref().unwrap(),
        &PrimaryKey {
            columns: vec!["primary_col".to_owned()],
            sequence: None,
            constraint_name: None,
        }
    );
}

#[test_each_connector(tags("sqlite"))]
async fn escaped_quotes_in_string_defaults_must_be_unescaped(api: &TestApi) -> TestResult {
    let create_table = format!(
        r#"
            CREATE TABLE "{0}"."string_defaults_test" (
                id INTEGER PRIMARY KEY,
                regular VARCHAR NOT NULL DEFAULT 'meow, says the cat',
                escaped VARCHAR NOT NULL DEFAULT '"That''s a lot of fish!"
- Godzilla, 1998'
            );
        "#,
        api.schema_name()
    );

    api.database().query_raw(&create_table, vec![]).await?;

    let schema = api.describe().await?;

    let table = schema.table_bang("string_defaults_test");

    let regular_column_default = table
        .column_bang("regular")
        .default
        .as_ref()
        .unwrap()
        .as_value()
        .unwrap()
        .clone()
        .into_string()
        .unwrap();

    assert_eq!(regular_column_default, "meow, says the cat");

    let escaped_column_default = table
        .column_bang("escaped")
        .default
        .as_ref()
        .unwrap()
        .as_value()
        .unwrap()
        .clone()
        .into_string()
        .unwrap();

    assert_eq!(escaped_column_default, "\"That's a lot of fish!\"\n- Godzilla, 1998");

    Ok(())
}

#[test_each_connector(tags("sqlite"))]
async fn escaped_backslashes_in_string_literals_must_be_unescaped(api: &TestApi) -> TestResult {
    let create_table = format!(
        r#"
            CREATE TABLE "{0}"."test" (
                model_name_space VARCHAR(255) NOT NULL DEFAULT 'xyz\Datasource\Model'
            );
        "#,
        api.schema_name()
    );

    api.database().query_raw(&create_table, vec![]).await?;

    let schema = api.describe().await?;

    let table = schema.table_bang("test");

    let default = table
        .column_bang("model_name_space")
        .default
        .as_ref()
        .unwrap()
        .as_value()
        .unwrap()
        .clone()
        .into_string()
        .unwrap();

    assert_eq!(default, "xyz\\Datasource\\Model");

    Ok(())
}
