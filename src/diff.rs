use diffr_lib::{diff, tokenize, DiffInput, HashedSpan, Snake, Tokenization};
use std::cmp;
use std::iter::Peekable;
use std::slice;

pub struct Diff<'a> {
    pub worth_printing: bool,
    expected: &'a str,
    expected_tokens: Vec<HashedSpan>,
    actual: &'a str,
    actual_tokens: Vec<HashedSpan>,
    common: Vec<Snake>,
}

impl<'a> Diff<'a> {
    pub fn compute(expected: &'a str, actual: &'a str) -> Self {
        let mut actual_tokens = Vec::new();
        tokenize(actual.as_bytes(), 0, &mut actual_tokens);
        let added = Tokenization::new(actual.as_bytes(), &actual_tokens);

        let mut expected_tokens = Vec::new();
        tokenize(expected.as_bytes(), 0, &mut expected_tokens);
        let removed = Tokenization::new(expected.as_bytes(), &expected_tokens);

        let input = DiffInput { added, removed };
        let mut scratch = Vec::new();
        let mut common = Vec::new();
        diff(&input, &mut scratch, &mut common);

        let min_len = cmp::max(expected_tokens.len(), actual_tokens.len());
        let common_len = common.iter().map(|snake| snake.len).sum::<isize>() as usize;
        let worth_printing = common_len / 4 >= min_len / 5;

        Diff {
            worth_printing,
            expected,
            expected_tokens,
            actual,
            actual_tokens,
            common,
        }
    }

    pub fn iter(&self, input: &str) -> Iter {
        if input == self.expected {
            Iter {
                pos: 0,
                input: self.expected,
                tokens: &self.expected_tokens,
                common: self.common.iter().peekable(),
                token_index: |snake| snake.x0,
            }
        } else if input == self.actual {
            Iter {
                pos: 0,
                input: self.actual,
                tokens: &self.actual_tokens,
                common: self.common.iter().peekable(),
                token_index: |snake| snake.y0,
            }
        } else {
            panic!("unrecognized input");
        }
    }
}

pub struct Iter<'a> {
    pos: usize,
    input: &'a str,
    tokens: &'a [HashedSpan],
    common: Peekable<slice::Iter<'a, Snake>>,
    token_index: fn(&Snake) -> isize,
}

pub enum Chunk<'a> {
    Common(&'a str),
    Unique(&'a str),
}

impl<'a> Iterator for Iter<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.common.peek() {
            Some(common) => {
                let index = (self.token_index)(common);
                let begin = &self.tokens[index as usize];
                if self.pos < begin.lo {
                    let chunk = &self.input[self.pos..begin.lo];
                    self.pos = begin.lo;
                    Some(Chunk::Unique(chunk))
                } else {
                    let index = (self.token_index)(common) + common.len - 1;
                    let end = &self.tokens[index as usize];
                    let chunk = &self.input[begin.lo..end.hi];
                    self.common.next().unwrap();
                    self.pos = end.hi;
                    Some(Chunk::Common(chunk))
                }
            }
            None => {
                if self.pos < self.input.len() {
                    let chunk = &self.input[self.pos..];
                    self.pos = self.input.len();
                    Some(Chunk::Unique(chunk))
                } else {
                    None
                }
            }
        }
    }
}
