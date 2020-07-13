use crate::{error::*, AliasedCondition, SqlRow, ToSqlRow};
use async_trait::async_trait;
use connector_interface::{filter::Filter, RecordFilter};
use datamodel::FieldArity;
use futures::future::FutureExt;
use prisma_models::*;
use quaint::{
    ast::*,
    connector::{self, Queryable},
    pooled::PooledConnection,
};

use serde_json::{Map, Value};
use std::{convert::TryFrom, panic::AssertUnwindSafe};

impl<'t> QueryExt for connector::Transaction<'t> {}
impl QueryExt for PooledConnection {}

/// An extension trait for Quaint's `Queryable`, offering certain Prisma-centric
/// database operations on top of `Queryable`.
#[async_trait]
pub trait QueryExt: Queryable + Send + Sync {
    /// Filter and map the resulting types with the given identifiers.
    async fn filter(&self, q: Query<'_>, idents: &[(TypeIdentifier, FieldArity)]) -> crate::Result<Vec<SqlRow>> {
        let result_set = self.query(q).await?;
        let mut sql_rows = Vec::new();

        for row in result_set {
            sql_rows.push(row.to_sql_row(idents)?);
        }

        Ok(sql_rows)
    }

    /// Execute a singular SQL query in the database, returning an arbitrary
    /// JSON `Value` as a result.
    async fn raw_json<'a>(
        &'a self,
        q: String,
        params: Vec<PrismaValue>,
    ) -> std::result::Result<Value, crate::error::RawError> {
        let params: Vec<_> = params.into_iter().map(quaint::ast::Value::from).collect();
        let result_set = AssertUnwindSafe(self.query_raw(&q, params)).catch_unwind().await??;

        let columns: Vec<String> = result_set.columns().into_iter().map(ToString::to_string).collect();
        let mut result = Vec::new();

        for row in result_set.into_iter() {
            let mut object = Map::new();

            for (idx, p_value) in row.into_iter().enumerate() {
                let column_name: String = columns[idx].clone();
                object.insert(column_name, Value::from(p_value));
            }

            result.push(Value::Object(object));
        }

        Ok(Value::Array(result))
    }

    /// Execute a singular SQL query in the database, returning the number of
    /// affected rows.
    async fn raw_count<'a>(
        &'a self,
        q: String,
        params: Vec<PrismaValue>,
    ) -> std::result::Result<usize, crate::error::RawError> {
        let params: Vec<_> = params.into_iter().map(quaint::ast::Value::from).collect();
        let changes = AssertUnwindSafe(self.execute_raw(&q, params)).catch_unwind().await??;

        Ok(changes as usize)
    }

    /// Select one row from the database.
    async fn find(&self, q: Select<'_>, idents: &[(TypeIdentifier, FieldArity)]) -> crate::Result<SqlRow> {
        self.filter(q.limit(1).into(), idents)
            .await?
            .into_iter()
            .next()
            .ok_or(SqlError::RecordDoesNotExist)
    }

    /// Read the first column from the first row as an integer.
    async fn find_int(&self, q: Select<'_>) -> crate::Result<i64> {
        // UNWRAP: A dataset will always have at least one column, even if it contains no data.
        let id = self
            .find(q, &[(TypeIdentifier::Int, FieldArity::Required)])
            .await?
            .values
            .into_iter()
            .next()
            .unwrap();

        Ok(i64::try_from(id).map_err(|err| {
            let domain_error: DomainError = err.into();
            domain_error
        })?)
    }

    /// Process the record filter and either return directly with precomputed values,
    /// or fetch IDs from the database.
    async fn filter_selectors(
        &self,
        model: &ModelRef,
        record_filter: RecordFilter,
    ) -> crate::Result<Vec<RecordProjection>> {
        if let Some(selectors) = record_filter.selectors {
            Ok(selectors)
        } else {
            self.filter_ids(model, record_filter.filter).await
        }
    }

    /// Read the all columns as a (primary) identifier.
    async fn filter_ids(&self, model: &ModelRef, filter: Filter) -> crate::Result<Vec<RecordProjection>> {
        let model_id = model.primary_identifier();
        let id_cols: Vec<Column<'static>> = model_id.as_columns().collect();

        let select = Select::from_table(model.as_table())
            .columns(id_cols)
            .so_that(filter.aliased_cond(None));

        self.select_ids(select, model_id).await
    }

    async fn select_ids(&self, select: Select<'_>, model_id: ModelProjection) -> crate::Result<Vec<RecordProjection>> {
        let idents: Vec<_> = model_id
            .fields()
            .into_iter()
            .flat_map(|f| match f {
                Field::Scalar(sf) => vec![sf.type_identifier_with_arity()],
                Field::Relation(rf) => rf.type_identifiers_with_arities(),
            })
            .collect();

        let mut rows = self.filter(select.into(), &idents).await?;
        let mut result = Vec::new();

        for row in rows.drain(0..) {
            let tuples: Vec<_> = model_id.scalar_fields().zip(row.values.into_iter()).collect();
            let record_id: RecordProjection = RecordProjection::new(tuples);

            result.push(record_id);
        }

        Ok(result)
    }
}
