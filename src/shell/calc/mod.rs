use alloc::String;
use core::str;

mod arithmetic;

use self::arithmetic::expression;

pub fn eval(expr: &[u8]) -> Result<i64, String> {
    expression(str::from_utf8(expr).map_err(|e| format!("{}", e))?).map_err(|e| format!("{}", e))
}
