# seaqs
A mini tool to convert a querystring into seaquery's condition.


## Description
Let's say we have a user table and we want to provide a rest endpoint for some admin panel.
With this crate we can define a filter struct and use it with sea_query(or sea_orm, sqlx should work too).

```rust
use serde::Deserialize;
use sea_query::{Iden, Cond, Query, PostgresQueryBuilder};
use seaqs::{ApplyConds, ToCond, ToFieldCond, filters::*};
use serde_querystring::{from_str, de::ParseMode};

// It's part of the sea_query definition of a table.
#[derive(Iden)]
enum User {
    Table,
    Id,
    Name,
    Age,
    Birthday,
    CreatedAt
}

// And we define a filter struct like below
#[derive(Deserialize)]
struct UserFilters<'a> {
    id: Option<UuidFilterSet>,
    name: Option<StringFilterSet<'a>>,
    age: Option<NumberFilterSet>,
    birthday: Option<DateFilterSet>,
    created_at: Option<DateTimeFilterSet>,
}

// Then we should impl the 'ToCond' trait, which should be done using a macro but there isn't one yet.
impl<'a> ToCond for UserFilters<'a> {
    fn to_cond(&self) -> Cond {
        let mut cond = Cond::all();
        if let Some(id) = self.id.to_cond(User::Id) {
            cond = cond.add(id)
        }
        if let Some(name) = self.name.to_cond(User::Name) {
            cond = cond.add(name)
        }
        if let Some(age) = self.age.to_cond(User::Age) {
            cond = cond.add(age)
        }
        if let Some(birthday) = self.birthday.to_cond(User::Birthday) {
            cond = cond.add(birthday)
        }
        if let Some(created_at) = self.created_at.to_cond(User::CreatedAt) {
            cond = cond.add(created_at)
        }
        cond
    }
}

// I'm using serde_querystring here, but serde_json works too(whatever works with serde_with, works here)
let filters = from_str::<UserFilters>(
    "age[lt]=50&age[gte]=20&name[contains]=John",
    ParseMode::Brackets,
)
.unwrap();

// And create your query normally
let q = Query::select()
    .column(User::Name)
    .from(User::Table)
    // Just use ApplyConds trait from seaqs
    .apply_conds(&filters)
    // You shouldn't use to_string, it's just here for the test
    .to_string(PostgresQueryBuilder);

assert_eq!(
    q,
    r#"SELECT "name" FROM "user" WHERE "name" LIKE '%John%' AND ("age" >= 20 AND "age" < 50)"#
);

// You can also use the provided QueryFilter to add sort/order/page/limit to your query. It's designed to work well with react-admin or similar admin panels.

use seaqs::{ApplyFilters, QueryFilter, Filter};

// You need to impl Filter for it to work
impl<'a> Filter for UserFilters<'a> {
    const SORTABLE_FIELDS: &'static [&'static str] = &["name", "age", "created_at"];

    fn get_max_limit() -> i32 {
        100
    }
}

// Notice that we need to use the `filter` key now.
let filters = from_str::<QueryFilter<UserFilters>>(
    "filter[age][lt]=50&filter[age][gte]=20&filter[name][contains]=John&start=10&end=100&sort=age&order=DESC",
    ParseMode::Brackets,
)
.unwrap();

// And create your query normally
let q = Query::select()
    .column(User::Name)
    .from(User::Table)
    // Just use ApplyFilters trait from seaqs
    .apply_filters(&filters)
    // You shouldn't use to_string, it's just here for the test
    .to_string(PostgresQueryBuilder);

assert_eq!(
    q,
    r#"SELECT "name" FROM "user" WHERE "name" LIKE '%John%' AND ("age" >= 20 AND "age" < 50) ORDER BY "age" DESC LIMIT 90 OFFSET 10"#
)
```
