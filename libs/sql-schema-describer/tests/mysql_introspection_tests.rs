mod mysql;
mod test_api;

use crate::mysql::*;
use barrel::{types, Migration};
use pretty_assertions::assert_eq;
use sql_schema_describer::*;
use test_api::*;
use test_macros::*;

#[async_std::test]
async fn all_mysql_column_types_must_work() {
    let db_name = "all_mysql_column_types_must_work";

    let mut migration = Migration::new().schema(db_name);
    migration.create_table("User", move |t| {
        t.add_column("primary_col", types::primary());
        t.add_column("int_col", types::custom("int"));
        t.add_column("smallint_col", types::custom("smallint"));
        t.add_column("tinyint4_col", types::custom("tinyint(4)"));
        t.add_column("tinyint1_col", types::custom("tinyint(1)"));
        t.add_column("mediumint_col", types::custom("mediumint"));
        t.add_column("bigint_col", types::custom("bigint"));
        t.add_column("decimal_col", types::custom("decimal"));
        t.add_column("numeric_col", types::custom("numeric"));
        t.add_column("float_col", types::custom("float"));
        t.add_column("double_col", types::custom("double"));
        t.add_column("date_col", types::custom("date"));
        t.add_column("time_col", types::custom("time"));
        t.add_column("datetime_col", types::custom("datetime"));
        t.add_column("timestamp_col", types::custom("timestamp"));
        t.add_column("year_col", types::custom("year"));
        t.add_column("char_col", types::custom("char"));
        t.add_column("varchar_col", types::custom("varchar(255)"));
        t.add_column("text_col", types::custom("text"));
        t.add_column("tinytext_col", types::custom("tinytext"));
        t.add_column("mediumtext_col", types::custom("mediumtext"));
        t.add_column("longtext_col", types::custom("longtext"));
        t.add_column("enum_col", types::custom("enum('a', 'b')"));
        t.add_column("set_col", types::custom("set('a', 'b')"));
        t.add_column("binary_col", types::custom("binary"));
        t.add_column("varbinary_col", types::custom("varbinary(255)"));
        t.add_column("blob_col", types::custom("blob"));
        t.add_column("tinyblob_col", types::custom("tinyblob"));
        t.add_column("mediumblob_col", types::custom("mediumblob"));
        t.add_column("longblob_col", types::custom("longblob"));
        t.add_column("geometry_col", types::custom("geometry"));
        t.add_column("point_col", types::custom("point"));
        t.add_column("linestring_col", types::custom("linestring"));
        t.add_column("polygon_col", types::custom("polygon"));
        t.add_column("multipoint_col", types::custom("multipoint"));
        t.add_column("multilinestring_col", types::custom("multilinestring"));
        t.add_column("multipolygon_col", types::custom("multipolygon"));
        t.add_column("geometrycollection_col", types::custom("geometrycollection"));
        t.add_column("json_col", types::custom("json"));
    });

    let full_sql = migration.make::<barrel::backend::MySql>();
    let inspector = get_mysql_describer_for_schema(&full_sql, db_name).await;
    let result = inspector.describe(db_name).await.expect("describing");
    let mut table = result.get_table("User").expect("couldn't get User table").to_owned();
    // Ensure columns are sorted as expected when comparing
    table.columns.sort_unstable_by_key(|c| c.name.to_owned());
    let mut expected_columns = vec![
        Column {
            name: "primary_col".to_string(),
            tpe: ColumnType {
                data_type: "int".to_string(),
                full_data_type: "int(11)".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Int,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: true,
        },
        Column {
            name: "int_col".to_string(),
            tpe: ColumnType {
                data_type: "int".to_string(),
                full_data_type: "int(11)".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Int,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "smallint_col".to_string(),
            tpe: ColumnType {
                data_type: "smallint".to_string(),
                full_data_type: "smallint(6)".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Int,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "tinyint4_col".to_string(),
            tpe: ColumnType {
                data_type: "tinyint".to_string(),
                full_data_type: "tinyint(4)".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Int,
                arity: ColumnArity::Required,
            },
            default: None,
            auto_increment: false,
        },
        Column {
            name: "tinyint1_col".to_string(),
            tpe: ColumnType {
                data_type: "tinyint".to_string(),
                full_data_type: "tinyint(1)".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Boolean,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "mediumint_col".to_string(),
            tpe: ColumnType {
                data_type: "mediumint".to_string(),
                full_data_type: "mediumint(9)".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Int,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "bigint_col".to_string(),
            tpe: ColumnType {
                data_type: "bigint".to_string(),
                full_data_type: "bigint(20)".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Int,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "decimal_col".to_string(),
            tpe: ColumnType {
                data_type: "decimal".to_string(),
                full_data_type: "decimal(10,0)".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Float,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "numeric_col".to_string(),
            tpe: ColumnType {
                data_type: "decimal".to_string(),
                full_data_type: "decimal(10,0)".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Float,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "float_col".to_string(),
            tpe: ColumnType {
                data_type: "float".to_string(),
                full_data_type: "float".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Float,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "double_col".to_string(),
            tpe: ColumnType {
                data_type: "double".to_string(),
                full_data_type: "double".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Float,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "date_col".to_string(),
            tpe: ColumnType {
                data_type: "date".to_string(),
                full_data_type: "date".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::DateTime,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "time_col".to_string(),
            tpe: ColumnType {
                data_type: "time".to_string(),
                full_data_type: "time".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::DateTime,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "datetime_col".to_string(),
            tpe: ColumnType {
                data_type: "datetime".to_string(),
                full_data_type: "datetime".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::DateTime,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "timestamp_col".to_string(),
            tpe: ColumnType {
                data_type: "timestamp".to_string(),
                full_data_type: "timestamp".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::DateTime,
                arity: ColumnArity::Required,
            },

            default: Some(DefaultValue::NOW),
            auto_increment: false,
        },
        Column {
            name: "year_col".to_string(),
            tpe: ColumnType {
                data_type: "year".to_string(),
                full_data_type: "year(4)".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Int,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "char_col".to_string(),
            tpe: ColumnType {
                data_type: "char".to_string(),
                full_data_type: "char(1)".to_string(),
                character_maximum_length: Some(1),
                family: ColumnTypeFamily::String,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "varchar_col".to_string(),
            tpe: ColumnType {
                data_type: "varchar".to_string(),
                full_data_type: "varchar(255)".to_string(),
                character_maximum_length: Some(255),
                family: ColumnTypeFamily::String,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "text_col".to_string(),
            tpe: ColumnType {
                data_type: "text".to_string(),
                full_data_type: "text".to_string(),
                character_maximum_length: Some(65535),
                family: ColumnTypeFamily::String,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "tinytext_col".to_string(),
            tpe: ColumnType {
                data_type: "tinytext".to_string(),
                full_data_type: "tinytext".to_string(),
                character_maximum_length: Some(255),
                family: ColumnTypeFamily::String,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "mediumtext_col".to_string(),
            tpe: ColumnType {
                data_type: "mediumtext".to_string(),
                full_data_type: "mediumtext".to_string(),
                character_maximum_length: Some(16777215),
                family: ColumnTypeFamily::String,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "longtext_col".to_string(),
            tpe: ColumnType {
                data_type: "longtext".to_string(),
                full_data_type: "longtext".to_string(),
                character_maximum_length: Some(4294967295),
                family: ColumnTypeFamily::String,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "enum_col".to_string(),
            tpe: ColumnType {
                data_type: "enum".to_string(),
                full_data_type: "enum(\'a\',\'b\')".to_string(),
                character_maximum_length: Some(1),
                family: ColumnTypeFamily::Enum("User_enum_col".into()),
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "set_col".to_string(),
            tpe: ColumnType {
                data_type: "set".to_string(),
                full_data_type: "set(\'a\',\'b\')".to_string(),
                character_maximum_length: Some(3),
                family: ColumnTypeFamily::String,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "binary_col".to_string(),
            tpe: ColumnType {
                data_type: "binary".to_string(),
                full_data_type: "binary(1)".to_string(),
                character_maximum_length: Some(1),
                family: ColumnTypeFamily::Binary,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "varbinary_col".to_string(),
            tpe: ColumnType {
                data_type: "varbinary".to_string(),
                full_data_type: "varbinary(255)".to_string(),
                character_maximum_length: Some(255),
                family: ColumnTypeFamily::Binary,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "blob_col".to_string(),
            tpe: ColumnType {
                data_type: "blob".to_string(),
                full_data_type: "blob".to_string(),
                character_maximum_length: Some(65535),
                family: ColumnTypeFamily::Binary,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "tinyblob_col".to_string(),
            tpe: ColumnType {
                data_type: "tinyblob".to_string(),
                full_data_type: "tinyblob".to_string(),
                character_maximum_length: Some(255),

                family: ColumnTypeFamily::Binary,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "mediumblob_col".to_string(),
            tpe: ColumnType {
                data_type: "mediumblob".to_string(),
                full_data_type: "mediumblob".to_string(),
                character_maximum_length: Some(16777215),
                family: ColumnTypeFamily::Binary,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "longblob_col".to_string(),
            tpe: ColumnType {
                data_type: "longblob".to_string(),
                full_data_type: "longblob".to_string(),
                character_maximum_length: Some(4294967295),
                family: ColumnTypeFamily::Binary,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "geometry_col".to_string(),
            tpe: ColumnType {
                data_type: "geometry".to_string(),
                full_data_type: "geometry".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Geometric,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "point_col".to_string(),
            tpe: ColumnType {
                data_type: "point".to_string(),
                full_data_type: "point".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Geometric,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "linestring_col".to_string(),
            tpe: ColumnType {
                data_type: "linestring".to_string(),
                full_data_type: "linestring".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Geometric,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "polygon_col".to_string(),
            tpe: ColumnType {
                data_type: "polygon".to_string(),
                full_data_type: "polygon".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Geometric,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "multipoint_col".to_string(),
            tpe: ColumnType {
                data_type: "multipoint".to_string(),
                full_data_type: "multipoint".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Geometric,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "multilinestring_col".to_string(),
            tpe: ColumnType {
                data_type: "multilinestring".to_string(),
                full_data_type: "multilinestring".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Geometric,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "multipolygon_col".to_string(),
            tpe: ColumnType {
                data_type: "multipolygon".to_string(),
                full_data_type: "multipolygon".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Geometric,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "geometrycollection_col".to_string(),
            tpe: ColumnType {
                data_type: "geometrycollection".to_string(),
                full_data_type: "geometrycollection".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Geometric,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
        Column {
            name: "json_col".to_string(),
            tpe: ColumnType {
                data_type: "json".to_string(),
                full_data_type: "json".to_string(),
                character_maximum_length: None,
                family: ColumnTypeFamily::Json,
                arity: ColumnArity::Required,
            },

            default: None,
            auto_increment: false,
        },
    ];
    expected_columns.sort_unstable_by_key(|c| c.name.to_owned());

    assert_eq!(
        table,
        Table {
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
async fn mysql_foreign_key_on_delete_must_be_handled() {
    let db_name = "mysql_foreign_key_on_delete_must_be_handled";

    // NB: We don't test the SET DEFAULT variety since it isn't supported on InnoDB and will
    // just cause an error
    let sql = format!(
        "CREATE TABLE `{0}`.City (id INTEGER NOT NULL AUTO_INCREMENT PRIMARY KEY);
         CREATE TABLE `{0}`.User (
            id INTEGER NOT NULL AUTO_INCREMENT PRIMARY KEY,
            city INTEGER, FOREIGN KEY(city) REFERENCES City (id) ON DELETE NO ACTION,
            city_cascade INTEGER, FOREIGN KEY(city_cascade) REFERENCES City (id) ON DELETE CASCADE,
            city_restrict INTEGER, FOREIGN KEY(city_restrict) REFERENCES City (id) ON DELETE RESTRICT,
            city_set_null INTEGER, FOREIGN KEY(city_set_null) REFERENCES City (id) ON DELETE SET NULL
        )",
        db_name
    );
    let inspector = get_mysql_describer_for_schema(&sql, db_name).await;

    let schema = inspector.describe(db_name).await.expect("describing");
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
                        data_type: "int".to_string(),
                        full_data_type: "int(11)".to_string(),
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
                        data_type: "int".to_string(),
                        full_data_type: "int(11)".to_string(),
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
                        data_type: "int".to_string(),
                        full_data_type: "int(11)".to_string(),
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
                        data_type: "int".to_string(),
                        full_data_type: "int(11)".to_string(),
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
                        data_type: "int".to_string(),
                        full_data_type: "int(11)".to_string(),
                        character_maximum_length: None,
                        family: ColumnTypeFamily::Int,
                        arity: ColumnArity::Nullable,
                    },
                    default: None,
                    auto_increment: false,
                },
            ],
            indices: vec![
                Index {
                    name: "city".to_owned(),
                    columns: vec!["city".to_owned(),],
                    tpe: IndexType::Normal
                },
                Index {
                    name: "city_cascade".to_owned(),
                    columns: vec!["city_cascade".to_owned(),],
                    tpe: IndexType::Normal
                },
                Index {
                    name: "city_restrict".to_owned(),
                    columns: vec!["city_restrict".to_owned(),],
                    tpe: IndexType::Normal
                },
                Index {
                    name: "city_set_null".to_owned(),
                    columns: vec!["city_set_null".to_owned(),],
                    tpe: IndexType::Normal
                }
            ],
            primary_key: Some(PrimaryKey {
                columns: vec!["id".to_string()],
                sequence: None,
                constraint_name: None,
            }),
            foreign_keys: vec![
                ForeignKey {
                    constraint_name: Some("User_ibfk_1".to_owned()),
                    columns: vec!["city".to_string()],
                    referenced_columns: vec!["id".to_string()],
                    referenced_table: "City".to_string(),
                    on_delete_action: ForeignKeyAction::NoAction,
                },
                ForeignKey {
                    constraint_name: Some("User_ibfk_2".to_owned()),
                    columns: vec!["city_cascade".to_string()],
                    referenced_columns: vec!["id".to_string()],
                    referenced_table: "City".to_string(),
                    on_delete_action: ForeignKeyAction::Cascade,
                },
                ForeignKey {
                    constraint_name: Some("User_ibfk_3".to_owned()),
                    columns: vec!["city_restrict".to_string()],
                    referenced_columns: vec!["id".to_string()],
                    referenced_table: "City".to_string(),
                    on_delete_action: ForeignKeyAction::Restrict,
                },
                ForeignKey {
                    constraint_name: Some("User_ibfk_4".to_owned()),
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
async fn mysql_multi_field_indexes_must_be_inferred() {
    let db_name = "mysql_multi_field_indexes_must_be_inferred";

    let mut migration = Migration::new().schema(db_name);
    migration.create_table("Employee", move |t| {
        t.add_column("id", types::primary());
        t.add_column("age", types::integer());
        t.add_column("name", types::varchar(200));
        t.add_index("age_and_name_index", types::index(vec!["name", "age"]).unique(true));
    });

    let full_sql = migration.make::<barrel::backend::MySql>();
    let inspector = get_mysql_describer_for_schema(&full_sql, db_name).await;
    let result = inspector.describe(db_name).await.expect("describing");
    let table = result.get_table("Employee").expect("couldn't get Employee table");

    assert_eq!(
        table.indices,
        &[Index {
            name: "age_and_name_index".into(),
            columns: vec!["name".to_owned(), "age".to_owned()],
            tpe: IndexType::Unique
        }]
    );
}

#[async_std::test]
async fn mysql_join_table_unique_indexes_must_be_inferred() {
    let db_name = "mysql_join_table_unique_indexes_must_be_inferred";

    let mut migration = Migration::new().schema(db_name);

    migration.create_table("Cat", move |t| {
        t.add_column("id", types::primary());
        t.add_column("name", types::text());
    });

    migration.create_table("Human", move |t| {
        t.add_column("id", types::primary());
        t.add_column("name", types::text());
    });

    migration.create_table("CatToHuman", move |t| {
        t.add_column("cat", types::foreign("Cat", "id").nullable(true));
        t.add_column("human", types::foreign("Human", "id").nullable(true));
        t.add_column("relationship", types::text());
        t.add_index("cat_and_human_index", types::index(vec!["cat", "human"]).unique(true));
    });

    let full_sql = migration.make::<barrel::backend::MySql>();
    let inspector = get_mysql_describer_for_schema(&full_sql, db_name).await;
    let result = inspector.describe(db_name).await.expect("describing");
    let table = result.get_table("CatToHuman").expect("couldn't get CatToHuman table");

    assert_eq!(
        table.indices,
        &[Index {
            name: "cat_and_human_index".into(),
            columns: vec!["cat".to_owned(), "human".to_owned()],
            tpe: IndexType::Unique
        }]
    );
}

// When multiple databases exist on a mysql instance, and they share names for foreign key
// constraints, introspecting one database should not yield constraints from the other.
#[async_std::test]
async fn constraints_from_other_databases_should_not_be_introspected() {
    let db_name = "constraints_from_other_databases_should_not_be_introspected";

    let mut other_migration = Migration::new().schema("other_schema");

    other_migration.create_table("User", |t| {
        t.add_column("id", types::primary());
    });
    other_migration.create_table("Post", |t| {
        t.add_column("id", types::primary());
        t.inject_custom("user_id INTEGER, FOREIGN KEY (`user_id`) REFERENCES `User`(`id`) ON DELETE CASCADE");
    });

    let full_sql = other_migration.make::<barrel::backend::MySql>();
    let inspector = get_mysql_describer_for_schema(&full_sql, "other_schema").await;

    let schema = inspector
        .describe(&"other_schema".to_string())
        .await
        .expect("describing");
    let table = schema.table_bang("Post");

    let fks = &table.foreign_keys;

    assert_eq!(
        fks,
        &[ForeignKey {
            constraint_name: Some("Post_ibfk_1".into()),
            columns: vec!["user_id".into()],
            referenced_table: "User".into(),
            referenced_columns: vec!["id".into()],
            on_delete_action: ForeignKeyAction::Cascade,
        }]
    );

    // Now the migration in the current database.

    let mut migration = Migration::new().schema(db_name);

    migration.create_table("User", |t| {
        t.add_column("id", types::primary());
    });

    migration.create_table("Post", |t| {
        t.add_column("id", types::primary());
        t.inject_custom("user_id INTEGER, FOREIGN KEY (`user_id`) REFERENCES `User`(`id`) ON DELETE RESTRICT");
    });

    let full_sql = migration.make::<barrel::backend::MySql>();
    let inspector = get_mysql_describer_for_schema(&full_sql, db_name).await;
    let schema = inspector.describe(db_name).await.expect("describing");
    let table = schema.table_bang("Post");

    let fks = &table.foreign_keys;

    assert_eq!(
        fks,
        &[ForeignKey {
            constraint_name: Some("Post_ibfk_1".into()),
            columns: vec!["user_id".into()],
            referenced_table: "User".into(),
            referenced_columns: vec!["id".into()],
            on_delete_action: ForeignKeyAction::Restrict,
        }]
    );
}

#[async_std::test]
async fn mysql_introspected_default_strings_should_be_unescaped() {
    let db_name = "mysql_introspected_default_strings_should_be_unescaped";

    let create_table = r#"
        CREATE TABLE `mysql_introspected_default_strings_should_be_unescaped`.`User` (
            id INTEGER PRIMARY KEY,
            favouriteQuote VARCHAR(500) NOT NULL DEFAULT '"That\'s a lot of fish!"\n - Godzilla, 1998'
        )
    "#;

    let inspector = get_mysql_describer_for_schema(create_table, db_name).await;
    let schema = inspector.describe(db_name).await.unwrap();

    let expected_default = prisma_value::PrismaValue::String(
        r#""That's a lot of fish!"
 - Godzilla, 1998"#
            .into(),
    );

    let table = schema.table_bang("User");
    let column = table.column_bang("favouriteQuote");

    let actual_default = column.default.as_ref().unwrap().as_value().unwrap();

    assert_eq!(actual_default, &expected_default);
}

#[test_each_connector(tags("mysql"))]
async fn escaped_quotes_in_string_defaults_must_be_unescaped(api: &TestApi) -> TestResult {
    let create_table = format!(
        r#"
            CREATE TABLE `{0}`.`string_defaults_test` (
                `id` INTEGER PRIMARY KEY,
                `regular` VARCHAR(200) NOT NULL DEFAULT 'meow, says the cat',
                `escaped` VARCHAR(200) NOT NULL DEFAULT '\"That\'s a lot of fish!\"\n- Godzilla, 1998'
            );
        "#,
        api.schema_name()
    );

    api.database().raw_cmd(&create_table).await?;

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

    assert_eq!(
        escaped_column_default,
        r#""That's a lot of fish!"
- Godzilla, 1998"#
    );

    Ok(())
}

#[test_each_connector(tags("mysql"))]
async fn escaped_backslashes_in_string_literals_must_be_unescaped(api: &TestApi) -> TestResult {
    let create_table = r#"
        CREATE TABLE test (
            `model_name_space` VARCHAR(255) NOT NULL DEFAULT 'xyz\\Datasource\\Model'
        )
    "#;

    api.database().raw_cmd(&create_table).await?;

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
