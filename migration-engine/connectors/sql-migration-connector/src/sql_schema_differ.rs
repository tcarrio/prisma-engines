mod column;
mod enums;
mod index;
mod table;

pub(crate) use column::{ColumnChange, ColumnChanges, ColumnDiffer};
pub(crate) use table::TableDiffer;

use crate::*;
use enums::EnumDiffer;
use once_cell::sync::Lazy;
use regex::RegexSet;
use sql_schema_describer::*;
use sql_schema_helpers::ForeignKeyRef;
use sql_schema_helpers::TableRef;

#[derive(Debug)]
pub(crate) struct DiffingOptions {
    is_mariadb: bool,
    sql_family: SqlFamily,
    ignore_tables: &'static RegexSet,
}

impl DiffingOptions {
    pub(crate) fn sql_family(&self) -> SqlFamily {
        self.sql_family
    }

    pub(crate) fn from_database_info(database_info: &DatabaseInfo) -> Self {
        DiffingOptions {
            is_mariadb: database_info.is_mariadb(),
            ignore_tables: match database_info.sql_family() {
                SqlFamily::Postgres => &POSTGRES_IGNORED_TABLES,
                _ => &EMPTY_REGEXSET,
            },
            sql_family: database_info.sql_family(),
        }
    }
}

#[cfg(test)]
impl Default for DiffingOptions {
    fn default() -> Self {
        DiffingOptions {
            is_mariadb: false,
            ignore_tables: &EMPTY_REGEXSET,
            sql_family: SqlFamily::Postgres,
        }
    }
}

#[derive(Debug)]
pub struct SqlSchemaDiffer<'a> {
    previous: &'a SqlSchema,
    next: &'a SqlSchema,
    sql_family: SqlFamily,
    diffing_options: &'a DiffingOptions,
}

#[derive(Debug, Clone)]
pub struct SqlSchemaDiff {
    pub add_foreign_keys: Vec<AddForeignKey>,
    pub drop_foreign_keys: Vec<DropForeignKey>,
    pub drop_tables: Vec<DropTable>,
    pub create_tables: Vec<CreateTable>,
    pub alter_tables: Vec<AlterTable>,
    pub create_indexes: Vec<CreateIndex>,
    pub drop_indexes: Vec<DropIndex>,
    pub alter_indexes: Vec<AlterIndex>,
    pub create_enums: Vec<CreateEnum>,
    pub drop_enums: Vec<DropEnum>,
    pub alter_enums: Vec<AlterEnum>,
}

impl SqlSchemaDiff {
    /// Translate the diff into steps that should be executed in order. The general idea in the
    /// ordering of steps is to drop obsolete constraints first, alter/create tables, then add the new constraints.
    pub fn into_steps(self) -> Vec<SqlMigrationStep> {
        wrap_as_step(self.create_enums, SqlMigrationStep::CreateEnum)
            .chain(wrap_as_step(self.alter_enums, SqlMigrationStep::AlterEnum))
            .chain(wrap_as_step(self.drop_indexes, SqlMigrationStep::DropIndex))
            .chain(wrap_as_step(self.drop_foreign_keys, SqlMigrationStep::DropForeignKey))
            .chain(wrap_as_step(self.create_tables, SqlMigrationStep::CreateTable))
            .chain(wrap_as_step(self.alter_tables, SqlMigrationStep::AlterTable))
            // Order matters: we must create indexes after ALTER TABLEs because the indexes can be
            // on fields that are dropped/created there.
            .chain(wrap_as_step(self.create_indexes, SqlMigrationStep::CreateIndex))
            // Order matters: this needs to come after create_indexes, because the foreign keys can depend on unique
            // indexes created there.
            .chain(wrap_as_step(self.add_foreign_keys, SqlMigrationStep::AddForeignKey))
            .chain(wrap_as_step(self.drop_tables, SqlMigrationStep::DropTable))
            .chain(wrap_as_step(self.drop_enums, SqlMigrationStep::DropEnum))
            .chain(wrap_as_step(self.alter_indexes, SqlMigrationStep::AlterIndex))
            .collect()
    }
}

impl<'schema> SqlSchemaDiffer<'schema> {
    pub(crate) fn diff(
        previous: &SqlSchema,
        next: &SqlSchema,
        sql_family: SqlFamily,
        options: &DiffingOptions,
    ) -> SqlSchemaDiff {
        let differ = SqlSchemaDiffer {
            previous,
            next,
            sql_family,
            diffing_options: &options,
        };
        differ.diff_internal()
    }

    fn diff_internal(&self) -> SqlSchemaDiff {
        let alter_indexes: Vec<_> = self.alter_indexes();
        let (drop_tables, mut drop_foreign_keys) = self.drop_tables();
        self.drop_foreign_keys(&mut drop_foreign_keys);

        SqlSchemaDiff {
            add_foreign_keys: self.add_foreign_keys(),
            drop_foreign_keys,
            drop_tables,
            create_tables: self.create_tables(),
            alter_tables: self.alter_tables(),
            create_indexes: self.create_indexes(),
            drop_indexes: self.drop_indexes(),
            alter_indexes,
            create_enums: self.create_enums(),
            drop_enums: self.drop_enums(),
            alter_enums: self.alter_enums(),
        }
    }

    fn create_tables(&self) -> Vec<CreateTable> {
        self.created_tables()
            .map(|created_table| CreateTable {
                table: created_table.clone(),
            })
            .collect()
    }

    // We drop the foreign keys of dropped tables first, so we can drop tables in whatever order we
    // please later.
    fn drop_tables(&self) -> (Vec<DropTable>, Vec<DropForeignKey>) {
        let (dropped_tables_count, dropped_fks_count) = self.dropped_tables().fold((0, 0), |(tables, fks), item| {
            (tables + 1, fks + item.foreign_keys.len())
        });
        let mut dropped_tables = Vec::with_capacity(dropped_tables_count);
        let mut dropped_foreign_keys = Vec::with_capacity(dropped_fks_count);

        for dropped_table in self.dropped_tables() {
            let drop_table = DropTable {
                name: dropped_table.name.clone(),
            };

            dropped_tables.push(drop_table);

            for fk_name in dropped_table
                .foreign_keys
                .iter()
                .filter_map(|fk| fk.constraint_name.as_ref())
            {
                let drop_foreign_key = DropForeignKey {
                    table: dropped_table.name.clone(),
                    constraint_name: fk_name.clone(),
                };

                dropped_foreign_keys.push(drop_foreign_key);
            }
        }

        (dropped_tables, dropped_foreign_keys)
    }

    fn add_foreign_keys(&self) -> Vec<AddForeignKey> {
        let mut add_foreign_keys = Vec::new();

        push_foreign_keys_from_created_tables(&mut add_foreign_keys, self.created_tables());
        push_created_foreign_keys(&mut add_foreign_keys, self.table_pairs());

        add_foreign_keys
    }

    fn alter_tables(&self) -> Vec<AlterTable> {
        self.table_pairs()
            .filter_map(|tables| {
                // Order matters.
                let changes: Vec<TableChange> = Self::drop_primary_key(&tables)
                    .into_iter()
                    .chain(Self::drop_columns(&tables))
                    .chain(Self::add_columns(&tables))
                    .chain(Self::alter_columns(&tables))
                    .chain(Self::add_primary_key(&tables))
                    .collect();

                Some(changes)
                    .filter(|changes| !changes.is_empty())
                    .map(|changes| AlterTable {
                        table: tables.next.table.clone(),
                        changes,
                    })
            })
            .collect()
    }

    fn drop_columns<'a>(differ: &'a TableDiffer<'schema>) -> impl Iterator<Item = TableChange> + 'a {
        differ.dropped_columns().map(|column| {
            let change = DropColumn {
                name: column.name().to_owned(),
            };

            TableChange::DropColumn(change)
        })
    }

    fn add_columns<'a>(differ: &'a TableDiffer<'schema>) -> impl Iterator<Item = TableChange> + 'a {
        differ.added_columns().map(move |column| {
            let change = AddColumn {
                column: column.column.clone(),
            };

            TableChange::AddColumn(change)
        })
    }

    fn alter_columns<'a>(table_differ: &'a TableDiffer<'schema>) -> impl Iterator<Item = TableChange> + 'a {
        table_differ.column_pairs().filter_map(move |column_differ| {
            if column_differ.differs_in_something() {
                let change = AlterColumn {
                    name: column_differ.previous.name().to_owned(),
                    column: column_differ.next.column.clone(),
                };

                return Some(TableChange::AlterColumn(change));
            }

            None
        })
    }

    fn drop_foreign_keys<'a>(&'a self, drop_foreign_keys: &mut Vec<DropForeignKey>) {
        for differ in self.table_pairs() {
            let table_name = differ.previous.name();
            for dropped_foreign_key_name in differ
                .dropped_foreign_keys()
                .filter_map(|foreign_key| foreign_key.constraint_name())
            {
                drop_foreign_keys.push(DropForeignKey {
                    table: table_name.to_owned(),
                    constraint_name: dropped_foreign_key_name.to_owned(),
                })
            }
        }
    }

    fn add_primary_key(differ: &TableDiffer<'_>) -> Option<TableChange> {
        differ
            .created_primary_key()
            .filter(|pk| !pk.columns.is_empty())
            .map(|pk| TableChange::AddPrimaryKey {
                columns: pk.columns.clone(),
            })
    }

    fn drop_primary_key(differ: &TableDiffer<'_>) -> Option<TableChange> {
        differ.dropped_primary_key().map(|pk| TableChange::DropPrimaryKey {
            constraint_name: pk.constraint_name.clone(),
        })
    }

    fn create_indexes(&self) -> Vec<CreateIndex> {
        let mut steps = Vec::new();

        for table in self.created_tables() {
            for index in &table.indices {
                let create = CreateIndex {
                    table: table.name.clone(),
                    index: index.clone(),
                };

                steps.push(create)
            }
        }

        for tables in self.table_pairs() {
            for index in tables.created_indexes() {
                let create = CreateIndex {
                    table: tables.next.name().to_owned(),
                    index: index.clone(),
                };

                steps.push(create)
            }
        }

        steps
    }

    fn drop_indexes(&self) -> Vec<DropIndex> {
        let mut drop_indexes = Vec::new();

        for tables in self.table_pairs() {
            for index in tables.dropped_indexes() {
                // On MySQL, foreign keys automatically create indexes. These foreign-key-created
                // indexes should only be dropped as part of the foreign key.
                if self.sql_family.is_mysql() && index::index_covers_fk(&tables.previous.table, index) {
                    continue;
                }
                drop_indexes.push(DropIndex {
                    table: tables.previous.name().to_owned(),
                    name: index.name.clone(),
                })
            }
        }

        drop_indexes
    }

    fn create_enums(&self) -> Vec<CreateEnum> {
        self.created_enums()
            .map(|r#enum| CreateEnum {
                name: r#enum.name.clone(),
                variants: r#enum.values.clone(),
            })
            .collect()
    }

    fn drop_enums(&self) -> Vec<DropEnum> {
        self.dropped_enums()
            .map(|r#enum| DropEnum {
                name: r#enum.name.clone(),
            })
            .collect()
    }

    fn alter_enums(&self) -> Vec<AlterEnum> {
        self.enum_pairs()
            .filter_map(|differ| {
                let step = AlterEnum {
                    created_variants: differ.created_values().map(String::from).collect(),
                    dropped_variants: differ.dropped_values().map(String::from).collect(),
                    name: differ.previous.name.clone(),
                };

                if step.is_empty() {
                    None
                } else {
                    Some(step)
                }
            })
            .collect()
    }

    /// An iterator over the tables that are present in both schemas.
    fn table_pairs<'a>(&'a self) -> impl Iterator<Item = TableDiffer<'schema>> + 'a
    where
        'schema: 'a,
    {
        self.previous.tables.iter().filter_map(move |previous_table| {
            self.next
                .tables
                .iter()
                .find(move |next_table| tables_match(previous_table, next_table))
                .map(move |next_table| TableDiffer {
                    diffing_options: &self.diffing_options,
                    previous: TableRef::new(self.previous, previous_table),
                    next: TableRef::new(self.next, next_table),
                })
        })
    }

    fn alter_indexes(&self) -> Vec<AlterIndex> {
        let mut alter_indexes = Vec::new();
        self.table_pairs().for_each(|differ| {
            differ.index_pairs().for_each(|(previous_index, renamed_index)| {
                alter_indexes.push(AlterIndex {
                    index_name: previous_index.name.clone(),
                    index_new_name: renamed_index.name.clone(),
                    table: differ.next.name().to_owned(),
                })
            })
        });

        alter_indexes
    }

    fn created_tables<'a>(&'a self) -> impl Iterator<Item = &'a Table> + 'a {
        self.next_tables()
            .filter(move |next_table| !self.previous.has_table(&next_table.name))
    }

    fn dropped_tables(&self) -> impl Iterator<Item = &Table> {
        self.previous_tables().filter(move |previous_table| {
            !self
                .next_tables()
                .any(|next_table| tables_match(previous_table, next_table))
        })
    }

    fn previous_tables(&self) -> impl Iterator<Item = &Table> {
        self.previous
            .tables
            .iter()
            .filter(move |table| !self.table_is_ignored(&table.name))
    }

    fn next_tables(&self) -> impl Iterator<Item = &Table> {
        self.next
            .tables
            .iter()
            .filter(move |table| !self.table_is_ignored(&table.name))
    }

    fn table_is_ignored(&self, table_name: &str) -> bool {
        table_name == MIGRATION_TABLE_NAME || self.diffing_options.ignore_tables.is_match(&table_name)
    }

    fn enum_pairs(&self) -> impl Iterator<Item = EnumDiffer<'_>> {
        self.previous_enums().filter_map(move |previous| {
            self.next_enums()
                .find(|next| enums_match(previous, next))
                .map(|next| EnumDiffer { previous, next })
        })
    }

    fn created_enums(&self) -> impl Iterator<Item = &Enum> {
        self.next_enums()
            .filter(move |next| !self.previous_enums().any(|previous| enums_match(previous, next)))
    }

    fn dropped_enums(&self) -> impl Iterator<Item = &Enum> {
        self.previous_enums()
            .filter(move |previous| !self.next_enums().any(|next| enums_match(previous, next)))
    }

    fn previous_enums(&self) -> impl Iterator<Item = &Enum> {
        self.previous.enums.iter()
    }

    fn next_enums(&self) -> impl Iterator<Item = &Enum> {
        self.next.enums.iter()
    }
}

fn push_created_foreign_keys<'a, 'schema>(
    added_foreign_keys: &mut Vec<AddForeignKey>,
    table_pairs: impl Iterator<Item = TableDiffer<'schema>>,
) {
    table_pairs.for_each(|differ| {
        added_foreign_keys.extend(differ.created_foreign_keys().map(|created_fk| AddForeignKey {
            table: differ.next.name().to_owned(),
            foreign_key: created_fk.inner().clone(),
        }))
    })
}

fn push_foreign_keys_from_created_tables<'a>(
    steps: &mut Vec<AddForeignKey>,
    created_tables: impl Iterator<Item = &'a Table>,
) {
    for table in created_tables {
        steps.extend(table.foreign_keys.iter().map(|fk| AddForeignKey {
            table: table.name.clone(),
            foreign_key: fk.clone(),
        }));
    }
}

/// Compare two [ForeignKey](/sql-schema-describer/struct.ForeignKey.html)s and return whether they
/// should be considered equivalent for schema diffing purposes.
fn foreign_keys_match(previous: &ForeignKeyRef<'_, '_>, next: &ForeignKeyRef<'_, '_>) -> bool {
    // Foreign keys point to different tables.
    if previous.referenced_table().name() != next.referenced_table().name() {
        return false;
    }

    // Foreign keys point to different columns.
    if previous.referenced_columns_count() != next.referenced_columns_count() {
        return false;
    }

    // Foreign keys constrain different columns.
    if previous.constrained_columns().count() != next.constrained_columns().count() {
        return false;
    }

    // Foreign keys constrain the same columns in a different order, or their types changed.
    for (previous_column, next_column) in previous.constrained_columns().zip(next.constrained_columns()) {
        if previous_column.name() != next_column.name()
            || previous_column.column_type_family() != next_column.column_type_family()
        {
            return false;
        }
    }

    true
}

fn tables_match(previous: &Table, next: &Table) -> bool {
    previous.name == next.name
}

fn enums_match(previous: &Enum, next: &Enum) -> bool {
    previous.name == next.name
}

static POSTGRES_IGNORED_TABLES: Lazy<RegexSet> = Lazy::new(|| {
    RegexSet::new(&[
        // PostGIS. Reference: https://postgis.net/docs/manual-1.4/ch04.html#id418599
        "(?i)^spatial_ref_sys$",
        "(?i)^geometry_columns$",
    ])
    .unwrap()
});

static EMPTY_REGEXSET: Lazy<RegexSet> = Lazy::new(|| RegexSet::new::<_, &&str>(&[]).unwrap());
