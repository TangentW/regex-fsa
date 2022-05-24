use crate::fsa::{
    self,
    dfa::{self, DFA},
    StateID, Symbol,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, LinkedList};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

pub struct NFA {
    start: StateNode,
    end: StateNode,
}

impl NFA {
    #[inline]
    pub fn new(start: StateNode, end: StateNode) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn start(&self) -> StateNode {
        self.start.clone()
    }

    #[inline]
    pub fn end(&self) -> StateNode {
        self.end.clone()
    }

    /// 子集构造法 (Subset Construction) 转换成 DFA
    pub fn as_dfa(&self) -> DFA {
        let start = Rc::new(StateSet::from_single(self.start.clone()).e_closure());
        let dfa_start = dfa::State::new_node(start.contains(&self.end().borrow()));

        let mut dfa_states = HashMap::from([(start.clone(), dfa_start.clone())]);
        let mut queue = LinkedList::from([start]);

        while let Some(set) = queue.pop_front() {
            for symbol in set.alphabet() {
                let new_set = Rc::new(set.move_to(symbol).e_closure());
                if new_set.is_empty() {
                    continue;
                }

                if !dfa_states.contains_key(&new_set) {
                    queue.push_back(new_set.clone());
                }

                let dfa_state = dfa_states
                    .entry(new_set)
                    .or_insert_with_key(|k| dfa::State::new_node(k.contains(&self.end().borrow())))
                    .clone();
                dfa_states
                    .get(&set)
                    .unwrap()
                    .borrow_mut()
                    .transition(symbol, dfa_state);
            }
        }

        DFA::new(dfa_start)
    }
}

impl Debug for NFA {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut ids = HashSet::new();
        let mut queue = LinkedList::from([self.start.clone()]);
        while let Some(state) = queue.pop_front() {
            let id = state.borrow().id;
            if ids.contains(&id) {
                continue;
            }
            ids.insert(id);

            for (symbol, targets) in &state.borrow().transitions {
                for target in targets {
                    queue.push_back(target.clone());
                    write!(f, "[{:?}, {:?}] = {:?}\n", id, symbol, target.borrow().id)?;
                }
            }
        }
        Result::Ok(())
    }
}

pub type StateNode = fsa::StateNode<State>;

/// NFA 状态
#[derive(Debug)]
pub struct State {
    id: StateID,
    transitions: HashMap<Symbol, LinkedList<StateNode>>,
}

impl fsa::State for State {
    #[inline]
    fn id(&self) -> StateID {
        self.id
    }

    #[inline]
    fn alphabet(&self) -> Box<dyn Iterator<Item = Symbol> + '_> {
        Box::new(self.transitions.keys().copied())
    }
}

impl State {
    #[inline]
    pub fn new() -> State {
        State {
            id: StateID::obtain(),
            transitions: Default::default(),
        }
    }

    /// 构建节点（包裹在 Rc 和 RefCell 中)
    #[inline]
    pub fn new_node() -> StateNode {
        Rc::new(RefCell::new(State::new()))
    }

    /// ε 转移
    #[inline]
    pub fn e_transition(&mut self, target: StateNode) -> &mut Self {
        self.transition(Symbol::Epsilon, target)
    }

    /// 转移
    #[inline]
    pub fn transition(&mut self, symbol: Symbol, target: StateNode) -> &mut Self {
        self.transitions
            .entry(symbol)
            .or_default()
            .push_back(target);
        self
    }

    /// 根据符号获取接下来的状态集
    #[inline]
    fn next_states(&self, symbol: Symbol) -> Option<impl Iterator<Item = StateNode> + '_> {
        self.transitions.get(&symbol).map(|s| s.iter().cloned())
    }
}

type StateSet = fsa::StateSet<State>;

impl StateSet {
    /// move 运算集
    fn move_to(&self, symbol: Symbol) -> Self {
        let mut set = Self::new();
        for state in self.states() {
            if let Some(next_states) = state.borrow().next_states(symbol) {
                set.extend(next_states);
            }
        }
        set
    }

    /// ε-closure 运算集
    fn e_closure(&self) -> Self {
        let mut set = Self::new();
        let mut queue = LinkedList::from_iter(self.states());
        while let Some(state) = queue.pop_front() {
            if set.contains(&state.borrow()) {
                continue;
            }
            if let Some(states) = state.borrow().next_states(Symbol::Epsilon) {
                queue.extend(states);
            }
            set.insert(state);
        }
        set
    }
}
