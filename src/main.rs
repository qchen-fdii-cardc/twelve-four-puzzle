//! # 程序说明
//!
//! 这是一个 24 点求解器：程序会随机抽取 4 张扑克牌（数值 1~13），
//! 使用加减乘除与所有括号组合来寻找得到 24 的表达式，
//! 并把 "有解" 或 "无解" 的结果写入 `log/24_game_log.txt` 日志。
//!
//! ## 算法完整性与正确性
//! - **完整性**：对 4 张牌进行全排列，共 4! = 24 种顺序；
//!   每一顺序都会尝试 3 个运算符位的所有 4^3 组合；
//!   同时覆盖五种合法的二叉树括号形态，等价于枚举所有四元表达式结构。
//!   因此任何合法的 24 点表达式必定会被枚举到。
//! - **正确性**：所有运算在 `f64` 中完成，并使用 `EPSILON` 进行浮点比较；
//!   除法在分母绝对值小于 `EPSILON` 时会被忽略以避免除以零。
//!   这些约束确保枚举到的表达式都是真实可计算且确实等于 24 的结果。

use chrono::Local;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;

const TARGET: f64 = 24.0;
const EPSILON: f64 = 1e-6;

/// 程序入口：抽牌、求解、并把结果写入日志。
///
/// 这里的流程是：
/// 1. 打开（或创建）日志文件并定位到末尾；
/// 2. 随机抽取 4 张牌；
/// 3. 调用 `solve_24` 获取所有表达式；
/// 4. 按时间戳记录抽到的牌和对应的所有解，若无解则写入提示。
fn main() {
    std::fs::create_dir_all("log").expect("Failed to create log directory");
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("log/24_game_log.txt")
        .expect("Failed to open log file");
    let mut cards = (1..=13).collect::<Vec<i32>>();
    let mut rng = thread_rng();
    cards.shuffle(&mut rng);
    let hand: Vec<i32> = cards.into_iter().take(4).collect();
    let solutions = solve_24(&hand);
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(log_file, "[{}] Cards: {:?}", timestamp, hand).unwrap();
    if solutions.is_empty() {
        writeln!(log_file, "No solution found.").unwrap();
    } else {
        writeln!(log_file, "Solutions:").unwrap();
        for s in &solutions {
            writeln!(log_file, "{}", s).unwrap();
        }
    }
    writeln!(log_file, "--------------------").unwrap();
    println!(
        "Processed hand: {:3?}.\t Solution count {:12}.",
        hand,
        solutions.len()
    );
}

/// 对给定的 4 张牌，返回所有可得到 24 的表达式。
fn solve_24(cards: &[i32]) -> Vec<String> {
    let nums: Vec<f64> = cards.iter().map(|&x| x as f64).collect();
    permutations(&nums)
        .into_iter()
        .fold(HashSet::new(), |mut acc, perm| {
            acc.extend(find_solutions_for_permutation(&perm));
            acc
        })
        .into_iter()
        .collect()
}

/// 返回所有排列。
fn permutations(nums: &[f64]) -> Vec<Vec<f64>> {
    if nums.is_empty() {
        return vec![vec![]];
    }
    nums.iter()
        .enumerate()
        .flat_map(|(i, &v)| {
            let rest: Vec<f64> = nums
                .iter()
                .enumerate()
                .filter_map(|(j, &x)| if i == j { None } else { Some(x) })
                .collect();
            permutations(&rest)
                .into_iter()
                .map(move |tail| std::iter::once(v).chain(tail).collect())
        })
        .collect()
}

/// 对固定顺序的 4 个数字，尝试所有运算符组合与 5 种括号结构。
fn find_solutions_for_permutation(perm: &[f64]) -> HashSet<String> {
    let mut solutions = HashSet::new();
    let ops = ['+', '-', '*', '/'];
    for &op1 in &ops {
        for &op2 in &ops {
            for &op3 in &ops {
                if let Some(s) = try_struct1(perm, op1, op2, op3) {
                    solutions.insert(s);
                }
                if let Some(s) = try_struct2(perm, op1, op2, op3) {
                    solutions.insert(s);
                }
                if let Some(s) = try_struct3(perm, op1, op2, op3) {
                    solutions.insert(s);
                }
                if let Some(s) = try_struct4(perm, op1, op2, op3) {
                    solutions.insert(s);
                }
                if let Some(s) = try_struct5(perm, op1, op2, op3) {
                    solutions.insert(s);
                }
            }
        }
    }
    solutions
}

macro_rules! resolve_op {
    (op1, $op1:expr, $op2:expr, $op3:expr) => {
        $op1
    };
    (op2, $op1:expr, $op2:expr, $op3:expr) => {
        $op2
    };
    (op3, $op1:expr, $op2:expr, $op3:expr) => {
        $op3
    };
}

macro_rules! calc {
    ($perm:expr, $op1:expr, $op2:expr, $op3:expr, a) => {
        Some(($perm[0], format!("{}", $perm[0])))
    };
    ($perm:expr, $op1:expr, $op2:expr, $op3:expr, b) => {
        Some(($perm[1], format!("{}", $perm[1])))
    };
    ($perm:expr, $op1:expr, $op2:expr, $op3:expr, c) => {
        Some(($perm[2], format!("{}", $perm[2])))
    };
    ($perm:expr, $op1:expr, $op2:expr, $op3:expr, d) => {
        Some(($perm[3], format!("{}", $perm[3])))
    };
    ($perm:expr, $op1:expr, $op2:expr, $op3:expr, ($op:ident $lhs:tt $rhs:tt)) => {{
        let left = calc!($perm, $op1, $op2, $op3, $lhs);
        let right = calc!($perm, $op1, $op2, $op3, $rhs);
        left.and_then(|(lv, ls)| {
            right.and_then(|(rv, rs)| {
                let op_char = resolve_op!($op, $op1, $op2, $op3);
                apply_op(lv, rv, op_char).map(|res| (res, format!("({} {} {})", ls, op_char, rs)))
            })
        })
    }};
}

macro_rules! define_try_struct {
    ($name:ident, $expr:tt) => {
        fn $name(perm: &[f64], op1: char, op2: char, op3: char) -> Option<String> {
            calc!(perm, op1, op2, op3, $expr).and_then(|(value, repr)| {
                if (value - TARGET).abs() < EPSILON {
                    Some(repr)
                } else {
                    None
                }
            })
        }
    };
}

define_try_struct!(
    try_struct1,
    (op2 (op1 a b) (op3 c d))
);
define_try_struct!(
    try_struct2,
    (op3 (op2 (op1 a b) c) d)
);
define_try_struct!(
    try_struct3,
    (op1 a (op2 b (op3 c d)))
);
define_try_struct!(
    try_struct4,
    (op3 (op1 a (op2 b c)) d)
);
define_try_struct!(
    try_struct5,
    (op1 a (op3 (op2 b c) d))
);

/// 运算封装。
fn apply_op(a: f64, b: f64, op: char) -> Option<f64> {
    match op {
        '+' => Some(a + b),
        '-' => Some(a - b),
        '*' => Some(a * b),
        '/' if b.abs() > EPSILON => Some(a / b),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_apply_op_basic() {
        assert_eq!(apply_op(2.0, 3.0, '+'), Some(5.0));
        assert_eq!(apply_op(5.0, 3.0, '-'), Some(2.0));
        assert_eq!(apply_op(4.0, 3.0, '*'), Some(12.0));
        assert_eq!(apply_op(8.0, 2.0, '/'), Some(4.0));
        assert_eq!(apply_op(1.0, 1e-9, '/'), None);
    }
    #[test]
    fn test_try_struct1_success_and_failure() {
        let perm = [6.0, 2.0, 3.0, 4.0];
        assert!(try_struct1(&perm, '*', '+', '*').is_some());
        assert!(try_struct1(&perm, '+', '+', '+').is_none());
    }
    #[test]
    fn test_try_struct2_success() {
        let perm = [2.0, 3.0, 4.0, 1.0];
        assert!(try_struct2(&perm, '*', '*', '*').is_some());
    }
    #[test]
    fn test_try_struct3_success() {
        let perm = [3.0, 2.0, 4.0, 1.0];
        assert!(try_struct3(&perm, '*', '*', '*').is_some());
    }
    #[test]
    fn test_try_struct4_success() {
        let perm = [2.0, 3.0, 4.0, 1.0];
        assert!(try_struct4(&perm, '*', '*', '*').is_some());
    }
    #[test]
    fn test_try_struct5_success() {
        let perm = [3.0, 2.0, 2.0, 2.0];
        assert!(try_struct5(&perm, '*', '*', '*').is_some());
    }
}
