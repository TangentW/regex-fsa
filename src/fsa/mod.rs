use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub mod dfa;
pub mod nfa;

/// 输入符号
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum Symbol {
    Epsilon,
    Char(char),
}

impl Symbol {
    #[inline]
    pub fn is_epsilon(&self) -> bool {
        self == &Self::Epsilon
    }
}

impl From<char> for Symbol {
    #[inline]
    fn from(char: char) -> Self {
        Symbol::Char(char)
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Epsilon => write!(f, "ε"),
            Self::Char(c) => write!(f, "{c}"),
        }
    }
}

/// 状态 ID，作为每个状态的唯一表示，参与集合运算
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct StateID(usize);

impl StateID {
    fn obtain() -> Self {
        static mut ID_RAW_POOL: usize = 0;

        unsafe {
            let raw = ID_RAW_POOL;
            ID_RAW_POOL += 1;
            Self(raw)
        }
    }
}

impl Debug for StateID {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// 状态，其中包含标志其唯一的 ID
trait State {
    /// 唯一的 ID
    fn id(&self) -> StateID;

    /// 可接受的所有符号
    fn alphabet<'a>(&'a self) -> Box<dyn Iterator<Item = Symbol> + 'a>;
}

/// 状态节点，通过 Rc 和 RefCell 包裹
type StateNode<T> = Rc<RefCell<T>>;

/// 状态集，参与集合运算
#[derive(Debug)]
struct StateSet<T>(HashMap<StateID, StateNode<T>>);

impl<T> StateSet<T>
where
    T: State,
{
    #[inline]
    fn new() -> Self {
        Self(HashMap::new())
    }

    #[inline]
    fn from_single(state: StateNode<T>) -> Self {
        let mut set = Self::new();
        set.insert(state);
        set
    }

    #[inline]
    fn from_states(states: impl IntoIterator<Item = StateNode<T>>) -> Self {
        let mut set = Self::new();
        set.extend(states);
        set
    }

    #[inline]
    fn insert(&mut self, state: StateNode<T>) {
        let id = state.borrow().id();
        self.0.insert(id, state);
    }

    #[inline]
    fn extend(&mut self, states: impl IntoIterator<Item = StateNode<T>>) {
        self.0.extend(states.into_iter().map(|s| {
            let id = s.borrow().id();
            (id, s)
        }))
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    fn contains(&self, state: &T) -> bool {
        self.0.contains_key(&state.id())
    }

    #[inline]
    fn states(&self) -> impl Iterator<Item = StateNode<T>> + '_ {
        self.0.values().cloned()
    }

    /// 可接受的所有符号（输入字母表除ε)
    fn alphabet(&self) -> HashSet<Symbol> {
        HashSet::from_iter(self.0.values().flat_map(|s| {
            s.borrow()
                .alphabet()
                .filter(|s| !s.is_epsilon())
                .collect::<Vec<_>>()
        }))
    }

    /// 为判等、哈希运算提供支持
    #[inline]
    fn key(&self) -> BTreeSet<&StateID> {
        BTreeSet::from_iter(self.0.keys())
    }
}

impl<T> Clone for StateSet<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Hash for StateSet<T>
where
    T: State,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state)
    }
}

impl<T> Eq for StateSet<T> where T: State {}

impl<T> PartialEq<Self> for StateSet<T>
where
    T: State,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.key().eq(&other.key())
    }
}
