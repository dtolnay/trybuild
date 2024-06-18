test_normalize! {"
error[E0599]: the method `to_cxx_exception` exists for reference `&NonError`, but its trait bounds were not satisfied
 --> tests/ui/result_no_display.rs:4:19
  |
4 |         fn f() -> Result<()>;
  |                   ^^^^^^^^^^ method cannot be called on `&NonError` due to unsatisfied trait bounds
...
8 | pub struct NonError;
  | ------------------- doesn't satisfy `NonError: std::fmt::Display`
  |
  = note: the following trait bounds were not satisfied:
          `NonError: std::fmt::Display`
          which is required by `&NonError: ToCxxExceptionDefault`
note: the trait `std::fmt::Display` must be implemented
 --> /rustc/c5c7d2b37780dac1092e75f12ab97dd56c30861d/library/core/src/fmt/mod.rs:786:1
  |
  | pub trait Display {
  | ^^^^^^^^^^^^^^^^^
  = note: this error originates in the macro `::cxx::map_rust_error_to_cxx_exception` (in Nightly builds, run with -Z macro-backtrace for more info)
" "
error[E0599]: the method `to_cxx_exception` exists for reference `&NonError`, but its trait bounds were not satisfied
 --> tests/ui/result_no_display.rs:4:19
  |
4 |         fn f() -> Result<()>;
  |                   ^^^^^^^^^^ method cannot be called on `&NonError` due to unsatisfied trait bounds
...
8 | pub struct NonError;
  | ------------------- doesn't satisfy `NonError: std::fmt::Display`
  |
  = note: the following trait bounds were not satisfied:
          `NonError: std::fmt::Display`
          which is required by `&NonError: ToCxxExceptionDefault`
note: the trait `std::fmt::Display` must be implemented
 --> $RUST/core/src/fmt/mod.rs
  |
  | pub trait Display {
  | ^^^^^^^^^^^^^^^^^
  = note: this error originates in the macro `::cxx::map_rust_error_to_cxx_exception` (in Nightly builds, run with -Z macro-backtrace for more info)
"}
