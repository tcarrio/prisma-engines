mod existing_data;

use migration_connector::MigrationWarning;
use migration_engine_tests::sql::*;
use pretty_assertions::assert_eq;
use prisma_value::{PrismaValue, TypeHint};
use quaint::ast::*;
use sql_schema_describer::DefaultValue;

#[test_each_connector]
async fn dropping_a_table_with_rows_should_warn(api: &TestApi) {
    let dm = r#"
        model Test {
            id String @id @default(cuid())
        }
    "#;
    let original_database_schema = api.infer_and_apply(&dm).await.sql_schema;

    let conn = api.database();
    let insert = Insert::single_into((api.schema_name(), "Test")).value("id", "test");

    conn.query(insert.into()).await.unwrap();

    let dm = "";

    let InferAndApplyOutput {
        migration_output,
        sql_schema: final_database_schema,
    } = api.infer_and_apply(&dm).await;

    // The schema should not change because the migration should not run if there are warnings
    // and the force flag isn't passed.
    assert_eq!(original_database_schema, final_database_schema);

    assert_eq!(
        migration_output.warnings,
        &[MigrationWarning {
            description: "You are about to drop the `Test` table, which is not empty (1 rows).".into()
        }]
    );
}

#[test_each_connector]
async fn dropping_a_column_with_non_null_values_should_warn(api: &TestApi) {
    let dm = r#"
            model Test {
                id String @id @default(cuid())
                puppiesCount Int?
            }
        "#;

    let original_database_schema = api.infer_and_apply(&dm).await.sql_schema;

    let insert = Insert::multi_into((api.schema_name(), "Test"), &["id", "puppiesCount"])
        .values(("a", 7))
        .values(("b", 8));

    api.database().query(insert.into()).await.unwrap();

    // Drop the `favouriteAnimal` column.
    let dm = r#"
            model Test {
                id String @id @default(cuid())
            }
        "#;

    let InferAndApplyOutput {
        migration_output,
        sql_schema: final_database_schema,
    } = api.infer_and_apply(&dm).await;

    // The schema should not change because the migration should not run if there are warnings
    // and the force flag isn't passed.
    assert_eq!(original_database_schema, final_database_schema);

    assert_eq!(
            migration_output.warnings,
            &[MigrationWarning {
                description: "You are about to drop the column `puppiesCount` on the `Test` table, which still contains 2 non-null values.".to_owned(),
            }]
        );
}

#[test_each_connector]
async fn altering_a_column_without_non_null_values_should_not_warn(api: &TestApi) {
    let dm = r#"
        model Test {
            id String @id @default(cuid())
            puppiesCount Int?
        }
    "#;

    let original_database_schema = api.infer_and_apply(&dm).await.sql_schema;

    let insert = Insert::multi_into((api.schema_name(), "Test"), &["id"])
        .values(("a",))
        .values(("b",));

    api.database().query(insert.into()).await.unwrap();

    let dm2 = r#"
        model Test {
            id String @id @default(cuid())
            puppiesCount Float?
        }
    "#;

    let result = api.infer_and_apply(&dm2).await;
    let final_database_schema = &result.sql_schema;

    assert_ne!(&original_database_schema, final_database_schema);
    assert!(result.migration_output.warnings.is_empty());
}

#[test_each_connector]
async fn altering_a_column_with_non_null_values_should_warn(api: &TestApi) -> TestResult {
    let dm = r#"
        model Test {
            id String @id @default(cuid())
            age Int?
        }
    "#;

    api.infer_apply(&dm).send().await?.assert_green()?;
    let original_database_schema = api.describe_database().await?;

    let insert = Insert::multi_into((api.schema_name(), "Test"), vec!["id", "age"])
        .values(("a", 12))
        .values(("b", 22));

    api.database().query(insert.into()).await.unwrap();

    let dm2 = r#"
        model Test {
            id String @id @default(cuid())
            age Float?
        }
    "#;

    let migration_output = api.infer_apply(&dm2).send().await?.into_inner();

    // The schema should not change because the migration should not run if there are warnings
    // and the force flag isn't passed.
    api.assert_schema().await?.assert_equals(&original_database_schema)?;

    assert_eq!(
        migration_output.warnings,
        &[MigrationWarning {
            description:
                "You are about to alter the column `age` on the `Test` table, which still contains 2 non-null values. \
                 The data in that column could be lost."
                    .to_owned()
        }]
    );

    let data = api.dump_table("Test").await?;
    assert_eq!(data.len(), 2);

    Ok(())
}

#[test_each_connector]
async fn column_defaults_can_safely_be_changed(api: &TestApi) -> TestResult {
    let combinations = &[
        ("Meow", Some(PrismaValue::String("Cats".to_string())), None),
        ("Freedom", None, Some(PrismaValue::String("Braveheart".to_string()))),
        (
            "OutstandingMovies",
            Some(PrismaValue::String("Cats".to_string())),
            Some(PrismaValue::String("Braveheart".to_string())),
        ),
    ];

    for (model_name, first_default, second_default) in combinations {
        let span = tracing::info_span!("Combination", model_name, ?first_default, ?second_default);
        let _combination_scope = span.enter();
        tracing::info!("Testing new combination");

        // Set up the initial schema
        {
            let dm1 = format!(
                r#"
                    model {} {{
                        id String @id
                        name String? {}
                    }}
                "#,
                model_name,
                first_default
                    .as_ref()
                    .map(|default| format!("@default(\"{}\")", default))
                    .unwrap_or_else(String::new)
            );

            api.infer_apply(&dm1).force(Some(true)).send().await?;

            api.assert_schema().await?.assert_table(model_name, |table| {
                table.assert_column("name", |column| {
                    if let Some(first_default) = first_default.as_ref() {
                        column.assert_default_value(first_default)
                    } else {
                        column.assert_has_no_default()
                    }
                })
            })?;
        }

        // Insert data
        {
            let insert_span = tracing::info_span!("Data insertion");
            let _insert_scope = insert_span.enter();

            let query = Insert::single_into(api.render_table_name(model_name)).value("id", "abc");

            api.database().query(query.into()).await?;

            let query = Insert::single_into(api.render_table_name(model_name))
                .value("id", "def")
                .value("name", "Waterworld");

            api.database().query(query.into()).await?;

            let data = api.dump_table(model_name).await?;
            let names: Vec<PrismaValue> = data
                .into_iter()
                .filter_map(|row| {
                    row.get("name").map(|val| {
                        val.to_string()
                            .map(|val| PrismaValue::String(val))
                            .unwrap_or(PrismaValue::Null(TypeHint::String))
                    })
                })
                .collect();

            assert_eq!(
                &[
                    first_default
                        .as_ref()
                        .cloned()
                        .unwrap_or(PrismaValue::Null(TypeHint::String)),
                    PrismaValue::String("Waterworld".to_string())
                ],
                names.as_slice()
            );
        }

        // Migrate
        {
            let dm2 = format!(
                r#"
                    model {} {{
                        id String @id
                        name String? {}
                    }}
                "#,
                model_name,
                second_default
                    .as_ref()
                    .map(|default| format!(r#"@default("{}")"#, default))
                    .unwrap_or_else(String::new)
            );

            api.infer_apply(&dm2).send().await?.assert_green()?;
        }

        // Check that the data is still there
        {
            let data = api.dump_table(model_name).await?;
            let names: Vec<PrismaValue> = data
                .into_iter()
                .filter_map(|row| {
                    row.get("name").map(|val| {
                        val.to_string()
                            .map(|val| PrismaValue::String(val))
                            .unwrap_or(PrismaValue::Null(TypeHint::String))
                    })
                })
                .collect();
            assert_eq!(
                &[
                    first_default
                        .as_ref()
                        .cloned()
                        .unwrap_or(PrismaValue::Null(TypeHint::String)),
                    PrismaValue::String("Waterworld".to_string())
                ],
                names.as_slice()
            );

            api.assert_schema().await?.assert_table(model_name, |table| {
                table.assert_column("name", |column| {
                    if let Some(second_default) = second_default.as_ref() {
                        column.assert_default_value(second_default)
                    } else {
                        column.assert_has_no_default()
                    }
                })
            })?;
        }
    }

    Ok(())
}

#[test_each_connector]
async fn changing_a_column_from_required_to_optional_should_work(api: &TestApi) -> TestResult {
    let dm = r#"
        model Test {
            id String @id @default(cuid())
            age Int @default(30)
        }
    "#;

    api.infer_apply(&dm).send().await?.assert_green()?;
    let original_database_schema = api.describe_database().await?;

    let insert = Insert::multi_into((api.schema_name(), "Test"), &["id", "age"])
        .values(("a", 12))
        .values(("b", 22));

    api.database().query(insert.into()).await.unwrap();

    let dm2 = r#"
        model Test {
            id String @id @default(cuid())
            age Int? @default(30)
        }
    "#;

    let migration_output = api.infer_apply(&dm2).send().await?.into_inner();

    // On MySQL we can't safely restate the type in a CHANGE clause, so this change is still destructive.
    if api.is_mysql() {
        anyhow::ensure!(
            migration_output.warnings.len() == 1,
            "Migration warnings should have one warning on mysql. Got {:#?}",
            migration_output.warnings
        );

        assert_eq!(
            migration_output.warnings.get(0).unwrap().description,
            "You are about to alter the column `age` on the `Test` table, which still contains 2 non-null values. The data in that column could be lost.",
        );

        api.assert_schema().await?.assert_equals(&original_database_schema)?;
    } else {
        // On other databases, the migration should be successful.
        anyhow::ensure!(
            migration_output.warnings.is_empty(),
            "Migration warnings should be empty. Got {:#?}",
            migration_output.warnings
        );

        api.assert_schema().await?.assert_ne(&original_database_schema)?;
    }

    // Check that no data was lost.
    {
        let data = api.dump_table("Test").await?;
        assert_eq!(data.len(), 2);
        let ages: Vec<i64> = data
            .into_iter()
            .map(|row| row.get("age").unwrap().as_i64().unwrap())
            .collect();

        assert_eq!(ages, &[12, 22]);
    }

    Ok(())
}

#[test_each_connector(ignore("sqlite"))]
async fn changing_a_column_from_optional_to_required_is_unexecutable(api: &TestApi) -> TestResult {
    let dm = r#"
        model Test {
            id String @id @default(cuid())
            age Int?
        }
    "#;

    api.infer_apply(&dm).send().await?.assert_green()?;
    let original_database_schema = api.describe_database().await?;

    let insert = Insert::multi_into((api.schema_name(), "Test"), &["id", "age"])
        .values(("a", 12))
        .values(("b", 22))
        .values(("c", Value::Integer(None)));

    api.database().query(insert.into()).await.unwrap();

    let dm2 = r#"
        model Test {
            id String @id @default(cuid())
            age Int @default(30)
        }
    "#;

    api.infer_apply(&dm2)
        .send()
        .await?
        .assert_warnings(&[
            // This only warns temporarily: it is not executable.
            "Made the column `age` on table `Test` required, but there are 1 existing NULL values.".into(),
        ])?
        .assert_unexecutable(&[
            "Made the column `age` on table `Test` required, but there are 1 existing NULL values.".into(),
        ])?
        .assert_no_error()?;

    // The schema should not change because the migration should not run if there are warnings
    // and the force flag isn't passed.
    api.assert_schema().await?.assert_equals(&original_database_schema)?;

    // Check that no data was lost.
    {
        let data = api.dump_table("Test").await?;
        assert_eq!(data.len(), 3);
        let ages: Vec<Option<i64>> = data.into_iter().map(|row| row.get("age").unwrap().as_i64()).collect();

        assert_eq!(ages, &[Some(12), Some(22), None]);
    }

    Ok(())
}

#[test_each_connector(tags("sqlite"))]
async fn changing_a_column_from_optional_to_required_must_warn(api: &TestApi) -> TestResult {
    let dm = r#"
        model Test {
            id String @id @default(cuid())
            age Int?
        }
    "#;

    api.infer_apply(&dm).send().await?.assert_green()?;

    let insert = Insert::multi_into((api.schema_name(), "Test"), &["id", "age"])
        .values(("a", 12))
        .values(("b", 22))
        .values(("c", Value::Integer(None)));

    api.database().query(insert.into()).await.unwrap();

    let dm2 = r#"
        model Test {
            id String @id @default(cuid())
            age Int @default(30)
        }
    "#;

    // TODO: this should not warn, since this specific migration is safe.
    api.infer_apply(&dm2)
        .force(Some(true))
        .send()
        .await?
        .assert_executable()?
        .assert_no_error()?
        .assert_warnings(&["You are about to alter the column `age` on the `Test` table, which still contains 2 non-null values. The data in that column could be lost.".into()])?;

    api.assert_schema().await?.assert_table("Test", |table| {
        table.assert_column("age", |column| {
            column
                .assert_default(Some(DefaultValue::VALUE(PrismaValue::Int(30))))?
                .assert_is_required()
        })
    })?;

    // Check that no data was lost.
    {
        let data = api.dump_table("Test").await?;
        assert_eq!(data.len(), 3);
        let ages: Vec<Option<i64>> = data.into_iter().map(|row| row.get("age").unwrap().as_i64()).collect();

        // TODO: this is NOT what users would expect (it's a consequence of the stepped migration
        // process), we should have a more specific warning for this.
        assert_eq!(ages, &[Some(12), Some(22), Some(30)]);
    }

    Ok(())
}

#[test_each_connector(tags("sql"))]
async fn dropping_a_table_referenced_by_foreign_keys_must_work(api: &TestApi) -> TestResult {
    use quaint::ast::*;

    let dm1 = r#"
        model Category {
            id Int @id
            name String
        }

        model Recipe {
            id Int @id
            categoryId Int
            category Category @relation(fields: [categoryId], references: [id])
        }
    "#;

    api.infer_apply(&dm1).send().await?.assert_green()?;

    api.assert_schema()
        .await?
        .assert_table("Category", |table| table.assert_columns_count(2))?
        .assert_table("Recipe", |table| {
            table.assert_fk_on_columns(&["categoryId"], |fk| fk.assert_references("Category", &["id"]))
        })?;

    let id: i32 = 1;

    let insert = Insert::single_into(api.render_table_name("Category"))
        .value("name", "desserts")
        .value("id", id);
    api.database().query(insert.into()).await?;

    let insert = Insert::single_into(api.render_table_name("Recipe"))
        .value("categoryId", id)
        .value("id", id);
    api.database().query(insert.into()).await?;

    let dm2 = r#"
        model Recipe {
            id Int @id
        }
    "#;

    api.infer_apply(dm2).force(Some(true)).send().await?.into_inner();
    let sql_schema = api.describe_database().await.unwrap();

    assert!(sql_schema.table("Category").is_err());
    assert!(sql_schema.table_bang("Recipe").foreign_keys.is_empty());

    Ok(())
}

#[test_each_connector]
async fn string_columns_do_not_get_arbitrarily_migrated(api: &TestApi) -> TestResult {
    use quaint::ast::*;

    let dm1 = r#"
        model User {
            id           String  @default(cuid()) @id
            name         String?
            email        String  @unique
            kindle_email String? @unique
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    let insert = Insert::single_into(api.render_table_name("User"))
        .value("id", "the-id")
        .value("name", "George")
        .value("email", "george@prisma.io")
        .value("kindle_email", "george+kindle@prisma.io");

    api.database().query(insert.into()).await?;

    let dm2 = r#"
        model User {
            id           String  @default(cuid()) @id
            name         String?
            email        String  @unique
            kindle_email String? @unique
            count        Int     @default(0)
        }
    "#;

    let output = api.infer_apply(dm2).send().await?.assert_green()?.into_inner();

    assert!(output.warnings.is_empty());

    // Check that the string values are still there.
    let select = Select::from_table(api.render_table_name("User"))
        .column("name")
        .column("kindle_email")
        .column("email");

    let counts = api.database().query(select.into()).await?;

    let row = counts.get(0).unwrap();

    assert_eq!(row.get("name").unwrap().as_str().unwrap(), "George");
    assert_eq!(
        row.get("kindle_email").unwrap().as_str().unwrap(),
        "george+kindle@prisma.io"
    );
    assert_eq!(row.get("email").unwrap().as_str().unwrap(), "george@prisma.io");

    Ok(())
}

#[test_each_connector]
async fn altering_the_type_of_a_column_in_an_empty_table_should_not_warn(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model User {
            id String @id @default(cuid())
            name String
            dogs Int
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    let dm2 = r#"
        model User {
            id String @id @default(cuid())
            name String
            dogs String
        }
    "#;

    let response = api.infer_apply(dm2).send().await?.assert_green()?.into_inner();

    assert!(response.warnings.is_empty());

    api.assert_schema()
        .await?
        .assert_table("User", |table| {
            table.assert_column("dogs", |col| col.assert_type_is_string()?.assert_is_required())
        })
        .map(drop)
}

#[test_each_connector]
async fn making_a_column_required_in_an_empty_table_should_not_warn(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model User {
            id String @id @default(cuid())
            name String
            dogs Int?
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    let dm2 = r#"
        model User {
            id String @id @default(cuid())
            name String
            dogs Int
        }
    "#;

    let response = api.infer_apply(dm2).send().await?.assert_green()?.into_inner();

    assert!(response.warnings.is_empty());

    api.assert_schema()
        .await?
        .assert_table("User", |table| {
            table.assert_column("dogs", |col| col.assert_type_is_int()?.assert_is_required())
        })
        .map(drop)
}

#[test_each_connector]
async fn altering_the_type_of_a_column_in_a_non_empty_table_always_warns(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model User {
            id String @id @default(cuid())
            name String
            dogs Int
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    let insert = quaint::ast::Insert::single_into(api.render_table_name("User"))
        .value("id", "abc")
        .value("name", "Shinzo")
        .value("dogs", 7);

    api.database().query(insert.into()).await?;

    let dm2 = r#"
        model User {
            id String @id @default(cuid())
            name String
            dogs String
        }
    "#;

    let response = api.infer_apply(dm2).send().await?.into_inner();

    assert_eq!(
        response.warnings,
        &[MigrationWarning {
            // TODO: the message should say that altering the type of a column is not guaranteed to preserve the data, but the database is going to do its best.
            // Also think about timeouts.
            description: "You are about to alter the column `dogs` on the `User` table, which still contains 1 non-null values. The data in that column could be lost.".to_owned()
        }]
    );

    let rows = api.select("User").column("dogs").send_debug().await?;
    assert_eq!(rows, &[["Integer(Some(7))"]]);

    api.assert_schema().await?.assert_table("User", |table| {
        table.assert_column("dogs", |col| col.assert_type_is_int()?.assert_is_required())
    })?;

    Ok(())
}

#[test_each_connector(ignore("mysql"))]
async fn migrating_a_required_column_from_int_to_string_should_warn_and_cast(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model Test {
            id String @id
            serialNumber Int
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    api.insert("Test")
        .value("id", "abcd")
        .value("serialNumber", 47i64)
        .result_raw()
        .await?;

    let test = api.dump_table("Test").await?;
    let first_row = test.get(0).unwrap();
    assert_eq!(
        format!("{:?} {:?}", first_row.get("id"), first_row.get("serialNumber")),
        r#"Some(Text(Some("abcd"))) Some(Integer(Some(47)))"#
    );

    let original_schema = api.assert_schema().await?.into_schema();

    let dm2 = r#"
        model Test {
            id String @id
            serialNumber String
        }
    "#;

    let expected_warning = MigrationWarning {
        description: "You are about to alter the column `serialNumber` on the `Test` table, which still contains 1 non-null values. The data in that column could be lost.".to_owned(),
    };

    // Apply once without forcing
    {
        let result = api.infer_apply(dm2).send().await?.into_inner();

        assert_eq!(result.warnings, &[expected_warning.clone()]);

        api.assert_schema().await?.assert_equals(&original_schema)?;
    }

    // Force apply
    {
        let result = api.infer_apply(dm2).force(Some(true)).send().await?.into_inner();

        assert_eq!(result.warnings, &[expected_warning]);

        api.assert_schema().await?.assert_table("Test", |table| {
            table.assert_column("serialNumber", |col| col.assert_type_is_string())
        })?;

        let test = api.dump_table("Test").await?;
        let first_row = test.get(0).unwrap();
        assert_eq!(
            format!("{:?} {:?}", first_row.get("id"), first_row.get("serialNumber")),
            r#"Some(Text(Some("abcd"))) Some(Text(Some("47")))"#
        );
    }

    Ok(())
}

#[test_each_connector(capabilities("enums"))]
async fn enum_variants_can_be_added_without_data_loss(api: &TestApi) -> TestResult {
    let dm1 = r"
        model Cat {
            id String @id
            mood Mood
        }

        model Human {
            id String @id
            mood Mood
        }

        enum Mood {
            HAPPY
            HUNGRY
        }
    "#;

    dbg!(1);
    api.infer_apply(dm1)
        .migration_id(Some("initial-setup"))
        .send()
        .await?
        .assert_green()?;

    {
        dbg!(1);
        let cat_inserts = quaint::ast::Insert::multi_into(api.render_table_name("Cat"), vec!["id", "mood"])
            .values((Value::text("felix"), Value::enum_variant("HUNGRY")))
            .values((Value::text("mittens"), Value::enum_variant("HAPPY")));

        dbg!(1);
        api.database().query(cat_inserts.into()).await?;
    }

    let dm2 = r#"
        model Cat {
            id String @id
            mood Mood
        }

        model Human {
            id String @id
            mood Mood
        }

        enum Mood {
            ABSOLUTELY_FABULOUS
            HAPPY
            HUNGRY
        }
    "#;

    dbg!(1);
    api.infer_apply(dm2)
        .migration_id(Some("add-absolutely-fabulous-variant"))
        .send()
        .await?
        .assert_green()?;
    dbg!(1);

    // Assertions
    {
        let cat_data = api.dump_table("Cat").await?;
        dbg!(1);
        let cat_data: Vec<Vec<quaint::ast::Value>> =
            cat_data.into_iter().map(|row| row.into_iter().collect()).collect();
        dbg!(1);

        let expected_cat_data = if api.sql_family().is_mysql() {
            vec![
                vec![Value::text("felix"), Value::text("HUNGRY")],
                vec![Value::text("mittens"), Value::text("HAPPY")],
            ]
        } else {
            vec![
                vec![Value::text("felix"), Value::enum_variant("HUNGRY")],
                vec![Value::text("mittens"), Value::enum_variant("HAPPY")],
            ]
        };
        dbg!(1);

        assert_eq!(cat_data, expected_cat_data);

        dbg!(1);
        let human_data = api.dump_table("Human").await?;
        dbg!(1);
        let human_data: Vec<Vec<Value>> = human_data.into_iter().map(|row| row.into_iter().collect()).collect();
        dbg!(1);
        let expected_human_data: Vec<Vec<Value>> = Vec::new();
        dbg!(1);
        assert_eq!(human_data, expected_human_data);
        dbg!(1);

        if api.sql_family().is_mysql() {
            dbg!(1);
            api.assert_schema()
                .await?
                .assert_enum("Cat_mood", |enm| {
                    enm.assert_values(&["HAPPY", "HUNGRY", "ABSOLUTELY_FABULOUS"])
                })?
                .assert_enum("Human_mood", |enm| {
                    enm.assert_values(&["HAPPY", "HUNGRY", "ABSOLUTELY_FABULOUS"])
                })?;
        } else {
            dbg!(1);
            api.assert_schema().await?.assert_enum("Mood", |enm| {
                enm.assert_values(&["ABSOLUTELY_FABULOUS", "HAPPY", "HUNGRY"])
            })?;
        };
    }

    Ok(())
}

#[test_each_connector(capabilities("enums"))]
async fn enum_variants_can_be_dropped_without_data_loss(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model Cat {
            id String @id
            mood Mood
        }

        model Human {
            id String @id
            mood Mood
        }

        enum Mood {
            OUTRAGED
            HAPPY
            HUNGRY
        }
    "#;

    api.infer_apply(dm1)
        .migration_id(Some("initial-setup"))
        .send()
        .await?
        .assert_green()?;

    {
        let cat_inserts = quaint::ast::Insert::multi_into(api.render_table_name("Cat"), &["id", "mood"])
            .values((Value::text("felix"), Value::enum_variant("HUNGRY")))
            .values((Value::text("mittens"), Value::enum_variant("HAPPY")));

        api.database().query(cat_inserts.into()).await?;
    }

    let dm2 = r#"
        model Cat {
            id String @id
            mood Mood
        }

        model Human {
            id String @id
            mood Mood
        }

        enum Mood {
            HAPPY
            HUNGRY
        }
    "#;

    api.infer_apply(dm2)
        .migration_id(Some("add-absolutely-fabulous-variant"))
        .send()
        .await?
        .assert_green()?;

    // Assertions
    {
        let cat_data = api.dump_table("Cat").await?;
        let cat_data: Vec<Vec<quaint::ast::Value>> =
            cat_data.into_iter().map(|row| row.into_iter().collect()).collect();

        let expected_cat_data = if api.sql_family().is_mysql() {
            vec![
                vec![Value::text("felix"), Value::text("HUNGRY")],
                vec![Value::text("mittens"), Value::text("HAPPY")],
            ]
        } else {
            vec![
                vec![Value::text("felix"), Value::enum_variant("HUNGRY")],
                vec![Value::text("mittens"), Value::enum_variant("HAPPY")],
            ]
        };

        assert_eq!(cat_data, expected_cat_data);

        let human_data = api.dump_table("Human").await?;
        let human_data: Vec<Vec<Value>> = human_data.into_iter().map(|row| row.into_iter().collect()).collect();
        let expected_human_data: Vec<Vec<Value>> = Vec::new();
        assert_eq!(human_data, expected_human_data);

        if api.sql_family().is_mysql() {
            api.assert_schema()
                .await?
                .assert_enum("Cat_mood", |enm| enm.assert_values(&["HAPPY", "HUNGRY"]))?
                .assert_enum("Human_mood", |enm| enm.assert_values(&["HAPPY", "HUNGRY"]))?;
        } else {
            api.assert_schema()
                .await?
                .assert_enum("Mood", |enm| enm.assert_values(&["HAPPY", "HUNGRY"]))?;
        };
    }

    Ok(())
}

#[test_each_connector(log = "debug,sql_schema_describer=info")]
async fn set_default_current_timestamp_on_existing_column_works(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model User {
            id Int @id
            created_at DateTime
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    let conn = api.database();
    let insert = Insert::single_into(api.render_table_name("User")).value("id", 5).value(
        "created_at",
        Value::DateTime(Some("2020-06-15T14:50:00Z".parse().unwrap())),
    );
    conn.execute(insert.into()).await?;

    let dm2 = r#"
        model User {
            id Int @id
            created_at DateTime @default(now())
        }
    "#;

    api.infer_apply(dm2)
        .force(Some(true))
        .send()
        .await?
        .assert_executable()?
        .assert_warnings(&[])?;

    api.assert_schema().await?.assert_table("User", |table| {
        table.assert_column("created_at", |column| column.assert_default(Some(DefaultValue::NOW)))
    })?;

    Ok(())
}

#[test_each_connector(capabilities("scalar_lists"), log = "debug,sql_schema_describer=info")]
async fn changing_an_array_column_to_scalar_must_warn(api: &TestApi) -> TestResult {
    let datasource_block = api.datasource();

    let dm1 = format!(
        r#"
        {datasource_block}

        model Film {{
            id String @id
            mainProtagonist String[]
        }}
        "#,
        datasource_block = datasource_block,
    );

    api.infer_apply(&dm1).send().await?.assert_green()?;

    api.insert("Film")
        .value("id", "film1")
        .value(
            "mainProtagonist",
            Value::Array(Some(vec!["giant shark".into(), "jason statham".into()])),
        )
        // .value("mainProtagonist", Value::array(vec!["giant shark", "jason statham"]))
        .result_raw()
        .await?;

    let dm2 = format!(
        r#"
            {datasource_block}

            model Film {{
                id String @id
                mainProtagonist String
            }}
            "#,
        datasource_block = datasource_block,
    );

    api.infer_apply(&dm2)
        .force(Some(true))
        .send()
        .await?
        .assert_executable()?
        .assert_no_error()?
        .assert_warnings(&["You are about to alter the column `mainProtagonist` on the `Film` table, which still contains 1 non-null values. The data in that column could be lost.".into()])?;

    api.assert_schema().await?.assert_table("Film", |table| {
        table.assert_column("mainProtagonist", |column| column.assert_is_required())
    })?;

    let rows = api.select("Film").column("id").column("mainProtagonist").send().await?;

    let rows: Vec<Vec<Value>> = rows
        .into_iter()
        .map(|row| row.into_iter().collect::<Vec<_>>())
        .collect();

    assert_eq!(
        rows,
        &[&["film1".into(), "{\"giant shark\",\"jason statham\"}".into()]] // the array got cast ot a string by postgres
    );

    Ok(())
}

#[test_each_connector(capabilities("scalar_lists"))]
async fn changing_a_scalar_column_to_an_array_is_unexecutable(api: &TestApi) -> TestResult {
    let datasource_block = api.datasource();

    let dm1 = format!(
        r#"
        {datasource_block}

        model Film {{
            id String @id
            mainProtagonist String
        }}
        "#,
        datasource_block = datasource_block,
    );

    api.infer_apply(&dm1).send().await?.assert_green()?;

    api.insert("Film")
        .value("id", "film1")
        .value("mainProtagonist", "left shark")
        // .value("mainProtagonist", Value::array(vec!["giant shark", "jason statham"]))
        .result_raw()
        .await?;

    let dm2 = format!(
        r#"
            {datasource_block}

            model Film {{
                id String @id
                mainProtagonist String[]
            }}
            "#,
        datasource_block = datasource_block,
    );

    api.infer_apply(&dm2)
        .send()
        .await?
        .assert_unexecutable(&[
            "Changed the column `mainProtagonist` on the `Film` table from a scalar field to a list field. There are 1 existing non-null values in that column, this migration step cannot be executed.".into(),
        ])?
        // TEMPORARY, until the CLI displays unexecutable migrations.
        .assert_warnings(&["Changed the column `mainProtagonist` on the `Film` table from a scalar field to a list field. There are 1 existing non-null values in that column, this migration step cannot be executed.".into()])?
        .assert_no_error()?;

    api.assert_schema().await?.assert_table("Film", |table| {
        table.assert_column("mainProtagonist", |column| column.assert_is_required())
    })?;

    let rows = api.select("Film").column("id").column("mainProtagonist").send().await?;

    let rows: Vec<Vec<Value>> = rows
        .into_iter()
        .map(|row| row.into_iter().collect::<Vec<_>>())
        .collect();

    assert_eq!(rows, &[&["film1".into(), Value::text("left shark")]]);

    Ok(())
}

#[test_each_connector(log = "debug")]
async fn primary_key_migrations_do_not_cause_data_loss(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model Dog {
            name String
            passportNumber Int

            @@id([name, passportNumber])
        }

        model Puppy {
            id String @id
            motherName String
            motherPassportNumber Int
            mother Dog @relation(fields: [motherName, motherPassportNumber], references: [name, passportNumber])
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    api.insert("Dog")
        .value("name", "Marnie")
        .value("passportNumber", 8000)
        .result_raw()
        .await?;

    api.insert("Puppy")
        .value("id", "12345")
        .value("motherName", "Marnie")
        .value("motherPassportNumber", 8000)
        .result_raw()
        .await?;

    let dm2 = r#"
        model Dog {
            name String
            passportNumber String

            @@id([name, passportNumber])
        }

        model Puppy {
            id String @id
            motherName String
            motherPassportNumber String
            mother Dog @relation(fields: [motherName, motherPassportNumber], references: [name, passportNumber])
        }
    "#;

    api.infer_apply(dm2)
        .force(Some(true))
        .send()
        .await?
        .assert_executable()?
        .assert_no_error()?
        .assert_warnings(&[
            "The migration will change the primary key for the `Dog` table. If it partially fails, the table could be left without primary key constraint.".into(),
            "You are about to alter the column `passportNumber` on the `Dog` table, which still contains 1 non-null values. The data in that column could be lost.".into(),
            "You are about to alter the column `motherPassportNumber` on the `Puppy` table, which still contains 1 non-null values. The data in that column could be lost.".into(),
        ])?;

    api.assert_schema().await?.assert_table("Dog", |table| {
        table.assert_pk(|pk| pk.assert_columns(&["name", "passportNumber"]))
    })?;

    let dog = api.select("Dog").column("name").column("passportNumber").send().await?;
    let dog_row: Vec<quaint::Value> = dog.into_single().unwrap().into_iter().collect();
    assert_eq!(dog_row, &[Value::text("Marnie"), Value::text("8000")]);

    let puppy = api
        .select("Puppy")
        .column("id")
        .column("motherName")
        .column("motherPassportNumber")
        .send()
        .await?;

    let puppy_row: Vec<quaint::Value> = puppy.into_single().unwrap().into_iter().collect();
    assert_eq!(
        puppy_row,
        &[Value::text("12345"), Value::text("Marnie"), Value::text("8000")]
    );

    Ok(())
}
