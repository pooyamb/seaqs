use std::fmt;

use serde::Deserialize;

#[derive(Default, Deserialize, Debug, PartialEq)]
pub struct QueryFilter<T = ()> {
    pub start: Option<i32>,
    pub end: Option<i32>,
    pub sort: Option<String>,
    pub order: Option<String>,

    pub filter: Option<T>,
}

impl<T> QueryFilter<T>
where
    T: Filter,
{
    pub fn get_offset(&self) -> i32 {
        // Check if start is more than 0
        if let Some(offset) = self.start {
            offset
        } else {
            0
        }
    }

    pub fn get_limit(&self, offset: i32) -> i32 {
        if let Some(end) = self.end {
            std::cmp::max(end - offset, 1)
        } else {
            10
        }
    }

    pub fn get_sort(&self) -> Option<&'static str> {
        if let Some(ref field) = self.sort {
            return T::validate_sortable_field(field);
        } else {
            None
        }
    }

    pub fn get_order(&self) -> Order {
        if let Some(ref val) = self.order {
            val.parse().unwrap_or(Order::None)
        } else {
            Order::None
        }
    }

    pub fn get_filter(&self) -> Option<&T> {
        self.filter.as_ref()
    }
}

pub trait Filter {
    const SORTABLE_FIELDS: &'static [&'static str];

    fn validate_sortable_field(field: &str) -> Option<&'static str> {
        for f in Self::SORTABLE_FIELDS.iter() {
            if f == &field {
                return Some(f);
            }
        }
        if Self::SORTABLE_FIELDS.iter().any(|f| f == &field) {}
        None
    }

    fn get_max_limit() -> i32 {
        100
    }
}

pub enum Order {
    Asc,
    Desc,
    None,
}

impl Order {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Desc => "DESC",
            _ => "ASC",
        }
    }
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Order {
    type Err = ();
    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val.to_ascii_uppercase().as_str() {
            "ASC" => Ok(Order::Asc),
            "DESC" => Ok(Order::Desc),
            _ => Err(()),
        }
    }
}

#[cfg(feature = "seaq")]
mod seaq {
    use super::Order;

    impl Order {
        pub fn to_seaquery(&self) -> sea_query::Order {
            match self {
                Self::Desc => sea_query::Order::Desc,
                _ => sea_query::Order::Asc,
            }
        }
    }
}
