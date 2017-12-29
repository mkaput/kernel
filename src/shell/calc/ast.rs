use alloc::{String, Vec};
use alloc::boxed::Box;

#[derive(PartialEq, Clone, Debug)]
pub enum Ast {
    Num(String),
    OpNode(Op, Box<Ast>, Box<Ast>),
}

impl Ast {
    pub fn eval(&self) -> Result<i64, String> {
        match *self {
            Ast::Num(ref s) => s.parse().map_err(|e| format!("not a number {}: {}", s, e)),
            Ast::OpNode(op, ref a, ref b) => op.eval(a.eval()?, b.eval()?),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    fn eval(&self, a: i64, b: i64) -> Result<i64, String> {
        match *self {
            Op::Add => i64::checked_add(a, b).ok_or("math error".into()),
            Op::Sub => i64::checked_sub(a, b).ok_or("math error".into()),
            Op::Mul => i64::checked_mul(a, b).ok_or("math error".into()),
            Op::Div => i64::checked_div(a, b).ok_or("math error".into()),
        }
    }
}

/*
 * Example: ([(1, +), (2, -)], 4) ==> (1 + 2) - 4
 * Note that va.0 is in reverse order.
 */
pub fn vecast_to_ast<T: Copy>(va: (Vec<(Ast, T)>, Ast), fold: impl Fn(T, Ast, Ast) -> Ast) -> Ast {
    let (mut x, y) = va;
    if x.len() == 0 {
        return y;
    }
    x.reverse();
    let mut ast = x[0].0.clone();
    for i in 0..x.len() {
        ast = fold(
            x[i].1,
            ast,
            if i == x.len() - 1 {
                y.clone()
            } else {
                x[i + 1].0.clone()
            },
        );
    }
    ast
}
