//! A set of traits and impls for converting filters into seaquery conditions

use sea_query::{Cond, Iden, IntoColumnRef};
use sea_query::{DeleteStatement, SelectStatement};

use super::QueryFilter;
use crate::Filter;

pub trait ToFieldCond {
    fn to_cond<I: IntoColumnRef>(&self, iden: I) -> Option<Cond>;
}

impl<'b, T> ToFieldCond for Option<T>
where
    T: ToFieldCond,
{
    fn to_cond<I: IntoColumnRef>(&self, iden: I) -> Option<Cond> {
        match self {
            Some(val) => val.to_cond(iden),
            None => None,
        }
    }
}

impl ToFieldCond for () {
    fn to_cond<I: IntoColumnRef>(&self, _iden: I) -> Option<Cond> {
        None
    }
}

pub trait ToCond {
    fn to_cond(&self) -> Cond;
}

pub trait ApplyConds<T> {
    fn apply_conds(self, filters: &T) -> Self;
}

impl<T: ToCond> ApplyConds<T> for &mut SelectStatement {
    fn apply_conds(self, filters: &T) -> Self {
        let conds = filters.to_cond();
        self.cond_where(conds)
    }
}

impl<T: ToCond> ApplyConds<T> for &mut DeleteStatement {
    fn apply_conds(self, filters: &T) -> Self {
        let conds = filters.to_cond();
        self.cond_where(conds)
    }
}

pub trait ApplyFilters<T> {
    fn apply_filters(self, filters: &QueryFilter<T>) -> Self;
}

impl<T: Filter + ToCond> ApplyFilters<T> for &mut SelectStatement {
    fn apply_filters(self, filters: &QueryFilter<T>) -> Self {
        let offset = filters.get_offset();
        let limit = filters.get_limit(offset);
        let order = filters.get_order();
        let sort = filters.get_sort();

        let mut statement = self;

        if let Some(filter) = &filters.filter {
            statement = statement.apply_conds(filter);
        }

        statement = statement.offset(offset as u64).limit(limit as u64);

        if let Some(field) = sort {
            statement = statement.order_by(IntoColumnRefStr(field), order.to_seaquery())
        }

        statement
    }
}

impl<T: Filter + ToCond> ApplyFilters<T> for &mut DeleteStatement {
    fn apply_filters(self, filters: &QueryFilter<T>) -> Self {
        let limit = filters.get_limit(0);
        let order = filters.get_order();
        let sort = filters.get_sort();

        let mut statement = self;

        if let Some(filter) = &filters.filter {
            statement = statement.apply_conds(filter);
        }

        statement = statement.limit(limit as u64);

        if let Some(field) = sort {
            statement = statement.order_by(IntoColumnRefStr(field), order.to_seaquery())
        }

        statement
    }
}

#[derive(Clone)]
pub(crate) struct IntoColumnRefStr(pub &'static str);

impl Iden for IntoColumnRefStr {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        s.write_str(self.0).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use sea_query::{Cond, Iden, PostgresQueryBuilder, Query};
    use serde::Deserialize;
    use serde_querystring::de::ParseMode;

    use crate::{
        filters::{NumberFilterSet, StringFilterSet},
        seaq::{ApplyConds, ToCond, ToFieldCond},
        Filter, QueryFilter,
    };

    use super::ApplyFilters;

    #[derive(Deserialize)]
    struct MyFilters<'a> {
        name: Option<StringFilterSet<'a>>,
        age: Option<NumberFilterSet>,
        score: Option<NumberFilterSet>,
    }

    #[derive(Iden)]
    struct User;
    #[derive(Iden)]
    struct Name;
    #[derive(Iden)]
    struct Age;
    #[derive(Iden)]
    struct Score;

    impl<'a> ToCond for MyFilters<'a> {
        fn to_cond(&self) -> Cond {
            let mut cond = Cond::all();
            if let Some(name) = self.name.to_cond(Name) {
                cond = cond.add(name)
            }
            if let Some(age) = self.age.to_cond(Age) {
                cond = cond.add(age)
            }
            if let Some(score) = self.score.to_cond(Score) {
                cond = cond.add(score)
            }
            cond
        }
    }

    impl<'a> Filter for MyFilters<'a> {
        const SORTABLE_FIELDS: &'static [&'static str] = &["name", "age", "score"];

        fn get_max_limit() -> i32 {
            100
        }
    }

    #[test]
    fn test_filters() {
        let filters = serde_querystring::from_str::<MyFilters>(
            "age[lt]=50&age[gte]=20&name[contains]=John",
            ParseMode::Brackets,
        )
        .unwrap();

        let q = Query::select()
            .column(Age)
            .from(User)
            .apply_conds(&filters)
            .to_string(PostgresQueryBuilder);

        assert_eq!(
            q,
            r#"SELECT "age" FROM "user" WHERE "name" LIKE '%John%' AND ("age" >= 20 AND "age" < 50)"#
        )
    }

    #[test]
    fn test_query_filters() {
        let filters = serde_querystring::from_str::<QueryFilter<MyFilters>>(
            "filter[age][lt]=50&filter[age][gte]=20&filter[name][contains]=John&start=10&end=100&sort=age&order=DESC",
            ParseMode::Brackets,
        )
        .unwrap();

        let q = Query::select()
            .column(Age)
            .from(User)
            .apply_filters(&filters)
            .to_string(PostgresQueryBuilder);

        assert_eq!(
            q,
            "SELECT \"age\" FROM \"user\" WHERE \"name\" LIKE '%John%' AND (\"age\" >= 20 AND \"age\" < 50) \
             ORDER BY \"age\" DESC LIMIT 90 OFFSET 10"
        )
    }
}
