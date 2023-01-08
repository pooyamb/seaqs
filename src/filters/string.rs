use std::borrow::Cow;

use serde::Deserialize;
use serde_with::EnumMap;

#[cfg_attr(test, derive(Eq, PartialEq, Ord, PartialOrd))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StringFilter<'a> {
    Contains(Cow<'a, str>),
    NotContains(Cow<'a, str>),
    StartsWith(Cow<'a, str>),
    EndsWith(Cow<'a, str>),
}

#[cfg_attr(test, derive(PartialEq))]
#[serde_with::serde_as]
#[derive(Debug, Default, Deserialize)]
pub struct StringFilterSet<'a>(#[serde_as(as = "EnumMap")] pub(crate) Vec<StringFilter<'a>>);

impl<'a> StringFilterSet<'a> {
    pub fn push(&mut self, value: StringFilter<'a>) {
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

    use super::StringFilter::*;
    use crate::filters::StringFilterSet;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Sample<'a> {
        key: StringFilterSet<'a>,
        bla: StringFilterSet<'a>,
    }

    #[test]
    fn deserialize() {
        const QUERY: &'static str = "key[contains]=right_there\
                                    &key[notcontains]=not_there\
                                    &bla[startswith]=hello_there\
                                    &bla[endswith]=bye";

        let res = from_str::<Sample>(QUERY, ParseMode::Brackets).unwrap();

        let mut key = StringFilterSet::default();
        key.push(Contains("right_there".into()));
        key.push(NotContains("not_there".into()));

        assert_eq!(
            BTreeSet::from_iter(res.key.0.iter()),
            BTreeSet::from_iter(key.0.iter())
        );

        let mut bla = StringFilterSet::default();
        bla.push(StartsWith("hello_there".into()));
        bla.push(EndsWith("bye".into()));

        assert_eq!(
            BTreeSet::from_iter(res.bla.0.iter()),
            BTreeSet::from_iter(bla.0.iter())
        )
    }
}

#[cfg(feature = "seaq")]
mod seaq {
    use sea_query::{Cond, Expr, IntoColumnRef, IntoCondition};

    use super::{StringFilter, StringFilterSet};
    use crate::seaq::ToFieldCond;

    impl<'a> ToFieldCond for StringFilter<'a> {
        fn to_cond<I: IntoColumnRef>(&self, iden: I) -> Option<Cond> {
            Some(match self {
                StringFilter::Contains(val) => {
                    let value = ["%", &val, "%"].join("");
                    Expr::col(iden).like(value).into_condition()
                }
                StringFilter::NotContains(val) => {
                    let value = ["%", &val, "%"].join("");
                    Expr::col(iden).not_like(value).into_condition()
                }
                StringFilter::StartsWith(val) => {
                    let value = [&val, "%"].join("");
                    Expr::col(iden).like(value).into_condition()
                }
                StringFilter::EndsWith(val) => {
                    let value = ["%", &val].join("");
                    Expr::col(iden).like(value).into_condition()
                }
            })
        }
    }

    impl<'a> ToFieldCond for StringFilterSet<'a> {
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
        use super::StringFilter::*;
        use crate::{filters::StringFilterSet, test_utils::check_query};

        #[test]
        fn test_contains() {
            check_query(
                Contains("string".into()),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" LIKE '%string%'"#,
            );
        }

        #[test]
        fn test_not_contains() {
            check_query(
                NotContains("string".into()),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" NOT LIKE '%string%'"#,
            );
        }

        #[test]
        fn test_startswith() {
            check_query(
                StartsWith("string".into()),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" LIKE 'string%'"#,
            );
        }

        #[test]
        fn test_endswith() {
            check_query(
                EndsWith("string".into()),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" LIKE '%string'"#,
            );
        }

        #[test]
        fn test_set() {
            let mut set = StringFilterSet::default();
            set.push(Contains("string".into()));
            set.push(StartsWith("string".into()));

            check_query(
                set,
                r#"SELECT "image" FROM "glyph" WHERE "aspect" LIKE '%string%' AND "aspect" LIKE 'string%'"#,
            );
        }
    }
}
