use crate::SqlError;
use datamodel::{
    Datamodel, DefaultNames, DefaultValue as DMLDef, FieldArity, FieldType, IndexDefinition, Model, OnDeleteStrategy,
    RelationField, RelationInfo, ScalarField, ScalarType, ValueGenerator as VG,
};
use sql_schema_describer::{
    Column, ColumnArity, ColumnTypeFamily, DefaultValue as SQLDef, ForeignKey, Index, IndexType, SqlSchema, Table,
};
use tracing::debug;

//checks
pub fn is_migration_table(table: &Table) -> bool {
    table.name == "_Migration"
        && table.columns.iter().any(|c| c.name == "revision")
        && table.columns.iter().any(|c| c.name == "name")
        && table.columns.iter().any(|c| c.name == "datamodel")
        && table.columns.iter().any(|c| c.name == "status")
        && table.columns.iter().any(|c| c.name == "applied")
        && table.columns.iter().any(|c| c.name == "rolled_back")
        && table.columns.iter().any(|c| c.name == "datamodel_steps")
        && table.columns.iter().any(|c| c.name == "database_migration")
        && table.columns.iter().any(|c| c.name == "errors")
        && table.columns.iter().any(|c| c.name == "started_at")
        && table.columns.iter().any(|c| c.name == "finished_at")
}

pub(crate) fn is_relay_table(table: &Table) -> bool {
    table.name == "_RelayId"
        && table.columns[0].name == "id"
        && table.columns[1].name.to_lowercase() == "stablemodelidentifier"
}

pub(crate) fn is_prisma_1_or_11_list_table(table: &Table) -> bool {
    table.columns.len() == 3
        && table.columns[0].name.to_lowercase() == "nodeid"
        && table.columns[1].name == "position"
        && table.columns[2].name == "value"
}

pub(crate) fn is_prisma_1_point_1_or_2_join_table(table: &Table) -> bool {
    table.columns.len() == 2 && table.indices.len() >= 2 && common_prisma_m_to_n_relation_conditions(table)
}

pub(crate) fn is_prisma_1_point_0_join_table(table: &Table) -> bool {
    table.columns.len() == 3
        && table.indices.len() >= 2
        && table.columns.iter().any(|c| c.name == "id")
        && common_prisma_m_to_n_relation_conditions(table)
}

fn common_prisma_m_to_n_relation_conditions(table: &Table) -> bool {
    fn is_a(column: &String) -> bool {
        column.to_lowercase() == "a"
    }

    fn is_b(column: &String) -> bool {
        column.to_lowercase() == "b"
    }

    table.name.starts_with("_")
        //UNIQUE INDEX [A,B]
        && table.indices.iter().any(|i| {
            i.columns.len() == 2
                && is_a(&i.columns[0])
                && is_b(&i.columns[1])
                && i.tpe == IndexType::Unique
        })
        //INDEX [B]
        && table
            .indices
            .iter()
            .any(|i| i.columns.len() == 1 && is_b(&i.columns[0]) && i.tpe == IndexType::Normal)
        // 2 FKs
        && table.foreign_keys.len() == 2
        // Lexicographically lower model referenced by A
        && if table.foreign_keys[0].referenced_table <= table.foreign_keys[1].referenced_table {
            is_a(&table.foreign_keys[0].columns[0]) && is_b(&table.foreign_keys[1].columns[0])
        } else {
            is_b(&table.foreign_keys[0].columns[0]) && is_a(&table.foreign_keys[1].columns[0])
        }
}

//calculators

pub fn calculate_many_to_many_field(
    foreign_key: &ForeignKey,
    relation_name: String,
    is_self_relation: bool,
) -> RelationField {
    let relation_info = RelationInfo {
        name: relation_name,
        fields: vec![],
        to: foreign_key.referenced_table.clone(),
        to_fields: foreign_key.referenced_columns.clone(),
        on_delete: OnDeleteStrategy::None,
    };

    let basename = foreign_key.referenced_table.clone();

    let name = match is_self_relation {
        true => format!("{}_{}", basename, foreign_key.columns[0]),
        false => basename,
    };

    RelationField::new(&name, FieldArity::List, relation_info)
}

pub(crate) fn calculate_index(index: &Index) -> IndexDefinition {
    debug!("Handling index  {:?}", index);
    let tpe = match index.tpe {
        IndexType::Unique => datamodel::dml::IndexType::Unique,
        IndexType::Normal => datamodel::dml::IndexType::Normal,
    };

    IndexDefinition {
        name: Some(index.name.clone()),
        fields: index.columns.clone(),
        tpe,
    }
}

pub(crate) fn calculate_scalar_field(table: &Table, column: &Column) -> ScalarField {
    debug!("Handling column {:?}", column);
    let field_type = calculate_scalar_field_type(&column);
    let (is_commented_out, documentation) = match field_type {
        FieldType::Unsupported(_) => (true, Some("This type is currently not supported.".to_string())),
        _ => (false, None),
    };

    let arity = match column.tpe.arity {
        _ if column.auto_increment && field_type == FieldType::Base(ScalarType::Int, None) => FieldArity::Required,
        ColumnArity::Required => FieldArity::Required,
        ColumnArity::Nullable => FieldArity::Optional,
        ColumnArity::List => FieldArity::List,
    };

    let is_id = is_id(&column, &table);
    let default_value = calculate_default(table, &column, &arity);
    let is_unique = table.is_column_unique(&column.name) && !is_id;

    ScalarField {
        name: column.name.clone(),
        arity,
        field_type,
        database_name: None,
        default_value,
        is_unique,
        is_id,
        documentation,
        is_generated: false,
        is_updated_at: false,
        is_commented_out,
    }
}

pub(crate) fn calculate_relation_field(
    schema: &SqlSchema,
    table: &Table,
    foreign_key: &ForeignKey,
) -> Result<RelationField, SqlError> {
    debug!("Handling foreign key  {:?}", foreign_key);

    let relation_info = RelationInfo {
        name: calculate_relation_name(schema, foreign_key, table)?,
        fields: foreign_key.columns.clone(),
        to: foreign_key.referenced_table.clone(),
        to_fields: foreign_key.referenced_columns.clone(),
        on_delete: OnDeleteStrategy::None,
    };

    let columns: Vec<&Column> = foreign_key
        .columns
        .iter()
        .map(|c| table.columns.iter().find(|tc| tc.name == *c).unwrap())
        .collect();

    let arity = match !columns.iter().any(|c| c.is_required()) {
        true => FieldArity::Optional,
        false => FieldArity::Required,
    };

    Ok(RelationField::new(&foreign_key.referenced_table, arity, relation_info))
}

pub(crate) fn calculate_backrelation_field(
    schema: &SqlSchema,
    model: &Model,
    other_model: &Model,
    relation_field: &RelationField,
    relation_info: &RelationInfo,
) -> Result<RelationField, SqlError> {
    match schema.table(&model.name) {
        Err(table_name) => Err(SqlError::SchemaInconsistent {
            explanation: format!("Table {} not found.", table_name),
        }),
        Ok(table) => {
            let new_relation_info = RelationInfo {
                name: relation_info.name.clone(),
                to: model.name.clone(),
                fields: vec![],
                to_fields: vec![],
                on_delete: OnDeleteStrategy::None,
            };

            let other_is_unique = match &relation_info.fields.len() {
                1 => {
                    let column_name = &relation_info.fields.first().unwrap();
                    table.is_column_unique(column_name)
                }
                _ => table
                    .indices
                    .iter()
                    .any(|i| columns_match(&i.columns, &relation_info.fields) && i.tpe == IndexType::Unique),
            };

            let arity = match relation_field.arity {
                FieldArity::Required | FieldArity::Optional if other_is_unique => FieldArity::Optional,
                FieldArity::Required | FieldArity::Optional => FieldArity::List,
                FieldArity::List => FieldArity::Optional,
            };

            //if the backrelation name would be duplicate, probably due to being a selfrelation
            let name = if model.name == other_model.name && relation_field.name == model.name {
                format!("other_{}", model.name.clone())
            } else {
                model.name.clone()
            };

            Ok(RelationField::new(&name, arity, new_relation_info))
        }
    }
}

pub(crate) fn calculate_default(table: &Table, column: &Column, arity: &FieldArity) -> Option<DMLDef> {
    match (&column.default, &column.tpe.family) {
        (_, _) if *arity == FieldArity::List => None,
        (_, ColumnTypeFamily::Int) if column.auto_increment => Some(DMLDef::Expression(VG::new_autoincrement())),
        (_, ColumnTypeFamily::Int) if is_sequence(column, table) => Some(DMLDef::Expression(VG::new_autoincrement())),
        (Some(SQLDef::SEQUENCE(_)), _) => Some(DMLDef::Expression(VG::new_autoincrement())),
        (Some(SQLDef::NOW), ColumnTypeFamily::DateTime) => Some(DMLDef::Expression(VG::new_now())),
        (Some(SQLDef::DBGENERATED(_)), _) => Some(DMLDef::Expression(VG::new_dbgenerated())),
        (Some(SQLDef::VALUE(val)), _) => Some(DMLDef::Single(val.clone())),
        _ => None,
    }
}

pub(crate) fn is_id(column: &Column, table: &Table) -> bool {
    table
        .primary_key
        .as_ref()
        .map(|pk| pk.is_single_primary_key(&column.name))
        .unwrap_or(false)
}

pub(crate) fn is_sequence(column: &Column, table: &Table) -> bool {
    table
        .primary_key
        .as_ref()
        .map(|pk| pk.is_single_primary_key(&column.name) && pk.sequence.is_some())
        .unwrap_or(false)
}

pub(crate) fn calculate_relation_name(schema: &SqlSchema, fk: &ForeignKey, table: &Table) -> Result<String, SqlError> {
    //this is not called for prisma many to many relations. for them the name is just the name of the join table.
    let referenced_model = &fk.referenced_table;
    let model_with_fk = &table.name;
    let fk_column_name = fk.columns.join("_");

    let fk_to_same_model: Vec<&ForeignKey> = table
        .foreign_keys
        .iter()
        .filter(|fk| &fk.referenced_table == referenced_model)
        .collect();

    match schema.table(referenced_model) {
        Err(table_name) => Err(SqlError::SchemaInconsistent {
            explanation: format!("Table {} not found.", table_name),
        }),
        Ok(other_table) => {
            let fk_from_other_model_to_this: Vec<&ForeignKey> = other_table
                .foreign_keys
                .iter()
                .filter(|fk| &fk.referenced_table == model_with_fk)
                .collect();

            let name = if fk_to_same_model.len() < 2 && fk_from_other_model_to_this.is_empty() {
                DefaultNames::name_for_unambiguous_relation(model_with_fk, referenced_model)
            } else {
                DefaultNames::name_for_ambiguous_relation(model_with_fk, referenced_model, &fk_column_name)
            };

            Ok(name)
        }
    }
}

pub(crate) fn calculate_scalar_field_type(column: &Column) -> FieldType {
    debug!("Calculating field type for '{}'", column.name);

    match &column.tpe.family {
        ColumnTypeFamily::Boolean => FieldType::Base(ScalarType::Boolean, None),
        ColumnTypeFamily::DateTime => FieldType::Base(ScalarType::DateTime, None),
        ColumnTypeFamily::Float => FieldType::Base(ScalarType::Float, None),
        ColumnTypeFamily::Int => FieldType::Base(ScalarType::Int, None),
        ColumnTypeFamily::String => FieldType::Base(ScalarType::String, None),
        ColumnTypeFamily::Enum(name) => FieldType::Enum(name.clone()),
        ColumnTypeFamily::Uuid => FieldType::Base(ScalarType::String, None),
        ColumnTypeFamily::Json => FieldType::Base(ScalarType::Json, None),
        x => FieldType::Unsupported(x.to_string()),
    }
}

// misc

pub fn deduplicate_relation_field_names(datamodel: &mut Datamodel) {
    let mut duplicated_relation_fields = vec![];

    for model in datamodel.models() {
        for field in model.relation_fields() {
            if model.fields().filter(|f| field.name == f.name()).count() > 1 {
                duplicated_relation_fields.push((
                    model.name.clone(),
                    field.name.clone(),
                    field.relation_info.name.clone(),
                ));
            }
        }
    }

    duplicated_relation_fields
        .iter()
        .for_each(|(model, field, relation_name)| {
            let mut field = datamodel.find_model_mut(model).find_relation_field_mut(field);
            //todo self vs normal relation?
            field.name = format!("{}_{}", field.name, &relation_name);
        });
}
/// Returns whether the elements of the two slices match, regardless of ordering.
pub fn columns_match(a_cols: &[String], b_cols: &[String]) -> bool {
    a_cols.len() == b_cols.len() && a_cols.iter().all(|a_col| b_cols.iter().any(|b_col| a_col == b_col))
}

pub fn replace_field_names(target: &mut Vec<String>, old_name: &str, new_name: &str) {
    target
        .iter_mut()
        .map(|v| {
            if v == old_name {
                *v = new_name.to_string()
            }
        })
        .for_each(drop);
}
