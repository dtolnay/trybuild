pub use self::r#impl::Diff;

pub enum Render<'a> {
    Common(&'a str),
    Unique(&'a str),
}

#[cfg(feature = "diff")]
mod r#impl {
    use super::Render;
    use dissimilar::Chunk;
    use std::cmp;

    pub struct Diff<'a> {
        pub worth_printing: bool,
        expected: &'a str,
        actual: &'a str,
        diff: Vec<Chunk<'a>>,
    }

    impl<'a> Diff<'a> {
        pub fn compute(expected: &'a str, actual: &'a str) -> Self {
            let diff = dissimilar::diff(expected, actual);

            let mut common_len = 0;
            for chunk in &diff {
                if let Chunk::Equal(common) = chunk {
                    common_len += common.len();
                }
            }

            let bigger_len = cmp::max(expected.len(), actual.len());
            let worth_printing = 5 * common_len >= 4 * bigger_len;

            Diff {
                worth_printing,
                expected,
                actual,
                diff,
            }
        }

        pub fn iter<'i>(&'i self, input: &str) -> impl Iterator<Item = Render<'a>> + 'i {
            let expected = input == self.expected;
            let actual = input == self.actual;
            self.diff.iter().filter_map(move |chunk| match chunk {
                Chunk::Equal(common) => Some(Render::Common(common)),
                Chunk::Delete(unique) if expected => Some(Render::Unique(unique)),
                Chunk::Insert(unique) if actual => Some(Render::Unique(unique)),
                _ => None,
            })
        }
    }
}

#[cfg(not(feature = "diff"))]
mod r#impl {
    use super::Render;

    pub struct Diff {
        pub worth_printing: bool,
    }

    impl Diff {
        pub fn compute(_expected: &str, _actual: &str) -> Self {
            Diff {
                worth_printing: false,
            }
        }

        pub fn iter(&self, _input: &str) -> impl Iterator<Item = Render<'static>> {
            let _ = Render::Common;
            let _ = Render::Unique;
            [].iter()
        }
    }
}
