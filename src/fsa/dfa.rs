use crate::fsa::{self, StateID, Symbol};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, LinkedList};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

pub struct DFA {
    start: StateNode,
}

impl DFA {
    #[inline]
    pub fn new(start: StateNode) -> Self {
        Self { start }
    }

    /// 根据符号序列获取最终可达状态
    pub fn end_of(&self, symbols: impl IntoIterator<Item = Symbol>) -> Option<StateNode> {
        let mut state = self.start.clone();

        for symbol in symbols {
            let new_state = if let Some(state) = state.borrow().next_state(symbol) {
                state
            } else {
                return None;
            };
            state = new_state;
        }

        Some(state)
    }

    /// 最小化
    pub fn minimize(&self) -> DFA {
        let mut group = self.all_states().divide_by_acceptable();

        loop {
            let group_copy = group.clone();
            for set in group_copy.iter() {
                group.remove(&set);
                group.extend(set.divide(&group_copy).into_iter())
            }
            if group.len() == group_copy.len() {
                break;
            }
        }

        self.merge(group)
    }

    /// 获取所有的状态集
    fn all_states(&self) -> StateSet {
        let mut set = StateSet::new();
        let mut queue = LinkedList::from([self.start.clone()]);

        while let Some(state) = queue.pop_front() {
            if set.contains(&state.borrow()) {
                continue;
            }
            queue.extend(state.borrow().transitions.values().cloned());
            set.insert(state);
        }

        set
    }

    /// 将状态集组合并，构建新的 DFA
    fn merge(&self, group: StateSetGroup) -> DFA {
        let mut typical_states = HashMap::new();

        for set in group.iter() {
            let typical_state = Self::typical_state_of_set(set, &mut typical_states);
            for state in set.states() {
                let mut transitions = state.borrow().transitions.clone();
                for next_state in transitions.values_mut() {
                    *next_state =
                        Self::typical_state_of_state(next_state, &group, &mut typical_states);
                }
                typical_state.borrow_mut().transitions.extend(transitions);
            }
        }

        let start = Self::typical_state_of_state(&self.start, &group, &mut typical_states);
        DFA::new(start)
    }

    #[inline]
    fn typical_state_of_state<'a>(
        state: &StateNode,
        group: &'a StateSetGroup,
        typical_states: &mut HashMap<&'a StateSet, StateNode>,
    ) -> StateNode {
        let set = group.iter().find(|s| s.contains(&state.borrow())).unwrap();
        Self::typical_state_of_set(set, typical_states).clone()
    }

    #[inline]
    fn typical_state_of_set<'a>(
        set: &'a StateSet,
        typical_states: &mut HashMap<&'a StateSet, StateNode>,
    ) -> StateNode {
        typical_states
            .entry(set)
            .or_insert(State::new_node(set.states().any(|s| s.borrow().acceptable)))
            .clone()
    }
}

impl Debug for DFA {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn state_str(state: &State) -> String {
            if state.acceptable {
                format!("[{:?}]", state.id)
            } else {
                format!("({:?})", state.id)
            }
        }

        let mut ids = HashSet::new();
        let mut queue = LinkedList::from([self.start.clone()]);
        while let Some(state) = queue.pop_front() {
            let id = state.borrow().id;
            if ids.contains(&id) {
                continue;
            }
            ids.insert(id);

            for (symbol, next_state) in &state.borrow().transitions {
                queue.push_back(next_state.clone());

                write!(
                    f,
                    "[{}, {:?}] = {}\n",
                    state_str(&state.borrow()),
                    symbol,
                    state_str(&next_state.borrow())
                )?;
            }
        }
        Result::Ok(())
    }
}

pub type StateNode = fsa::StateNode<State>;

/// DFA 状态
#[derive(Debug)]
pub struct State {
    id: StateID,
    transitions: HashMap<Symbol, StateNode>,
    acceptable: bool,
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
    pub fn new(acceptable: bool) -> State {
        State {
            id: StateID::obtain(),
            transitions: Default::default(),
            acceptable,
        }
    }

    #[inline]
    pub fn acceptable(&self) -> bool {
        self.acceptable
    }

    #[inline]
    pub fn new_node(acceptable: bool) -> StateNode {
        Rc::new(RefCell::new(State::new(acceptable)))
    }

    #[inline]
    pub fn transition(&mut self, symbol: Symbol, target: StateNode) {
        self.transitions.insert(symbol, target);
    }

    /// 根据符号获取下一个状态
    #[inline]
    pub fn next_state(&self, symbol: Symbol) -> Option<StateNode> {
        self.transitions.get(&symbol).cloned()
    }
}

type StateSet = fsa::StateSet<State>;
type StateSetGroup = HashSet<StateSet>;

impl StateSet {
    /// 按照 `终态` / `非终态` 进行分组
    fn divide_by_acceptable(&self) -> StateSetGroup {
        let mut unacceptable = Self::new();
        let mut acceptable = Self::new();

        for state in self.states() {
            if state.borrow().acceptable {
                acceptable.insert(state);
            } else {
                unacceptable.insert(state);
            }
        }

        HashSet::from([unacceptable, acceptable])
    }

    /// 拆分，将状态集根据目前的状态集组拆分成独立的 N 组
    fn divide(&self, groups: &StateSetGroup) -> StateSetGroup {
        let symbols = self.alphabet();
        let mut sets = HashMap::new();

        for state in self.states() {
            let sets_of_symbols = symbols
                .iter()
                .copied()
                .map(|symbol| {
                    // 此刻状态经过符号变换后落入到的在传入状态集组的状态集
                    state
                        .borrow()
                        .next_state(symbol)
                        .and_then(|s| groups.iter().find(|set| set.contains(&s.borrow())))
                })
                .collect::<Vec<_>>();
            sets.entry(sets_of_symbols)
                .or_insert(LinkedList::new())
                .push_back(state.clone());
        }

        StateSetGroup::from_iter(sets.into_values().map(StateSet::from_states))
    }
}
