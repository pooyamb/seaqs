use serde::Deserialize;
use serde_with::EnumMap;
use uuid::Uuid;

#[cfg_attr(test, derive(Eq, PartialEq, Ord, PartialOrd))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Deserialize)]
pub enum UuidFilter {
    #[serde(rename = "eq")]
    Equals(Uuid),
    #[serde(rename = "in")]
    In(Vec<Uuid>),
}

#[cfg_attr(test, derive(PartialEq))]
#[serde_with::serde_as]
#[derive(Debug, Deserialize, Default)]
pub struct UuidFilterSet(#[serde_as(as = "EnumMap")] pub(crate) Vec<UuidFilter>);

impl UuidFilterSet {
    pub fn push(&mut self, value: UuidFilter) {
        self.0.push(value);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(feature = "openapi")]
impl<'__s> utoipa::ToSchema<'__s> for UuidFilterSet {
    fn schema() -> (
        &'__s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        (
            "UuidFilterSet",
            utoipa::openapi::schema::ArrayBuilder::new()
                .items(UuidFilter::schema().1)
                .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::iter::FromIterator;

    use serde::Deserialize;
    use serde_querystring::de::{from_str, ParseMode};
    use uuid::uuid;

    use super::UuidFilter::*;
    use super::UuidFilterSet;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Sample {
        id: UuidFilterSet,
    }

    #[test]
    fn deserialize() {
        const QUERY: &'static str = "id[in]=23191e01-8af8-4381-848c-f9387116d132\
                                    &id[in]=23191e01-8af8-4381-848c-f9387116d132\
                                    &id[eq]=23191e01-8af8-4381-848c-f9387116d132";

        let res = from_str::<Sample>(QUERY, ParseMode::Brackets).unwrap();

        let mut id = UuidFilterSet::default();
        id.push(In(vec![
            uuid!("23191e01-8af8-4381-848c-f9387116d132"),
            uuid!("23191e01-8af8-4381-848c-f9387116d132"),
        ]));
        id.push(Equals(uuid!("23191e01-8af8-4381-848c-f9387116d132")));

        assert_eq!(
            BTreeSet::from_iter(res.id.0.iter()),
            BTreeSet::from_iter(id.0.iter())
        );
    }
}

#[cfg(feature = "seaq")]
mod seaq {
    use sea_query::{Cond, Expr, IntoColumnRef, IntoCondition};

    use super::{UuidFilter, UuidFilterSet};
    use crate::seaq::ToFieldCond;

    impl ToFieldCond for UuidFilter {
        fn to_cond<I: IntoColumnRef>(&self, iden: I) -> Option<Cond> {
            Some(match self {
                UuidFilter::Equals(val) => Expr::col(iden).eq(*val).into_condition(),
                UuidFilter::In(val) => Expr::col(iden).is_in(val.iter().copied()).into_condition(),
            })
        }
    }

    impl ToFieldCond for UuidFilterSet {
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
        use uuid::uuid;

        use super::UuidFilter::*;
        use crate::{filters::UuidFilterSet, test_utils::check_query};

        #[test]
        fn test_eq() {
            check_query(
                Equals(uuid!("urn:uuid:F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4")),
                r#"SELECT "image" FROM "glyph" WHERE "aspect" = 'f9168c5e-ceb2-4faa-b6bf-329bf39fa1e4'"#,
            );
        }

        #[test]
        fn test_in() {
            check_query(
                In(vec![
                    uuid!("urn:uuid:F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4"),
                    uuid!("00000000-0000-0000-0000-ffff00000001"),
                    uuid!("00000000-0000-0000-0000-ffff00000002"),
                ]),
                "SELECT \"image\" FROM \"glyph\" WHERE \"aspect\" IN (\
                    'f9168c5e-ceb2-4faa-b6bf-329bf39fa1e4', \
                    '00000000-0000-0000-0000-ffff00000001', \
                    '00000000-0000-0000-0000-ffff00000002')",
            );
        }

        #[test]
        fn test_set() {
            let mut set = UuidFilterSet::default();
            set.push(In(vec![uuid!(
                "urn:uuid:F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4"
            )]));
            set.push(Equals(uuid!("00000000-0000-0000-0000-ffff00000002")));

            check_query(
                set,
                "SELECT \"image\" FROM \"glyph\" WHERE \
                \"aspect\" IN ('f9168c5e-ceb2-4faa-b6bf-329bf39fa1e4') AND \
                \"aspect\" = '00000000-0000-0000-0000-ffff00000002'",
            );
        }
    }
}
