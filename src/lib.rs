#![doc = include_str!("../README.md")]

pub mod filters;
mod query;

#[cfg(feature = "seaq")]
#[cfg(test)]
mod test_utils;

#[cfg(feature = "seaq")]
mod seaq;

#[cfg(feature = "seaq")]
pub use seaq::{ApplyConds, ApplyFilters, ToCond, ToFieldCond};

pub use query::{Filter, Order, QueryFilter};
