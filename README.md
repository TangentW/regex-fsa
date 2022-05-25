# 基于自动机理论的简易正则表达式引擎

## Start
目前尚未实现正则表达式字符串解析等功能，但我们可以通过此项目提供的 `API` 按照 `Rust` 的语法进行正则表达式的构建：

```Rust
use regex_fsa::regex::Regex;

// 构造 `a | b` 正则表达式
let a_or_b = "a".or("b");
// 构造 ab(a|b)*ba 正则表达式
let regex = "ab".and(a_or_b.many()).and("ba");
```

目前项目已经提供了最简单的匹配功能：`匹配输入字符串是否符合正则表达式`，通过构造`Matcher`即可使用：

```Rust
use regex_fsa::Matcher;

let matcher = Matcher::from_regex(regex);

let a = matcher.is_matched("abbbbaaaaba");
let b = matcher.is_matched("baaaaaaaba");
let c = matcher.is_matched("aabbbbbab");
println!("{} {} {}", a, b, c);
```

结果输出:

```
true false false
```

`Matcher`作为最简单的匹配功能，只能判断输入字符串是否匹配正则表达式。但这里能做的事情不仅如此，`Matcher`底层也是基于 `FSA` (有限状态自动机) ，通过 `FSA`，还能往上实现更多功能，如`词法分析器`。

## 原理

剥开 `Matcher` 的封装，其实里面主要会做以下几件事情。拿上面的正则表达式 `ab(a|b)*ba` 举例：

### 正则表达式到 NFA：Thompson 构造法
第一步首先将正则表达式转换为 `NFA` (非确定性有限自动机)，这里利用的是 `Thompson 构造法`，正则表达式 `ab(a|b)*ba` 由此将构造如下图所示的 `NFA`：

```Rust
// 通过 `Thompson 构造法` 构造 NFA
let nfa = regex.as_nfa();
```

![NFA](https://github.com/TangentW/regex-fsa/blob/894057457c65f4b8bf31ae035f0fa0a65925ea39/imgs/NFA.png)

### NFA 到 DFA：子集构造法
第二部将 `NFA` 转换成 `DFA` (确定性有限自动机)，使用的是 `子集构造法 (Subset Construction)`，构造出如下的 `DFA`：

```Rust
// 通过 `子集构造法` 构造 DFA
let dfa = nfa.as_dfa();
```

![DFA](https://github.com/TangentW/regex-fsa/blob/894057457c65f4b8bf31ae035f0fa0a65925ea39/imgs/DFA.png)

### DFA 最小化：Hopcroft 算法
使用 `Hopcroft 算法`，我们可以精简 `DFA`，使其状态数目最小化，最小化后的 `DFA`如下：

```Rust
// 通过 `Hopcroft 算法` 最小化 DFA
let dfa = dfa.minimize();
```

![M_DFA](https://github.com/TangentW/regex-fsa/blob/894057457c65f4b8bf31ae035f0fa0a65925ea39/imgs/M_DFA.png)

### 匹配
至此，`DFA` 最终构建完毕，可以进行字符串匹配了：

```Rust
/// 构建自动机符号
let symbols = "abaaabbba".chars().map(Symbol::Char);
/// 检查是否匹配
let is_matched = dfa
    /// 获取 DFA 经过所有符号后所到达的状态
    .end_of(symbols)
    /// 判断此时到达的状态是否为状态（可接受状态）
    .map(|s| s.borrow().acceptable())
    .unwrap_or_default();

println!("{}", is_matched);
```

### 总结

`Matcher` 可以理解为对以下代码的封装和运行：

其中 `Regex`、`NFA` 和 `DFA` 逻辑可在对应模块中找到。

```Rust
use regex_fsa::regex::Regex;
use regex_fsa::fsa::Symbol;

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
```

## TODO

- [ ] 支持更多正则表达式（如 `[a-z]`、非贪婪匹配等）
- [ ] 支持 Unicode 等字母表庞大的编码方式
- [ ] 基于 DFA 实现词法分析器
