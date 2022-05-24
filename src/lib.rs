#![feature(let_else)]

pub mod fsa;
pub mod regex;

use crate::fsa::nfa::NFA;
use crate::fsa::{dfa::DFA, Symbol};
use crate::regex::Regex;

pub struct Matcher(DFA);

impl Matcher {
    pub fn from_regex(regex: impl Regex) -> Self {
        Self::from_nfa(regex.as_nfa())
    }

    pub fn from_dfa(dfa: DFA) -> Self {
        Self(dfa.minimize())
    }

    pub fn from_nfa(nfa: NFA) -> Self {
        Self::from_dfa(nfa.as_dfa().minimize())
    }

    pub fn is_matched(&self, str: impl AsRef<str>) -> bool {
        let symbols = str.as_ref().chars().map(Symbol::Char);
        self.0
            .end_of(symbols)
            .map(|s| s.borrow().acceptable())
            .unwrap_or_default()
    }
}
