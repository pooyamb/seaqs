use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use serde_with::EnumMap;

#[cfg_attr(test, derive(Eq, PartialEq, Ord, PartialOrd))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DateTimeTzFilter {
    Before(DateTime<FixedOffset>),
    After(DateTime<FixedOffset>),
    #[serde(rename = "eq")]
    Equals(DateTime<FixedOffset>),
    #[serde(rename = "neq")]
    NotEquals(DateTime<FixedOffset>),
}

#[cfg_attr(test, derive(PartialEq))]
#[serde_with::serde_as]
#[derive(Debug, Deserialize, Default)]
pub struct DateTimeTzFilterSet(#[serde_as(as = "EnumMap")] pub(crate) Vec<DateTimeTzFilter>);

impl DateTimeTzFilterSet {
    pub fn push(&mut self, value: DateTimeTzFilter) {
        self.0.push(value);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(feature = "openapi")]
impl<'__s> utoipa::ToSchema<'__s> for DateTimeTzFilterSet {
    fn schema() -> (
        &'__s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        (
            "DateTimeTzFilterSet",
            utoipa::openapi::schema::ArrayBuilder::new()
                .items(DateTimeTzFilter::schema().1)
                .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::iter::FromIterator;

    use chrono::{DateTime, FixedOffset, NaiveDate};
    use serde::Deserialize;
    use serde_querystring::de::{from_str, ParseMode};

    use super::DateTimeTzFilter::*;
    use crate::filters::DateTimeTzFilterSet;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Sample {
        birthday: DateTimeTzFilterSet,
        register_date: DateTimeTzFilterSet,
    }

    #[test]
    fn deserialize() {
        const QUERY: &'static str = "birthday[before]=1993-10-15T10:30:5%2b00:00\
                                    &birthday[eq]=1993-2-28T10:30:05%2b00:00\
                                    &register_date[after]=2022-10-15T10:30:05%2b00:00\
                                    &register_date[neq]=2022-10-15T10:30:05%2b00:00";

        let res = from_str::<Sample>(QUERY, ParseMode::Brackets).unwrap();

        let mut birthday = DateTimeTzFilterSet::default();
        birthday.push(Before(DateTime::from_local(
            NaiveDate::from_ymd_opt(1993, 10, 15)
                .unwrap()
                .and_hms_opt(10, 30, 5)
                .unwrap(),
            FixedOffset::east_opt(0).unwrap(),
        )));
        birthday.push(Equals(DateTime::from_local(
            NaiveDate::from_ymd_opt(1993, 2, 28)
                .unwrap()
                .and_hms_opt(10, 30, 5)
                .unwrap(),
            FixedOffset::east_opt(0).unwrap(),
        )));

        assert_eq!(
            BTreeSet::from_iter(res.birthday.0.iter()),
            BTreeSet::from_iter(birthday.0.iter())
        );

        let mut register_date = DateTimeTzFilterSet::default();
        register_date.push(After(DateTime::from_local(
            NaiveDate::from_ymd_opt(2022, 10, 15)
                .unwrap()
                .and_hms_opt(10, 30, 5)
                .unwrap(),
            FixedOffset::east_opt(0).unwrap(),
        )));
        register_date.push(NotEquals(DateTime::from_local(
            NaiveDate::from_ymd_opt(2022, 10, 15)
                .unwrap()
                .and_hms_opt(10, 30, 5)
                .unwrap(),
            FixedOffset::east_opt(0).unwrap(),
        )));

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

    use super::{DateTimeTzFilter, DateTimeTzFilterSet};

    impl ToFieldCond for DateTimeTzFilter {
        fn to_cond<I: IntoColumnRef>(&self, iden: I) -> Option<Cond> {
            Some(match self {
                DateTimeTzFilter::Equals(val) => Expr::col(iden).eq(*val).into_condition(),
                DateTimeTzFilter::NotEquals(val) => Expr::col(iden).ne(*val).into_condition(),
                DateTimeTzFilter::Before(val) => Expr::col(iden).lt(*val).into_condition(),
                DateTimeTzFilter::After(val) => Expr::col(iden).gte(*val).into_condition(),
            })
        }
    }

    impl ToFieldCond for DateTimeTzFilterSet {
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
        use chrono::{DateTime, FixedOffset, NaiveDate};

        use super::DateTimeTzFilter::*;
        use crate::{filters::DateTimeTzFilterSet, test_utils::check_query};

        #[test]
        fn test_before() {
            check_query(
                Before(DateTime::from_local(
                    NaiveDate::from_ymd_opt(2022, 10, 15)
                        .unwrap()
                        .and_hms_opt(10, 30, 5)
                        .unwrap(),
                    FixedOffset::east_opt(0).unwrap(),
                )),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" < '2022-10-15 10:30:05 +00:00'"#,
            );
        }

        #[test]
        fn test_after() {
            check_query(
                After(DateTime::from_local(
                    NaiveDate::from_ymd_opt(2022, 10, 15)
                        .unwrap()
                        .and_hms_opt(10, 30, 5)
                        .unwrap(),
                    FixedOffset::east_opt(0).unwrap(),
                )),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" >= '2022-10-15 10:30:05 +00:00'"#,
            );
        }

        #[test]
        fn test_eq() {
            check_query(
                Equals(DateTime::from_local(
                    NaiveDate::from_ymd_opt(2022, 10, 15)
                        .unwrap()
                        .and_hms_opt(10, 30, 5)
                        .unwrap(),
                    FixedOffset::east_opt(0).unwrap(),
                )),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" = '2022-10-15 10:30:05 +00:00'"#,
            );
        }

        #[test]
        fn test_neq() {
            check_query(
                NotEquals(DateTime::from_local(
                    NaiveDate::from_ymd_opt(2022, 10, 15)
                        .unwrap()
                        .and_hms_opt(10, 30, 5)
                        .unwrap(),
                    FixedOffset::east_opt(0).unwrap(),
                )),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" <> '2022-10-15 10:30:05 +00:00'"#,
            );
        }

        #[test]
        fn test_set() {
            let mut set = DateTimeTzFilterSet::default();
            set.push(After(DateTime::from_local(
                NaiveDate::from_ymd_opt(2022, 10, 15)
                    .unwrap()
                    .and_hms_opt(10, 30, 5)
                    .unwrap(),
                FixedOffset::east_opt(0).unwrap(),
            )));
            set.push(Before(DateTime::from_local(
                NaiveDate::from_ymd_opt(2022, 10, 15)
                    .unwrap()
                    .and_hms_opt(10, 30, 5)
                    .unwrap(),
                FixedOffset::east_opt(0).unwrap(),
            )));

            check_query(
                set,
                "SELECT \"image\" FROM \"glyph\" WHERE \"aspect\" >= '2022-10-15 10:30:05 +00:00' AND \
                \"aspect\" < '2022-10-15 10:30:05 +00:00'",
            );
        }
    }
}
