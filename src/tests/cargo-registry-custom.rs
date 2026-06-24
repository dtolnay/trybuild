test_normalize! {"
error[E0308]: mismatched types
  --> tests/compile-fail/surface_source_interval_badarg.rs:7:25
   |
5  |       let mut df = hydroflow_syntax! {
   |  __________________-
6  | |         // Should be a `Duration`.
7  | |         source_interval(5) -> for_each(std::mem::drop);
   | |                         ^ expected `Duration`, found integer
8  | |     };
   | |_____- arguments to this function are incorrect
   |
note: function defined here
  --> /root/.cargo/registry/src/foo-registry.bar.com-3456e42d3d53e27e/tokio-1.26.0/src/time/interval.rs:74:8
   |
74 | pub fn interval(period: Duration) -> Interval {
   |        ^^^^^^^^
" "
error[E0308]: mismatched types
 --> tests/compile-fail/surface_source_interval_badarg.rs:7:25
  |
5 |       let mut df = hydroflow_syntax! {
  |  __________________-
6 | |         // Should be a `Duration`.
7 | |         source_interval(5) -> for_each(std::mem::drop);
  | |                         ^ expected `Duration`, found integer
8 | |     };
  | |_____- arguments to this function are incorrect
  |
note: function defined here
 --> $CARGO/tokio-$VERSION/src/time/interval.rs
  |
  | pub fn interval(period: Duration) -> Interval {
  |        ^^^^^^^^
"}
