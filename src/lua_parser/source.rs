use crate::error::ParserError;
use super::LuaChunk;
use super::header::{ChunkHeader, Endianness};
use super::ast::{Block, Stmt, Expr};
use super::compiler::Compiler;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{alpha1, alphanumeric1, char as parse_char, multispace0, digit1},
    combinator::{map, opt, recognize},
    sequence::{delimited, tuple},
    multi::{many0, separated_list0},
};

fn ws<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(multispace0, inner, multispace0)
}

fn identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(tuple((
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        ))),
        |s: &str| s.to_string(),
    )(input)
}

fn string_literal(input: &str) -> IResult<&str, Vec<u8>> {
    let double = delimited(parse_char('"'), take_until("\""), parse_char('"'));
    let single = delimited(parse_char('\''), take_until("'"), parse_char('\''));

    map(alt((double, single)), |s: &str| {
        let mut out = Vec::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                let mut num = String::new();
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_digit() {
                        num.push(n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if !num.is_empty() {
                    out.push(num.parse::<u8>().unwrap_or(0));
                } else if let Some(nc) = chars.next() {
                    out.push(nc as u8);
                }
            } else {
                out.push(c as u8);
            }
        }
        out
    })(input)
}

fn number(input: &str) -> IResult<&str, f64> {
    map(recognize(digit1), |s: &str| s.parse::<f64>().unwrap_or(0.0))(input)
}

fn expr(input: &str) -> IResult<&str, Expr> {
    let mut base = alt((
        map(string_literal, Expr::String),
        map(number, Expr::Number),
        map(identifier, Expr::Ident),
        map(parse_table, Expr::Table),
    ));

    // Simplistic binary op
    let (input, l) = base(input)?;
    if let Ok((input, _)) = ws(tag(".."))(input) {
        if let Ok((input, r)) = expr(input) {
            return Ok((input, Expr::BinOp("..".to_string(), Box::new(l), Box::new(r))));
        }
    }

    Ok((input, l))
}

fn parse_table(input: &str) -> IResult<&str, Vec<(Option<Expr>, Expr)>> {
    let (input, _) = ws(parse_char('{'))(input)?;

    let parse_entry = map(
        tuple((
            opt(delimited(ws(parse_char('[')), ws(expr), ws(parse_char(']')))),
            opt(ws(parse_char('='))),
            ws(expr),
        )),
        |(k, _, v)| (k, v)
    );

    let (input, entries) = separated_list0(ws(parse_char(',')), parse_entry)(input)?;
    let (input, _) = opt(ws(parse_char(',')))(input)?;
    let (input, _) = ws(parse_char('}'))(input)?;

    Ok((input, entries))
}

fn parse_local_decl(input: &str) -> IResult<&str, Stmt> {
    let (input, _) = tag("local ")(input)?;
    let (input, names) = separated_list0(ws(parse_char(',')), identifier)(input)?;

    let (input, eq) = opt(ws(parse_char('=')))(input)?;
    let (input, exprs) = if eq.is_some() {
        separated_list0(ws(parse_char(',')), ws(expr))(input)?
    } else {
        (input, Vec::new())
    };

    Ok((input, Stmt::LocalDecl(names, exprs)))
}

fn parse_assign(input: &str) -> IResult<&str, Stmt> {
    let (input, names) = separated_list0(ws(parse_char(',')), map(identifier, Expr::Ident))(input)?;
    let (input, _) = ws(parse_char('='))(input)?;
    let (input, exprs) = separated_list0(ws(parse_char(',')), ws(expr))(input)?;
    Ok((input, Stmt::Assign(names, exprs)))
}

fn parse_call(input: &str) -> IResult<&str, Stmt> {
    let (input, func) = identifier(input)?;
    let (input, _) = ws(parse_char('('))(input)?;
    let (input, args) = separated_list0(ws(parse_char(',')), ws(expr))(input)?;
    let (input, _) = ws(parse_char(')'))(input)?;
    Ok((input, Stmt::CallStmt(Expr::Call(Box::new(Expr::Ident(func)), args))))
}

fn parse_comment(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("--")(input)?;
    let (input, _) = take_while(|c| c != '\n')(input)?;
    Ok((input, ()))
}

// A lenient block parser that grabs statements it knows, and skips things it doesn't recognize
fn parse_lenient_block(input: &str) -> IResult<&str, Block> {
    let mut stmts = Vec::new();
    let mut current_input = input;

    while !current_input.is_empty() {
        let (next_input, _) = multispace0(current_input)?;
        current_input = next_input;
        if current_input.is_empty() { break; }

        if let Ok((next_input, _)) = parse_comment(current_input) {
            current_input = next_input;
            continue;
        }
        if let Ok((next_input, stmt)) = parse_local_decl(current_input) {
            stmts.push(stmt);
            current_input = next_input;
            continue;
        }
        if let Ok((next_input, stmt)) = parse_assign(current_input) {
            stmts.push(stmt);
            current_input = next_input;
            continue;
        }
        if let Ok((next_input, stmt)) = parse_call(current_input) {
            stmts.push(stmt);
            current_input = next_input;
            continue;
        }

        // Skip 1 char if we don't recognize it so we don't infinite loop
        let (next_input, _) = take_while(|c: char| c != '\n' && c != ' ' && c != ';')(current_input)?;
        if current_input == next_input { // if take_while matched 0 chars
             current_input = &current_input[1..];
        } else {
             current_input = next_input;
        }
    }

    Ok(("", Block(stmts)))
}

#[allow(missing_docs)]
pub fn parse_obfuscated_source(source: &str) -> Result<LuaChunk<'_>, ParserError> {
    let has_large_table = source.matches(',').count() >= 50 || source.contains("\\] = \"\\");
    let has_while_true = source.contains("while true do") || source.contains("repeat");
    let has_math_funcs = source.contains("bit32.bxor") || source.contains("string.char");

    if !has_large_table && !has_while_true && !has_math_funcs {
        return Err(ParserError::UnrecognizedFormat);
    }

    let (_, parsed_block) = parse_lenient_block(source).map_err(|_| ParserError::UnrecognizedFormat)?;

    let mut compiler = Compiler::new();
    compiler.compile_block(&parsed_block);
    let root_proto = compiler.finish();

    if root_proto.instructions.is_empty() || root_proto.instructions.len() == 1 { // only return
        return Err(ParserError::UnrecognizedFormat);
    }

    let header = ChunkHeader {
        version: 0x51,
        format: 0xFF,
        endianness: Endianness::Little,
        int_size: 4,
        size_t_size: 4,
        instruction_size: 4,
        number_size: 8,
        number_is_integral: false,
    };

    Ok(LuaChunk { header, root_proto })
}
