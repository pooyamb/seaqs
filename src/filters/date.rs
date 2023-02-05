use chrono::NaiveDate;
use serde::Deserialize;
use serde_with::EnumMap;

#[cfg_attr(test, derive(Eq, PartialEq, Ord, PartialOrd))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DateFilter {
    Before(NaiveDate),
    After(NaiveDate),
    #[serde(rename = "eq")]
    Equals(NaiveDate),
    #[serde(rename = "neq")]
    NotEquals(NaiveDate),
}

#[cfg_attr(test, derive(PartialEq))]
#[serde_with::serde_as]
#[derive(Debug, Deserialize, Default)]
pub struct DateFilterSet(#[serde_as(as = "EnumMap")] pub(crate) Vec<DateFilter>);

impl DateFilterSet {
    pub fn push(&mut self, value: DateFilter) {
        self.0.push(value);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(feature = "openapi")]
impl<'__s> utoipa::ToSchema<'__s> for DateFilterSet {
    fn schema() -> (
        &'__s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        (
            "DateFilterSet",
            utoipa::openapi::schema::ArrayBuilder::new()
                .items(DateFilter::schema().1)
                .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::iter::FromIterator;

    use chrono::NaiveDate;
    use serde::Deserialize;
    use serde_querystring::de::ParseMode;

    use super::DateFilter::*;
    use crate::filters::DateFilterSet;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Sample {
        birthday: DateFilterSet,
        register_date: DateFilterSet,
    }

    #[test]
    fn deserialize() {
        const QUERY: &'static str = "birthday[before]=1993-10-15\
                                    &birthday[eq]=1993-2-28\
                                    &register_date[after]=2022-10-15\
                                    &register_date[neq]=2022-10-15";

        let res = serde_querystring::from_str::<Sample>(QUERY, ParseMode::Brackets).unwrap();

        let mut birthday = DateFilterSet::default();
        birthday.push(Before(NaiveDate::from_ymd_opt(1993, 10, 15).unwrap()));
        birthday.push(Equals(NaiveDate::from_ymd_opt(1993, 2, 28).unwrap()));

        assert_eq!(
            BTreeSet::from_iter(res.birthday.0.iter()),
            BTreeSet::from_iter(birthday.0.iter())
        );

        let mut register_date = DateFilterSet::default();
        register_date.push(After(NaiveDate::from_ymd_opt(2022, 10, 15).unwrap()));
        register_date.push(NotEquals(NaiveDate::from_ymd_opt(2022, 10, 15).unwrap()));

        assert_eq!(
            BTreeSet::from_iter(res.register_date.0.iter()),
            BTreeSet::from_iter(register_date.0.iter())
        );
    }
}

#[cfg(feature = "seaq")]
mod seaq {
    use sea_query::{Cond, Expr, IntoColumnRef, IntoCondition};

    use crate::seaq::ToFieldCond;

    use super::{DateFilter, DateFilterSet};

    impl ToFieldCond for DateFilter {
        fn to_cond<I: IntoColumnRef>(&self, iden: I) -> Option<Cond> {
            Some(match self {
                DateFilter::Equals(val) => Expr::col(iden).eq(*val).into_condition(),
                DateFilter::NotEquals(val) => Expr::col(iden).ne(*val).into_condition(),
                DateFilter::Before(val) => Expr::col(iden).lt(*val).into_condition(),
                DateFilter::After(val) => Expr::col(iden).gte(*val).into_condition(),
            })
        }
    }

    impl ToFieldCond for DateFilterSet {
        fn to_cond<I: IntoColumnRef>(&self, iden: I) -> Option<Cond> {
            let mut conds = Cond::all();
            let col_ref = iden.into_column_ref();
            for filter in self.0.iter() {
                if let Some(filter) = filter.to_cond(col_ref.clone()) {
                    conds = conds.add(filter);
                }
            }
            Some(conds)
        }
    }

    #[cfg(test)]
    mod tests {
        use chrono::NaiveDate;

        use super::DateFilter::*;
        use crate::{filters::DateFilterSet, test_utils::check_query};

        #[test]
        fn test_before() {
            check_query(
                Before(NaiveDate::from_ymd_opt(2022, 10, 15).unwrap()),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" < '2022-10-15'"#,
            );
        }

        #[test]
        fn test_after() {
            check_query(
                After(NaiveDate::from_ymd_opt(2022, 10, 15).unwrap()),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" >= '2022-10-15'"#,
            );
        }

        #[test]
        fn test_eq() {
            check_query(
                Equals(NaiveDate::from_ymd_opt(2022, 10, 15).unwrap()),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" = '2022-10-15'"#,
            );
        }

        #[test]
        fn test_neq() {
            check_query(
                NotEquals(NaiveDate::from_ymd_opt(2022, 10, 15).unwrap()),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" <> '2022-10-15'"#,
            );
        }

        #[test]
        fn test_set() {
            let mut set = DateFilterSet::default();
            set.push(After(NaiveDate::from_ymd_opt(2022, 10, 15).unwrap()));
            set.push(Before(NaiveDate::from_ymd_opt(2022, 10, 15).unwrap()));

            check_query(
                set,
                r#"SELECT "image" FROM "glyph" WHERE "aspect" >= '2022-10-15' AND "aspect" < '2022-10-15'"#,
            );
        }
    }
}
