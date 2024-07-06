test_normalize! {r#"
error[E0277]: the trait bound `MyStruct: Deserialize<'_>` is not satisfied
  --> tests/ui/on_unimplemented.rs:22:23
   |
22 |     let _: MyStruct = from_str("");
   |                       ^^^^^^^^^^^^ the trait `Deserialize<'_>` is not implemented for `MyStruct`
   |
   = help: the following other types implement trait `Deserialize<'de>`:
             &'a Path
             &'a [u8]
             &'a str
             ()
             (T,)
             (T0, T1)
             (T0, T1, T2)
             (T0, T1, T2, T3)
note: required by a bound in `from_str`
  --> tests/ui/on_unimplemented.rs:13:8
   |
11 | fn from_str<'de, T>(_: &'de str) -> T
   |    -------- required by a bound in this function
12 | where
13 |     T: Deserialize<'de>,
   |        ^^^^^^^^^^^^^^^^ required by this bound in `from_str`

error[E0277]: the trait bound `MyStruct: Deserialize<'_>` is not satisfied
  --> tests/ui/on_unimplemented.rs:22:23
   |
22 |     let _: MyStruct = from_str("");
   |                       ^^^^^^^^^^^^ the trait `Deserialize<'_>` is not implemented for `MyStruct`
   |
   = help: the following other types implement trait `Deserialize<'de>`:
             &'a Path
             &'a [u8]
             &'a str
             ()
             (T,)
             (T0, T1)
             (T0, T1, T2)
             (T0, T1, T2, T3)
             (T0, T1, T2, T3, T4)
note: required by a bound in `from_str`
  --> tests/ui/on_unimplemented.rs:13:8
   |
11 | fn from_str<'de, T>(_: &'de str) -> T
   |    -------- required by a bound in this function
12 | where
13 |     T: Deserialize<'de>,
   |        ^^^^^^^^^^^^^^^^ required by this bound in `from_str`

error[E0277]: the trait bound `MyStruct: Deserialize<'_>` is not satisfied
  --> tests/ui/on_unimplemented.rs:22:23
   |
22 |     let _: MyStruct = from_str("");
   |                       ^^^^^^^^^^^^ the trait `Deserialize<'_>` is not implemented for `MyStruct`
   |
   = help: the following other types implement trait `Deserialize<'de>`:
             &'a Path
             &'a [u8]
             &'a str
             ()
             (T,)
             (T0, T1)
             (T0, T1, T2)
             (T0, T1, T2, T3)
             (T0, T1, T2, T3, T4)
             (T0, T1, T2, T3, T4, T5)
note: required by a bound in `from_str`
  --> tests/ui/on_unimplemented.rs:13:8
   |
11 | fn from_str<'de, T>(_: &'de str) -> T
   |    -------- required by a bound in this function
12 | where
13 |     T: Deserialize<'de>,
   |        ^^^^^^^^^^^^^^^^ required by this bound in `from_str`
"# r#"
error[E0277]: the trait bound `MyStruct: Deserialize<'_>` is not satisfied
  --> tests/ui/on_unimplemented.rs:22:23
   |
22 |     let _: MyStruct = from_str("");
   |                       ^^^^^^^^^^^^ the trait `Deserialize<'_>` is not implemented for `MyStruct`
   |
   = help: the following other types implement trait `Deserialize<'de>`:
             &'a Path
             &'a [u8]
             &'a str
             ()
             (T,)
             (T0, T1)
             (T0, T1, T2)
             (T0, T1, T2, T3)
note: required by a bound in `from_str`
  --> tests/ui/on_unimplemented.rs:13:8
   |
11 | fn from_str<'de, T>(_: &'de str) -> T
   |    -------- required by a bound in this function
12 | where
13 |     T: Deserialize<'de>,
   |        ^^^^^^^^^^^^^^^^ required by this bound in `from_str`

error[E0277]: the trait bound `MyStruct: Deserialize<'_>` is not satisfied
  --> tests/ui/on_unimplemented.rs:22:23
   |
22 |     let _: MyStruct = from_str("");
   |                       ^^^^^^^^^^^^ the trait `Deserialize<'_>` is not implemented for `MyStruct`
   |
   = help: the following other types implement trait `Deserialize<'de>`:
             &'a Path
             &'a [u8]
             &'a str
             ()
             (T,)
             (T0, T1)
             (T0, T1, T2)
             (T0, T1, T2, T3)
             (T0, T1, T2, T3, T4)
note: required by a bound in `from_str`
  --> tests/ui/on_unimplemented.rs:13:8
   |
11 | fn from_str<'de, T>(_: &'de str) -> T
   |    -------- required by a bound in this function
12 | where
13 |     T: Deserialize<'de>,
   |        ^^^^^^^^^^^^^^^^ required by this bound in `from_str`

error[E0277]: the trait bound `MyStruct: Deserialize<'_>` is not satisfied
  --> tests/ui/on_unimplemented.rs:22:23
   |
22 |     let _: MyStruct = from_str("");
   |                       ^^^^^^^^^^^^ the trait `Deserialize<'_>` is not implemented for `MyStruct`
   |
   = help: the following other types implement trait `Deserialize<'de>`:
             &'a Path
             &'a [u8]
             &'a str
             ()
             (T,)
             (T0, T1)
             (T0, T1, T2)
             (T0, T1, T2, T3)
           and $N others
note: required by a bound in `from_str`
  --> tests/ui/on_unimplemented.rs:13:8
   |
11 | fn from_str<'de, T>(_: &'de str) -> T
   |    -------- required by a bound in this function
12 | where
13 |     T: Deserialize<'de>,
   |        ^^^^^^^^^^^^^^^^ required by this bound in `from_str`
"#}
