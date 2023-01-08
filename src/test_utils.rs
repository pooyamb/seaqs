use sea_query::{tests_cfg::*, PostgresQueryBuilder, Query};

use crate::seaq::ToFieldCond;

pub(crate) fn check_query(filter: impl ToFieldCond, result: &'static str) {
    let cond = filter.to_cond(Glyph::Aspect).unwrap();

    let query = Query::select()
        .column(Glyph::Image)
        .from(Glyph::Table)
        .cond_where(cond)
        .to_owned();

    assert_eq!(query.to_string(PostgresQueryBuilder), result);
}
