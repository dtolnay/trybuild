use std::path::Path;

macro_rules! test_normalize {
    ($name:ident $original:literal $expected:literal) => {
        #[test]
        fn $name() {
            let context = super::Context {
                krate: "trybuild",
                source_dir: Path::new("/git/trybuild/test_suite"),
                workspace: Path::new("/git/trybuild"),
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
