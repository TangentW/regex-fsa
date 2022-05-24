fn main() {
    use regex_fsa::fsa::Symbol;
    use regex_fsa::regex::Regex;

    let a_or_b = "a".or("b");
    let regex = "ab".and(a_or_b.many()).and("ba");
    // 通过 `Thompson 构造法` 构造 NFA
    let nfa = regex.as_nfa();
    // 通过 `子集构造法` 构造 DFA
    let dfa = nfa.as_dfa();
    // 通过 `Hopcroft 算法` 最小化 DFA
    let dfa = dfa.minimize();

    // 构建自动机符号
    let symbols = "abaaabbba".chars().map(Symbol::Char);
    // 检查是否匹配
    let is_matched = dfa
        // 获取 DFA 经过所有符号后所到达的状态
        .end_of(symbols)
        // 判断此时到达的状态是否为状态（可接受状态）
        .map(|s| s.borrow().acceptable())
        .unwrap_or_default();

    println!("{}", is_matched);
}
