use crate::run::PathDependency;
use std::path::Path;

macro_rules! test_normalize {
    ($name:ident $original:literal $expected:literal) => {
        #[test]
        fn $name() {
            let context = super::Context {
                krate: "trybuild000",
                source_dir: Path::new("/git/trybuild/test_suite"),
                workspace: Path::new("/git/trybuild"),
                path_dependencies: &[PathDependency {
                    name: String::from("diesel"),
                    normalized_path: Path::new("/home/user/documents/rust/diesel/diesel").into(),
                }],
            };
            let original = $original.to_owned().into_bytes();
            let variations = super::diagnostics(original, context);
            assert_eq!(variations.preferred(), $expected);
        }
    };
}

test_normalize! {test_basic "
error: `self` parameter is only allowed in associated functions
  --> /git/trybuild/test_suite/ui/error.rs:11:23
   |
11 | async fn bad_endpoint(self) -> Result<HttpResponseOkObject<()>, HttpError> {
   |                       ^^^^ not semantically valid as function parameter

error: aborting due to 2 previous errors

For more information about this error, try `rustc --explain E0401`.
error: could not compile `trybuild-tests`.

To learn more, run the command again with --verbose.
" "
error: `self` parameter is only allowed in associated functions
  --> $DIR/error.rs:11:23
   |
11 | async fn bad_endpoint(self) -> Result<HttpResponseOkObject<()>, HttpError> {
   |                       ^^^^ not semantically valid as function parameter
"}

test_normalize! {test_dir_backslash "
error[E0277]: the trait bound `QueryParams: serde::de::Deserialize<'de>` is not satisfied
   --> \\git\\trybuild\\test_suite\\ui\\error.rs:22:61
" "
error[E0277]: the trait bound `QueryParams: serde::de::Deserialize<'de>` is not satisfied
   --> $DIR/error.rs:22:61
"}

test_normalize! {test_rust_lib "
error[E0599]: no method named `quote_into_iter` found for struct `std::net::Ipv4Addr` in the current scope
  --> /git/trybuild/test_suite/ui/not-repeatable.rs:6:13
   |
6  |     let _ = quote! { #(#ip)* };
   |             ^^^^^^^^^^^^^^^^^^ method not found in `std::net::Ipv4Addr`
   |
  ::: /rustlib/src/rust/src/libstd/net/ip.rs:83:1
  ::: /rustlib/src/rust/library/std/src/net/ip.rs:83:1
   |
83 | pub struct Ipv4Addr {
   | -------------------
   | |
   | doesn't satisfy `std::net::Ipv4Addr: quote::to_tokens::ToTokens`
" "
error[E0599]: no method named `quote_into_iter` found for struct `std::net::Ipv4Addr` in the current scope
  --> $DIR/not-repeatable.rs:6:13
   |
6  |     let _ = quote! { #(#ip)* };
   |             ^^^^^^^^^^^^^^^^^^ method not found in `std::net::Ipv4Addr`
   |
  ::: $RUST/src/libstd/net/ip.rs
  ::: $RUST/std/src/net/ip.rs
   |
   | pub struct Ipv4Addr {
   | -------------------
   | |
   | doesn't satisfy `std::net::Ipv4Addr: quote::to_tokens::ToTokens`
"}

test_normalize! {test_type_dir_backslash "
error[E0277]: `*mut _` cannot be shared between threads safely
   --> /git/trybuild/test_suite/ui/compile-fail-3.rs:7:5
    |
7   |     thread::spawn(|| {
    |     ^^^^^^^^^^^^^ `*mut _` cannot be shared between threads safely
    |
    = help: the trait `std::marker::Sync` is not implemented for `*mut _`
    = note: required because of the requirements on the impl of `std::marker::Send` for `&*mut _`
    = note: required because it appears within the type `[closure@/git/trybuild/test_suite/ui/compile-fail-3.rs:7:19: 9:6 x:&*mut _]`
" "
error[E0277]: `*mut _` cannot be shared between threads safely
   --> $DIR/compile-fail-3.rs:7:5
    |
7   |     thread::spawn(|| {
    |     ^^^^^^^^^^^^^ `*mut _` cannot be shared between threads safely
    |
    = help: the trait `std::marker::Sync` is not implemented for `*mut _`
    = note: required because of the requirements on the impl of `std::marker::Send` for `&*mut _`
    = note: required because it appears within the type `[closure@$DIR/ui/compile-fail-3.rs:7:19: 9:6 x:&*mut _]`
"}

test_normalize! {test_strip_path_dependencies "
error[E0277]: the trait bound `diesel::query_builder::SelectStatement<users::table, diesel::query_builder::select_clause::DefaultSelectClause, diesel::query_builder::distinct_clause::NoDistinctClause, diesel::query_builder::where_clause::WhereClause<diesel::expression::grouped::Grouped<diesel::expression::operators::Eq<posts::columns::id, diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>>>>>: diesel::query_builder::IntoUpdateTarget` is not satisfied
  --> $DIR/update_requires_valid_where_clause.rs:21:12
   |
21 |     update(users::table.filter(posts::id.eq(1)));
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `diesel::query_builder::IntoUpdateTarget` is not implemented for `diesel::query_builder::SelectStatement<users::table, diesel::query_builder::select_clause::DefaultSelectClause, diesel::query_builder::distinct_clause::NoDistinctClause, diesel::query_builder::where_clause::WhereClause<diesel::expression::grouped::Grouped<diesel::expression::operators::Eq<posts::columns::id, diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>>>>>`
   |
  ::: /home/user/documents/rust/diesel/diesel/src/query_builder/functions.rs:78:18
   |
78 | pub fn update<T: IntoUpdateTarget>(source: T) -> UpdateStatement<T::Table, T::WhereClause> {
   |                  ---------------- required by this bound in `diesel::update`
   |
   = help: the following implementations were found:
             <diesel::query_builder::SelectStatement<F, diesel::query_builder::select_clause::DefaultSelectClause, diesel::query_builder::distinct_clause::NoDistinctClause, W> as diesel::query_builder::IntoUpdateTarget>
" "
error[E0277]: the trait bound `diesel::query_builder::SelectStatement<users::table, diesel::query_builder::select_clause::DefaultSelectClause, diesel::query_builder::distinct_clause::NoDistinctClause, diesel::query_builder::where_clause::WhereClause<diesel::expression::grouped::Grouped<diesel::expression::operators::Eq<posts::columns::id, diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>>>>>: diesel::query_builder::IntoUpdateTarget` is not satisfied
  --> $DIR/update_requires_valid_where_clause.rs:21:12
   |
21 |     update(users::table.filter(posts::id.eq(1)));
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `diesel::query_builder::IntoUpdateTarget` is not implemented for `diesel::query_builder::SelectStatement<users::table, diesel::query_builder::select_clause::DefaultSelectClause, diesel::query_builder::distinct_clause::NoDistinctClause, diesel::query_builder::where_clause::WhereClause<diesel::expression::grouped::Grouped<diesel::expression::operators::Eq<posts::columns::id, diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>>>>>`
   |
  ::: $DIESEL/src/query_builder/functions.rs
   |
   | pub fn update<T: IntoUpdateTarget>(source: T) -> UpdateStatement<T::Table, T::WhereClause> {
   |                  ---------------- required by this bound in `diesel::update`
   |
   = help: the following implementations were found:
             <diesel::query_builder::SelectStatement<F, diesel::query_builder::select_clause::DefaultSelectClause, diesel::query_builder::distinct_clause::NoDistinctClause, W> as diesel::query_builder::IntoUpdateTarget>
"}

test_normalize! {test_cargo_registry "
error[E0277]: the trait bound `Thread: serde::de::Deserialize<'_>` is not satisfied
    --> src/main.rs:2:36
     |
2    |     let _ = serde_json::from_str::<std::thread::Thread>(\"???\");
     |                                    ^^^^^^^^^^^^^^^^^^^ the trait `serde::de::Deserialize<'_>` is not implemented for `Thread`
     |
    ::: /home/ferris/.cargo/registry/src/github.com-1ecc6299db9ec823/serde_json-1.0.64/src/de.rs:2584:8
     |
2584 |     T: de::Deserialize<'a>,
     |        ------------------- required by this bound in `serde_json::from_str`

For more information about this error, try `rustc --explain E0277`.
error: could not compile `testing` due to previous error
" "
error[E0277]: the trait bound `Thread: serde::de::Deserialize<'_>` is not satisfied
    --> $DIR/main.rs:2:36
     |
2    |     let _ = serde_json::from_str::<std::thread::Thread>(\"???\");
     |                                    ^^^^^^^^^^^^^^^^^^^ the trait `serde::de::Deserialize<'_>` is not implemented for `Thread`
     |
    ::: $CARGO/serde_json-1.0.64/src/de.rs
     |
     |     T: de::Deserialize<'a>,
     |        ------------------- required by this bound in `serde_json::from_str`
"}
