fn main() {
    #[cfg(feature="a")]
    assert!(true);
    #[cfg(feature="b")]
    assert!(false);
}
