test_normalize! {
    INPUT="tests/ui-testing-optout.rs"
"
error[E0412]: cannot find type `F` in this scope
  --> $DIR/ui-testing-optout.rs:92:10
   |
 4 | type A = B;
   | ----------- similarly named type alias `A` defined here
...
92 | type E = F;
   |          ^ help: a type alias with a similar name exists: `A`
" "
error[E0412]: cannot find type `F` in this scope
  --> $DIR/ui-testing-optout.rs:92:10
   |
 4 | type A = B;
   | ----------- similarly named type alias `A` defined here
...
92 | type E = F;
   |          ^ help: a type alias with a similar name exists: `A`
"}
