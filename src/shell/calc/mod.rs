use alloc::String;
use core::str;

mod ast;
mod grammar;

pub fn eval(expr: &[u8]) -> Result<i64, String> {
    let expr_str = str::from_utf8(expr).map_err(|e| format!("{}", e))?;
    let ast = grammar::top_exp(expr_str).map_err(|e| format!("{}", e))?;
    ast.eval()
}
