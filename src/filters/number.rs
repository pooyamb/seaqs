use serde::Deserialize;
use serde_with::EnumMap;

#[cfg_attr(test, derive(Eq, PartialEq, Ord, PartialOrd))]
#[derive(Debug, Deserialize)]
pub enum NumberFilter {
    #[serde(rename = "eq")]
    Equals(i64),
    #[serde(rename = "neq")]
    NotEquals(i64),
    #[serde(rename = "lt")]
    LesserThan(i64),
    #[serde(rename = "lte")]
    LesserThanEqual(i64),
    #[serde(rename = "gt")]
    GreaterThan(i64),
    #[serde(rename = "gte")]
    GreaterThanEqual(i64),
}

#[cfg_attr(test, derive(PartialEq))]
#[serde_with::serde_as]
#[derive(Debug, Deserialize, Default)]
pub struct NumberFilterSet(#[serde_as(as = "EnumMap")] pub(crate) Vec<NumberFilter>);

impl NumberFilterSet {
    pub fn push(&mut self, value: NumberFilter) {
        self.0.push(value);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::iter::FromIterator;

    use serde::Deserialize;
    use serde_querystring::de::{from_str, ParseMode};

    use super::{NumberFilter::*, NumberFilterSet};

    #[derive(Debug, Deserialize, PartialEq)]
    struct Sample {
        key: NumberFilterSet,
        bar: NumberFilterSet,
    }

    #[test]
    fn deserialize() {
        const QUERY: &'static str = "key[lt]=100\
                                    &key[gt]=50\
                                    &key[eq]=75\
                                    &bar[lte]=200\
                                    &bar[gte]=100\
                                    &bar[neq]=150";

        let res = from_str::<Sample>(QUERY, ParseMode::Brackets).unwrap();

        let mut key = NumberFilterSet::default();
        key.push(LesserThan(100));
        key.push(GreaterThan(50));
        key.push(Equals(75));

        assert_eq!(
            BTreeSet::from_iter(res.key.0.iter()),
            BTreeSet::from_iter(key.0.iter())
        );

        let mut bar = NumberFilterSet::default();
        bar.push(LesserThanEqual(200));
        bar.push(GreaterThanEqual(100));
        bar.push(NotEquals(150));

        assert_eq!(
            BTreeSet::from_iter(res.bar.0.iter()),
            BTreeSet::from_iter(bar.0.iter())
        )
    }
}

#[cfg(feature = "seaq")]
mod seaq {
    use sea_query::{Cond, Expr, IntoColumnRef, IntoCondition};

    use crate::seaq::ToFieldCond;

    use super::{NumberFilter, NumberFilterSet};

    impl ToFieldCond for NumberFilter {
        fn to_cond<I: IntoColumnRef>(&self, iden: I) -> Option<Cond> {
            Some(match self {
                NumberFilter::Equals(val) => Expr::col(iden).eq(*val).into_condition(),
                NumberFilter::NotEquals(val) => Expr::col(iden).ne(*val).into_condition(),
                NumberFilter::GreaterThan(val) => Expr::col(iden).gt(*val).into_condition(),
                NumberFilter::GreaterThanEqual(val) => Expr::col(iden).gte(*val).into_condition(),
                NumberFilter::LesserThan(val) => Expr::col(iden).lt(*val).into_condition(),
                NumberFilter::LesserThanEqual(val) => Expr::col(iden).lte(*val).into_condition(),
            })
        }
    }

    impl ToFieldCond for NumberFilterSet {
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
        use super::NumberFilter::*;
        use crate::{filters::NumberFilterSet, test_utils::check_query};

        #[test]
        fn test_lt() {
            check_query(
                LesserThan(100),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" < 100"#,
            );
        }

        #[test]
        fn test_lte() {
            check_query(
                LesserThanEqual(100),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" <= 100"#,
            );
        }

        #[test]
        fn test_gt() {
            check_query(
                GreaterThan(100),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" > 100"#,
            );
        }

        #[test]
        fn test_gte() {
            check_query(
                GreaterThanEqual(100),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" >= 100"#,
            );
        }

        #[test]
        fn test_eq() {
            check_query(
                Equals(100),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" = 100"#,
            );
        }

        #[test]
        fn test_neq() {
            check_query(
                NotEquals(100),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" <> 100"#,
            );
        }

        #[test]
        fn test_set() {
            let mut set = NumberFilterSet::default();
            set.push(LesserThan(120));
            set.push(GreaterThanEqual(140));

            check_query(
                set,
                r#"SELECT "image" FROM "glyph" WHERE "aspect" < 120 AND "aspect" >= 140"#,
            );
        }
    }
}
