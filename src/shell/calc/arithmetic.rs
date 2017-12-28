use alloc::String;
use self::RuleResult::{Failed, Matched};
fn escape_default(s: &str) -> String {
    s.chars().flat_map(|c| c.escape_default()).collect()
}
fn char_range_at(s: &str, pos: usize) -> (char, usize) {
    let c = &s[pos..].chars().next().unwrap();
    let next_pos = pos + c.len_utf8();
    (*c, next_pos)
}
#[derive(Clone)]
enum RuleResult<T> {
    Matched(usize, T),
    Failed,
}
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub expected: ::hashmap_core::HashSet<&'static str>,
}
pub type ParseResult<T> = Result<T, ParseError>;
impl ::core::fmt::Display for ParseError {
    fn fmt(
        &self,
        fmt: &mut ::core::fmt::Formatter,
    ) -> ::core::result::Result<(), ::core::fmt::Error> {
        try!(write!(
            fmt,
            "error at {}:{}: expected ",
            self.line, self.column
        ))
;
        if self.expected.len() == 0 {
            try!(write!(fmt, "EOF"));
        } else if self.expected.len() == 1 {
            try!(write!(
                fmt,
                "`{}`",
                escape_default(self.expected.iter().next().unwrap())
            ));
        } else {
            let mut iter = self.expected.iter();
            try!(write!(
                fmt,
                "one of `{}`",
                escape_default(iter.next().unwrap())
            ));
            for elem in iter {
                try!(write!(fmt, ", `{}`", escape_default(elem)));
            }
        }
        Ok(())
    }
}
fn slice_eq(input: &str, state: &mut ParseState, pos: usize, m: &'static str) -> RuleResult<()> {
    #![inline(always)]
    #![allow(dead_code)]
    let l = m.len();
    if input.len() >= pos + l && &input.as_bytes()[pos..pos + l] == m.as_bytes() {
        Matched(pos + l, ())
    } else {
        state.mark_failure(pos, m)
    }
}
fn slice_eq_case_insensitive(
    input: &str,
    state: &mut ParseState,
    pos: usize,
    m: &'static str,
) -> RuleResult<()> {
    #![inline]
    #![allow(dead_code)]
    let mut used = 0usize;
    let mut input_iter = input[pos..].chars().flat_map(|x| x.to_uppercase());
    for m_char_upper in m.chars().flat_map(|x| x.to_uppercase()) {
        used += m_char_upper.len_utf8();
        let input_char_result = input_iter.next();
        if input_char_result.is_none() || input_char_result.unwrap() != m_char_upper {
            return state.mark_failure(pos, m);
        }
    }
    Matched(pos + used, ())
}
fn any_char(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<()> {
    #![inline]
    #![allow(dead_code)]
    if input.len() > pos {
        let (_, next) = char_range_at(input, pos);
        Matched(next, ())
    } else {
        state.mark_failure(pos, "<character>")
    }
}
fn pos_to_line(input: &str, pos: usize) -> (usize, usize) {
    let before = &input[..pos];
    let line = before.as_bytes().iter().filter(|&&c| c == b'\n').count() + 1;
    let col = before.chars().rev().take_while(|&c| c != '\n').count() + 1;
    (line, col)
}
impl<'input> ParseState<'input> {
    #[inline(never)]
    fn mark_failure_slow_path(&mut self, pos: usize, expected: &'static str) {
        if pos == self.max_err_pos {
            self.expected.insert(expected);
        }
    }
    #[inline(always)]
    fn mark_failure(&mut self, pos: usize, expected: &'static str) -> RuleResult<()> {
        if self.suppress_fail == 0 {
            if self.reparsing_on_error {
                self.mark_failure_slow_path(pos, expected);
            } else if pos > self.max_err_pos {
                self.max_err_pos = pos;
            }
        }
        Failed
    }
}
struct ParseState<'input> {
    max_err_pos: usize,
    suppress_fail: usize,
    reparsing_on_error: bool,
    expected: ::hashmap_core::HashSet<&'static str>,
    _phantom: ::core::marker::PhantomData<&'input ()>,
}
impl<'input> ParseState<'input> {
    fn new() -> ParseState<'input> {
        ParseState {
            max_err_pos: 0,
            suppress_fail: 0,
            reparsing_on_error: false,
            expected: ::hashmap_core::HashSet::new(),
            _phantom: ::core::marker::PhantomData,
        }
    }
}

fn __parse_expression<'input>(
    __input: &'input str,
    __state: &mut ParseState<'input>,
    __pos: usize,
) -> RuleResult<i64> {
    #![allow(non_snake_case, unused)]
    __parse_sum(__input, __state, __pos)
}

fn __parse_sum<'input>(
    __input: &'input str,
    __state: &mut ParseState<'input>,
    __pos: usize,
) -> RuleResult<i64> {
    #![allow(non_snake_case, unused)]
    {
        let __choice_res = {
            let __seq_res = __parse_product(__input, __state, __pos);
            match __seq_res {
                Matched(__pos, l) => {
                    let __seq_res = slice_eq(__input, __state, __pos, "+");
                    match __seq_res {
                        Matched(__pos, _) => {
                            let __seq_res = __parse_product(__input, __state, __pos);
                            match __seq_res {
                                Matched(__pos, r) => Matched(__pos, { l + r }),
                                Failed => Failed,
                            }
                        }
                        Failed => Failed,
                    }
                }
                Failed => Failed,
            }
        };
        match __choice_res {
            Matched(__pos, __value) => Matched(__pos, __value),
            Failed => __parse_product(__input, __state, __pos),
        }
    }
}

fn __parse_product<'input>(
    __input: &'input str,
    __state: &mut ParseState<'input>,
    __pos: usize,
) -> RuleResult<i64> {
    #![allow(non_snake_case, unused)]
    {
        let __choice_res = {
            let __seq_res = __parse_atom(__input, __state, __pos);
            match __seq_res {
                Matched(__pos, l) => {
                    let __seq_res = slice_eq(__input, __state, __pos, "*");
                    match __seq_res {
                        Matched(__pos, _) => {
                            let __seq_res = __parse_atom(__input, __state, __pos);
                            match __seq_res {
                                Matched(__pos, r) => Matched(__pos, { l * r }),
                                Failed => Failed,
                            }
                        }
                        Failed => Failed,
                    }
                }
                Failed => Failed,
            }
        };
        match __choice_res {
            Matched(__pos, __value) => Matched(__pos, __value),
            Failed => __parse_atom(__input, __state, __pos),
        }
    }
}

fn __parse_atom<'input>(
    __input: &'input str,
    __state: &mut ParseState<'input>,
    __pos: usize,
) -> RuleResult<i64> {
    #![allow(non_snake_case, unused)]
    {
        let __choice_res = __parse_number(__input, __state, __pos);
        match __choice_res {
            Matched(__pos, __value) => Matched(__pos, __value),
            Failed => {
                let __seq_res = slice_eq(__input, __state, __pos, "(");
                match __seq_res {
                    Matched(__pos, _) => {
                        let __seq_res = __parse_sum(__input, __state, __pos);
                        match __seq_res {
                            Matched(__pos, v) => {
                                let __seq_res = slice_eq(__input, __state, __pos, ")");
                                match __seq_res {
                                    Matched(__pos, _) => Matched(__pos, { v }),
                                    Failed => Failed,
                                }
                            }
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            }
        }
    }
}

fn __parse_number<'input>(
    __input: &'input str,
    __state: &mut ParseState<'input>,
    __pos: usize,
) -> RuleResult<i64> {
    #![allow(non_snake_case, unused)]
    {
        let __seq_res = {
            let str_start = __pos;
            match {
                let mut __repeat_pos = __pos;
                let mut __repeat_value = vec![];
                loop {
                    let __pos = __repeat_pos;
                    let __step_res = if __input.len() > __pos {
                        let (__ch, __next) = char_range_at(__input, __pos);
                        match __ch {
                            '0'...'9' => Matched(__next, ()),
                            _ => __state.mark_failure(__pos, "[0-9]"),
                        }
                    } else {
                        __state.mark_failure(__pos, "[0-9]")
                    };
                    match __step_res {
                        Matched(__newpos, __value) => {
                            __repeat_pos = __newpos;
                            __repeat_value.push(__value);
                        }
                        Failed => {
                            break;
                        }
                    }
                }
                if __repeat_value.len() >= 1 {
                    Matched(__repeat_pos, ())
                } else {
                    Failed
                }
            } {
                Matched(__newpos, _) => Matched(__newpos, &__input[str_start..__newpos]),
                Failed => Failed,
            }
        };
        match __seq_res {
            Matched(__pos, n) => Matched(__pos, { n.parse().unwrap() }),
            Failed => Failed,
        }
    }
}

pub fn expression<'input>(__input: &'input str) -> ParseResult<i64> {
    #![allow(non_snake_case, unused)]
    let mut __state = ParseState::new();
    match __parse_expression(__input, &mut __state, 0) {
        Matched(__pos, __value) => {
            if __pos == __input.len() {
                return Ok(__value);
            }
        }
        _ => (),
    }
    let __err_pos = __state.max_err_pos;
    __state = ParseState::new();
    __state.reparsing_on_error = true;
    __state.max_err_pos = __err_pos;
    __parse_expression(__input, &mut __state, 0);
    let (__line, __col) = pos_to_line(__input, __err_pos);
    Err(ParseError {
        line: __line,
        column: __col,
        offset: __err_pos,
        expected: __state.expected,
    })
}
