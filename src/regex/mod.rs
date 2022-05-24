use crate::fsa::nfa::NFA;
use crate::regex::tokens::{Alternative, Closure, Concatenation, Some};

pub mod tokens;

pub trait Regex: Sized {
    /// 转变为 NFA
    fn as_nfa(&self) -> NFA;

    /// 连接两个正规式
    #[inline]
    fn and<R>(self, next: R) -> Concatenation<Self, R> {
        Concatenation::new(self, next)
    }

    /// 构建两个正规式候选
    #[inline]
    fn or<R>(self, other: R) -> Alternative<Self, R> {
        Alternative::new(self, other)
    }

    /// 0 次或多次匹配
    #[inline]
    fn many(self) -> Closure<Self> {
        Closure::new(self)
    }

    /// 1 次以上多次匹配
    #[inline]
    fn some(self) -> Some<Self> {
        Some::new(self)
    }
}
