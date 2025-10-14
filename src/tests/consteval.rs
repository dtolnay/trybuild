test_normalize! {
    WORKSPACE="/git/monostate"
"
error[E0080]: evaluation panicked: assertion failed: N == mem::size_of::<T::Type>()
  --> /git/monostate/src/string.rs:46:13
   |
46 |             assert!(N == mem::size_of::<T::Type>());
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ evaluation of `<(monostate::alphabet::len<8>, (monostate::alphabet::char<'a'>, monostate::alphabet::char<'s'>, monostate::alphabet::char<'d'>, monostate::alphabet::char<'f'>)) as monostate::string::Sealed>::__private::{constant#0}` failed here

note: erroneous constant encountered
  --> /git/monostate/src/string.rs:45:9
   |
45 | /         const {
46 | |             assert!(N == mem::size_of::<T::Type>());
47 | |         }
   | |_________^

note: erroneous constant encountered
  --> /git/monostate/src/string.rs:29:33
   |
29 |     const VALUE: &'static str = T::__private.0;
   |                                 ^^^^^^^^^^^^

note: erroneous constant encountered
   --> /git/monostate/src/value.rs:132:37
    |
132 |     pub const VALUE: &'static str = V::VALUE;
    |                                     ^^^^^^^^
" "
error[E0080]: evaluation panicked: assertion failed: N == mem::size_of::<T::Type>()
 --> $WORKSPACE/src/string.rs
  |
  |             assert!(N == mem::size_of::<T::Type>());
  |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ evaluation of `<(monostate::alphabet::len<8>, (monostate::alphabet::char<'a'>, monostate::alphabet::char<'s'>, monostate::alphabet::char<'d'>, monostate::alphabet::char<'f'>)) as monostate::string::Sealed>::__private::{constant#0}` failed here

note: erroneous constant encountered
 --> $WORKSPACE/src/string.rs
  |
  | /         const {
  | |             assert!(N == mem::size_of::<T::Type>());
  | |         }
  | |_________^

note: erroneous constant encountered
 --> $WORKSPACE/src/string.rs
  |
  |     const VALUE: &'static str = T::__private.0;
  |                                 ^^^^^^^^^^^^

note: erroneous constant encountered
 --> $WORKSPACE/src/value.rs
  |
  |     pub const VALUE: &'static str = V::VALUE;
  |                                     ^^^^^^^^
"}
