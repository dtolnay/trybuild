test_normalize! {"
error[E0004]: non-exhaustive patterns: `MyPhantom::__Phantom(_)` not covered
  --> src/main.rs:9:11
   |
 9 |     match phantom {
   |           ^^^^^^^ pattern `MyPhantom::__Phantom(_)` not covered
   |
note: `MyPhantom<u8>` defined here
  --> src/main.rs:4:8
   |
 3 | #[phantom]
   | ---------- not covered
 4 | struct MyPhantom<T: ?Sized>;
   |        ^^^^^^^^^
   = note: the matched value is of type `MyPhantom<u8>`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
10 ~         MyPhantom => {},
11 +         MyPhantom::__Phantom(_) => todo!()
   |
" "
error[E0004]: non-exhaustive patterns: `MyPhantom::__Phantom(_)` not covered
  --> src/main.rs:9:11
   |
 9 |     match phantom {
   |           ^^^^^^^ pattern `MyPhantom::__Phantom(_)` not covered
   |
note: `MyPhantom<u8>` defined here
  --> src/main.rs:4:8
   |
 3 | #[phantom]
   | ---------- not covered
 4 | struct MyPhantom<T: ?Sized>;
   |        ^^^^^^^^^
   = note: the matched value is of type `MyPhantom<u8>`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
10 ~         MyPhantom => {},
11 +         MyPhantom::__Phantom(_) => todo!()
   |
"}
