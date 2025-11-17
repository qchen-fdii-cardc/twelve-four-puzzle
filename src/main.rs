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
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("log/24_game_log.txt")
        .expect("Failed to open log file");
    loop {
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
                // 按引用遍历，避免不必要的克隆
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

    // println!("Log file has been updated.");
}

/// 对给定的 4 张牌，返回所有可得到 24 的表达式。
///
/// 为了确保覆盖所有组合，先将牌转为 `f64` 并生成全排列，
/// 再对每一个排列调用 `find_solutions_for_permutation` 来遍历
/// 运算符与括号结构。使用 `HashSet` 避免重复表达式。
fn solve_24(cards: &[i32]) -> Vec<String> {
    let mut solutions = HashSet::new();
    let mut nums: Vec<f64> = cards.iter().map(|&x| x as f64).collect();
    let mut used = [false; 4];
    let mut p = Vec::new();
    generate_permutations(&mut nums, &mut used, &mut p, &mut |perm| {
        find_solutions_for_permutation(perm, &mut solutions);
    });
    solutions.into_iter().collect()
}

/// 一个简单的回溯函数，用于生成 `nums` 的所有排列，
/// 并在取得完整排列后调用回调函数。
fn generate_permutations(
    nums: &mut Vec<f64>,
    used: &mut [bool],
    p: &mut Vec<f64>,
    callback: &mut dyn FnMut(&[f64]),
) {
    if p.len() == nums.len() {
        callback(p);
        return;
    }
    for i in 0..nums.len() {
        if !used[i] {
            used[i] = true;
            p.push(nums[i]);
            generate_permutations(nums, used, p, callback);
            p.pop();
            used[i] = false;
        }
    }
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
fn find_solutions_for_permutation(perm: &[f64], solutions: &mut HashSet<String>) {
    let ops = ['+', '-', '*', '/'];
    for &op1 in &ops {
        for &op2 in &ops {
            for &op3 in &ops {
                // Structure: (a op1 b) op2 (c op3 d)
                if let Some(val1) = apply_op(perm[0], perm[1], op1) {
                    if let Some(val2) = apply_op(perm[2], perm[3], op3) {
                        if let Some(res) = apply_op(val1, val2, op2) {
                            if (res - TARGET).abs() < EPSILON {
                                solutions.insert(format!(
                                    "({} {} {}) {} ({} {} {})",
                                    perm[0], op1, perm[1], op2, perm[2], op3, perm[3]
                                ));
                            }
                        }
                    }
                }

                // Structure: ((a op1 b) op2 c) op3 d
                if let Some(val1) = apply_op(perm[0], perm[1], op1) {
                    if let Some(val2) = apply_op(val1, perm[2], op2) {
                        if let Some(res) = apply_op(val2, perm[3], op3) {
                            if (res - TARGET).abs() < EPSILON {
                                solutions.insert(format!(
                                    "(({} {} {}) {} {}) {} {}",
                                    perm[0], op1, perm[1], op2, perm[2], op3, perm[3]
                                ));
                            }
                        }
                    }
                }

                // Structure: a op1 (b op2 (c op3 d))
                if let Some(val1) = apply_op(perm[2], perm[3], op3) {
                    if let Some(val2) = apply_op(perm[1], val1, op2) {
                        if let Some(res) = apply_op(perm[0], val2, op1) {
                            if (res - TARGET).abs() < EPSILON {
                                solutions.insert(format!(
                                    "{} {} ({} {} ({} {} {}))",
                                    perm[0], op1, perm[1], op2, perm[2], op3, perm[3]
                                ));
                            }
                        }
                    }
                }

                // Structure: (a op1 (b op2 c)) op3 d
                if let Some(val1) = apply_op(perm[1], perm[2], op2) {
                    if let Some(val2) = apply_op(perm[0], val1, op1) {
                        if let Some(res) = apply_op(val2, perm[3], op3) {
                            if (res - TARGET).abs() < EPSILON {
                                solutions.insert(format!(
                                    "({} {} ({} {} {})) {} {}",
                                    perm[0], op1, perm[1], op2, perm[2], op3, perm[3]
                                ));
                            }
                        }
                    }
                }

                // Structure: a op1 ((b op2 c) op3 d)
                if let Some(val1) = apply_op(perm[1], perm[2], op2) {
                    if let Some(val2) = apply_op(val1, perm[3], op3) {
                        if let Some(res) = apply_op(perm[0], val2, op1) {
                            if (res - TARGET).abs() < EPSILON {
                                solutions.insert(format!(
                                    "{} {} (({} {} {}) {} {})",
                                    perm[0], op1, perm[1], op2, perm[2], op3, perm[3]
                                ));
                            }
                        }
                    }
                }
            }
        }
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
