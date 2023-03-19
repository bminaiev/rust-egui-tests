use std::{collections::VecDeque, fs, str::FromStr};

pub struct Scanner {
    tokens: VecDeque<String>,
}

impl Scanner {
    pub fn new(path: &str) -> Self {
        let mut tokens = VecDeque::new();
        for token in fs::read_to_string(path).unwrap().split_ascii_whitespace() {
            tokens.push_back(token.to_owned());
        }
        Self { tokens }
    }

    pub fn next<T: FromStr>(&mut self) -> T
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        T::from_str(&self.tokens.pop_front().unwrap()).unwrap()
    }
}
