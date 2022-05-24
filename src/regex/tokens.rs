use crate::fsa::nfa::{self, NFA};
use crate::regex::Regex;
use std::fmt::{Debug, Formatter};

/// 字符 (a)
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct Char(char);

impl Char {
    #[inline]
    pub fn new(char: char) -> Self {
        Self(char)
    }
}

impl Regex for Char {
    fn as_nfa(&self) -> NFA {
        let (start, end) = (nfa::State::new_node(), nfa::State::new_node());

        start.borrow_mut().transition(self.0.into(), end.clone());

        NFA::new(start, end)
    }
}

impl Debug for Char {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// 连接运算 (ab)
#[derive(Clone)]
pub struct Concatenation<L, R>(L, R);

impl<L, R> Concatenation<L, R> {
    #[inline]
    pub fn new(l: L, r: R) -> Self {
        Self(l, r)
    }
}

impl<L, R> Regex for Concatenation<L, R>
where
    L: Regex,
    R: Regex,
{
    fn as_nfa(&self) -> NFA {
        let (left, right) = (self.0.as_nfa(), self.1.as_nfa());

        left.end().borrow_mut().e_transition(right.start());

        NFA::new(left.start(), right.end())
    }
}

impl<L, R> Debug for Concatenation<L, R>
where
    L: Debug,
    R: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}", self.0, self.1)
    }
}

/// 选择运算 (a|b)
#[derive(Clone)]
pub struct Alternative<L, R>(L, R);

impl<L, R> Alternative<L, R> {
    #[inline]
    pub fn new(l: L, r: R) -> Self {
        Self(l, r)
    }
}

impl<L, R> Regex for Alternative<L, R>
where
    L: Regex,
    R: Regex,
{
    fn as_nfa(&self) -> NFA {
        let (start, end) = (nfa::State::new_node(), nfa::State::new_node());
        let (left, right) = (self.0.as_nfa(), self.1.as_nfa());

        start
            .borrow_mut()
            .e_transition(left.start())
            .e_transition(right.start());

        left.end().borrow_mut().e_transition(end.clone());
        right.end().borrow_mut().e_transition(end.clone());

        NFA::new(start, end)
    }
}

impl<L, R> Debug for Alternative<L, R>
where
    L: Debug,
    R: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}|{:?})", self.0, self.1)
    }
}

/// 闭包运算 (a*)
#[derive(Clone)]
pub struct Closure<R>(R);

impl<R> Closure<R> {
    #[inline]
    pub fn new(r: R) -> Self {
        Self(r)
    }
}

impl<R> Regex for Closure<R>
where
    R: Regex,
{
    fn as_nfa(&self) -> NFA {
        let (start, end) = (nfa::State::new_node(), nfa::State::new_node());
        let inner = self.0.as_nfa();

        start
            .borrow_mut()
            .e_transition(end.clone())
            .e_transition(inner.start());
        inner
            .end()
            .borrow_mut()
            .e_transition(inner.start())
            .e_transition(end.clone());

        NFA::new(start, end)
    }
}

impl<R> Debug for Closure<R>
where
    R: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?})*", self.0)
    }
}

/// 一个或多个
#[derive(Clone)]
pub struct Some<R>(R);

impl<R> Some<R> {
    #[inline]
    pub fn new(r: R) -> Self {
        Self(r)
    }
}

impl<R> Regex for Some<R>
where
    R: Regex + Clone,
{
    #[inline]
    fn as_nfa(&self) -> NFA {
        Concatenation::new(self.0.clone(), Closure::new(self.0.clone())).as_nfa()
    }
}

impl<R> Debug for Some<R>
where
    R: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?})+", self.0)
    }
}

/// 字符串
impl<T> Regex for T
where
    T: AsRef<str>,
{
    fn as_nfa(&self) -> NFA {
        self.as_ref()
            .chars()
            .map(Char)
            .map(|c| c.as_nfa())
            .reduce(|l, r| {
                l.end().borrow_mut().e_transition(r.start());
                NFA::new(l.start(), r.end())
            })
            .expect("正规式不能为空！")
    }
}
