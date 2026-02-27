use crate::core::content;

/// Print the spec-driven development methodology reference and exit.
pub fn run_explain() -> i32 {
    println!("{}", content::methodology());
    0
}
