test_normalize! {"
error[E0038]: the trait `MyTrait` is not dyn compatible
   --> src/main.rs:8:12
    |
8   |     let _: &dyn MyTrait;
    |            ^^^^^^^^^^^^ `MyTrait` is not dyn compatible
    |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#object-safety>
   --> /home/ferris/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/hash/mod.rs:199:8
    |
199 |     fn hash<H: Hasher>(&self, state: &mut H);
    |        ^^^^ ...because method `hash` has generic type parameters
    |
   ::: src/main.rs:3:7
    |
3   | trait MyTrait: Hash {
    |       ------- this trait is not dyn compatible...
    = help: consider moving `hash` to another trait

For more information about this error, try `rustc --explain E0038`.
error: could not compile `testing` (bin \"testing\") due to 1 previous error
" "
error[E0038]: the trait `MyTrait` is not dyn compatible
 --> src/main.rs:8:12
  |
8 |     let _: &dyn MyTrait;
  |            ^^^^^^^^^^^^ `MyTrait` is not dyn compatible
  |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#object-safety>
 --> $RUST/core/src/hash/mod.rs
  |
  |     fn hash<H: Hasher>(&self, state: &mut H);
  |        ^^^^ ...because method `hash` has generic type parameters
  |
 ::: src/main.rs:3:7
  |
3 | trait MyTrait: Hash {
  |       ------- this trait is not dyn compatible...
  = help: consider moving `hash` to another trait
"}
