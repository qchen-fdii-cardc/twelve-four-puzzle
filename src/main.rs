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
    // Ensure the `log` directory exists so opening the file won't fail.
    std::fs::create_dir_all("log").expect("Failed to create log directory");

    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("log/24_game_log.txt")
        .expect("Failed to open log file");
    // Run a single hand (generate, solve, log) and then exit.
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

    // println!("Log file has been updated.");
}

/// 对给定的 4 张牌，返回所有可得到 24 的表达式。
///
/// 为了确保覆盖所有组合，先将牌转为 `f64` 并生成全排列，
/// 再对每一个排列调用 `find_solutions_for_permutation` 来遍历
/// 运算符与括号结构。使用 `HashSet` 避免重复表达式。
fn solve_24(cards: &[i32]) -> Vec<String> {
    let nums: Vec<f64> = cards.iter().map(|&x| x as f64).collect();

    let mut all_solutions = HashSet::new();
    for perm in permutations(&nums) {
        let sols = find_solutions_for_permutation(&perm);
        all_solutions.extend(sols);
    }
    all_solutions.into_iter().collect()
}

/// 返回 `nums` 的所有排列（每个排列为 `Vec<f64>`）。
///
/// 详细说明：
/// - 该函数以递归方式实现。对于非空输入，函数会枚举每个位置 `i` 作为当前头元素 `v`，
///   构造剩余元素 `rest`（去掉索引 `i` 的元素），递归计算 `rest` 的所有排列，
///   然后把 `v` 置于每个子排列的头部，得到完整排列列表。
/// - 基准情形：当 `nums` 为空时，返回 `vec![vec![]]`，即包含一个空排列，这样递归拼接时能正确回溯。
/// - 风格与性能：该实现是函数式的——不依赖外部可变状态或回调，返回新分配的数据结构，
///   因而易于理解与测试。其时间复杂度为 O(n! * n)，空间复杂度也为 O(n!)（因为要保存所有排列），
///   对本程序的 n=4 情形而言开销可忽略。
///
/// 示例：
/// ```rust
/// let perms = permutations(&[1.0, 2.0, 3.0]);
/// // `perms` 将包含 6 个排列：
/// // [1.0, 2.0, 3.0]
/// // [1.0, 3.0, 2.0]
/// // [2.0, 1.0, 3.0]
/// // [2.0, 3.0, 1.0]
/// // [3.0, 1.0, 2.0]
/// // [3.0, 2.0, 1.0]
/// ```
fn permutations(nums: &[f64]) -> Vec<Vec<f64>> {
    if nums.is_empty() {
        return vec![vec![]];
    }

    let mut result = Vec::new();
    for (i, &v) in nums.iter().enumerate() {
        let mut rest = nums.to_vec();
        rest.remove(i);
        for mut perm in permutations(&rest) {
            perm.insert(0, v);
            result.push(perm);
        }
    }

    result
}

/// 对固定顺序的 4 个数字，尝试所有运算符组合与 5 种括号结构。
///
/// 这 5 种形态对应所有不同的二叉树结构：
/// 1. `(a op b) op (c op d)`
/// 2. `((a op b) op c) op d`
/// 3. `a op (b op (c op d))`
/// 4. `(a op (b op c)) op d`
/// 5. `a op ((b op c) op d)`
///
/// 每个结构都严格按照计算顺序逐步调用 `apply_op`，当结果与 `TARGET`
/// 在 `EPSILON` 范围内相等时，即认为找到了一个正确解。
fn find_solutions_for_permutation(perm: &[f64]) -> HashSet<String> {
    let mut solutions = HashSet::new();
    let ops = ['+', '-', '*', '/'];
    for &op1 in &ops {
        for &op2 in &ops {
            for &op3 in &ops {
                // For each structure, call small pure helpers and insert any match.
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

// Each of the following functions represents one of the five parenthesization
// structures. They are pure (no mutation) and return an Option<String>
// describing the expression when it evaluates to TARGET.
fn try_struct1(perm: &[f64], op1: char, op2: char, op3: char) -> Option<String> {
    // (a op1 b) op2 (c op3 d)
    let first = apply_op(perm[0], perm[1], op1).unwrap_or_else(|| f64::NAN);
    let second = apply_op(perm[2], perm[3], op3).unwrap_or_else(|| f64::NAN);
    let result = apply_op(first, second, op2).unwrap_or_else(|| f64::NAN);
    if (result - TARGET).abs() < EPSILON {
        Some(format!(
            "({} {} {}) {} ({} {} {})",
            perm[0], op1, perm[1], op2, perm[2], op3, perm[3]
        ))
    } else {
        None
    }
}

fn try_struct2(perm: &[f64], op1: char, op2: char, op3: char) -> Option<String> {
    // ((a op1 b) op2 c) op3 d
    let first = apply_op(perm[0], perm[1], op1).unwrap_or_else(|| f64::NAN);
    let second = apply_op(first, perm[2], op2).unwrap_or_else(|| f64::NAN);
    let result = apply_op(second, perm[3], op3).unwrap_or_else(|| f64::NAN);
    if (result - TARGET).abs() < EPSILON {
        Some(format!(
            "(({} {} {}) {} {}) {} {}",
            perm[0], op1, perm[1], op2, perm[2], op3, perm[3]
        ))
    } else {
        None
    }
}

fn try_struct3(perm: &[f64], op1: char, op2: char, op3: char) -> Option<String> {
    // a op1 (b op2 (c op3 d))
    let first = apply_op(perm[2], perm[3], op3).unwrap_or_else(|| f64::NAN);
    let second = apply_op(perm[1], first, op2).unwrap_or_else(|| f64::NAN);
    let result = apply_op(perm[0], second, op1).unwrap_or_else(|| f64::NAN);
    if (result - TARGET).abs() < EPSILON {
        Some(format!(
            "{} {} ({} {} ({} {} {}))",
            perm[0], op1, perm[1], op2, perm[2], op3, perm[3]
        ))
    } else {
        None
    }
}

fn try_struct4(perm: &[f64], op1: char, op2: char, op3: char) -> Option<String> {
    // (a op1 (b op2 c)) op3 d
    let first = apply_op(perm[1], perm[2], op2).unwrap_or_else(|| f64::NAN);
    let second = apply_op(perm[0], first, op1).unwrap_or_else(|| f64::NAN);
    let result = apply_op(second, perm[3], op3).unwrap_or_else(|| f64::NAN);
    if (result - TARGET).abs() < EPSILON {
        Some(format!(
            "({} {} ({} {} {})) {} {}",
            perm[0], op1, perm[1], op2, perm[2], op3, perm[3]
        ))
    } else {
        None
    }
}
fn try_struct5(perm: &[f64], op1: char, op2: char, op3: char) -> Option<String> {
    // a op1 ((b op2 c) op3 d)
    let first = apply_op(perm[1], perm[2], op2).unwrap_or_else(|| f64::NAN);
    let second = apply_op(first, perm[3], op3).unwrap_or_else(|| f64::NAN);
    let result = apply_op(perm[0], second, op1).unwrap_or_else(|| f64::NAN);
    if (result - TARGET).abs() < EPSILON {
        Some(format!(
            "{} {} (({} {} {}) {} {})",
            perm[0], op1, perm[1], op2, perm[2], op3, perm[3]
        ))
    } else {
        None
    }
}

/// 尝试对两个操作数应用运算符，必要时拦截非法操作并返回 `None`。
///
/// - 加、减、乘总是有效；
/// - 除法在分母绝对值小于 `EPSILON` 时直接跳过，以避免除零和数值震荡；
/// - `None` 会在上层被忽略，从而保证算法的健壮性。
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
mod test_vec {
    #[test]
    fn arr_basic() {
        let arr = [10, 20, 30];
        assert_eq!(arr[0], 10);
        assert_eq!(arr[1], 20);
        assert_eq!(arr[2], 30);
        assert_eq!(arr.len(), 3);

        let arr2 = [&arr[..], &arr[1..=2]];

        println!("arr2: {:?}", arr2);

        let v = vec![10, 20, 30];
        assert_eq!(v[0], 10);
        assert_eq!(v[1], 20);
        assert_eq!(v[2], 30);
        assert_eq!(v.len(), 3);

        let mut v2 = Vec::new();
        v2.push(100);
        v2.push(200);
        assert_eq!(v2.len(), 2);
        assert_eq!(v2[0], 100);
        assert_eq!(v2[1], 200);
    }

    #[test]
    fn test_array_range_collect() {
        assert_eq!((3..=5), std::ops::RangeInclusive::new(3, 5));
        assert_eq!((1..2), std::ops::Range { start: 1, end: 2 });
        assert_eq!(3 + 4 + 5, (3..=5).sum());
        let arr = [0, 1, 2, 3, 4];
        assert_eq!(arr[..], [0, 1, 2, 3, 4]);
        assert_eq!(arr[..3], [0, 1, 2]);
        assert_eq!(arr[..=3], [0, 1, 2, 3]);
        assert_eq!(arr[1..], [1, 2, 3, 4]);
        assert_eq!(arr[1..3], [1, 2]);
        assert_eq!(arr[1..=3], [1, 2, 3]); // This is a `RangeInclusive`
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
        // division by (near) zero should return None
        assert_eq!(apply_op(1.0, 1e-9, '/'), None);
    }

    #[test]
    fn test_permutations_count() {
        let nums = vec![1.0, 2.0, 3.0, 4.0];
        let perms = permutations(&nums);
        println!("Generated permutations: {:?}", perms);
        assert_eq!(perms.len(), 24); // 4! = 24
        let unique_perms: HashSet<_> = perms
            .into_iter()
            .map(|p| p.iter().map(|&f| f.to_bits()).collect::<Vec<u64>>())
            .collect();
        assert_eq!(unique_perms.len(), 24); // all should be unique
    }

    #[test]
    fn test_permutations_count_repeated() {
        let nums = vec![1.0, 2.0, 2.0];
        let perms = permutations(&nums);
        print!("Generated permutations with repeats: {:?}", perms);
        assert_eq!(perms.len(), 6); // 3! / 2! = 3
        let unique_perms: HashSet<_> = perms
            .into_iter()
            .map(|p| p.iter().map(|&f| f.to_bits()).collect::<Vec<u64>>())
            .collect();
        assert_eq!(unique_perms.len(), 3); // only 3 unique
    }

    #[test]
    fn test_try_struct1_success_and_failure() {
        let perm = [6.0, 2.0, 3.0, 4.0];
        // (6 * 2) + (3 * 4) == 24
        assert!(try_struct1(&perm, '*', '+', '*').is_some());
        println!(
            "Found expression: {}",
            try_struct1(&perm, '*', '+', '*').unwrap()
        );
        // wrong ops shouldn't match
        assert!(try_struct1(&perm, '+', '+', '+').is_none());
    }

    #[test]
    fn test_try_struct2_success() {
        let perm = [2.0, 3.0, 4.0, 1.0];
        // ((2 * 3) * 4) * 1 == 24
        assert!(try_struct2(&perm, '*', '*', '*').is_some());
    }

    #[test]
    fn test_try_struct3_success() {
        let perm = [3.0, 2.0, 4.0, 1.0];
        // 3 * (2 * (4 * 1)) == 24
        assert!(try_struct3(&perm, '*', '*', '*').is_some());
    }

    #[test]
    fn test_try_struct4_success() {
        let perm = [2.0, 3.0, 4.0, 1.0];
        // (2 * (3 * 4)) * 1 == 24
        assert!(try_struct4(&perm, '*', '*', '*').is_some());
    }

    #[test]
    fn test_try_struct5_success() {
        let perm = [3.0, 2.0, 2.0, 2.0];
        // 3 * ((2 * 2) * 2) == 24
        assert!(try_struct5(&perm, '*', '*', '*').is_some());
    }
}
