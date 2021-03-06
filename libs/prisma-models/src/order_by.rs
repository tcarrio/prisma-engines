use crate::{ModelRef, ScalarFieldRef};
use std::string::ToString;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrderBy {
    pub field: ScalarFieldRef,
    pub sort_order: SortOrder,
}

pub trait IntoOrderBy {
    fn into_order_by(self, model: ModelRef) -> OrderBy;
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl ToString for SortOrder {
    fn to_string(&self) -> String {
        match self {
            SortOrder::Ascending => String::from("ASC"),
            SortOrder::Descending => String::from("DESC"),
        }
    }
}
