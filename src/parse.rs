use pest::iterators::Pair;
use pest::Parser;

use crate::ast::*;

#[derive(Parser)]
#[grammar = "coro.pest"]
struct PEGParser;

pub struct CoParser;

impl CoParser {
    pub fn parse(src: &str) -> Result<Ast, String> {
        let mut ast = Ast::new();
        let mut start = match PEGParser::parse(Rule::program, src) {
            Err(e) => return Err(format!("{}", e)),
            Ok(p) => p,
        };

        let program = start.next().unwrap();
        let iter = program.into_inner();
        for pair in iter {
            match pair.as_rule() {
                Rule::bind => ast.items.push(parse_bind(pair)?),
                Rule::EOI => break,
                _ => unreachable!(),
            }
        }
        Ok(ast)
    }
}

fn parse_bind(pair: Pair<Rule>) -> Result<Bind, String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::bind_def => Ok(Bind::Def(parse_def(inner)?)),
        Rule::bind_let => Ok(Bind::Let(parse_let(inner)?)),
        Rule::cmd => Ok(Bind::Cmd(parse_cmd(inner)?)),
        _ => unreachable!(),
    }
}

fn parse_def(pair: Pair<Rule>) -> Result<DefBind, String> {
    let mut pairs: Vec<Pair<Rule>> = pair.into_inner().collect();
    let name = String::from(pairs[0].as_str());

    let mut params = Vec::new();
    let num_params = pairs.len() - 2;
    for i in 1..(num_params + 1) {
        params.push(String::from(pairs[i].as_str()));
    }

    let last = pairs.pop().unwrap();
    let body = parse_cmd(last)?;

    Ok(DefBind::new(name, params, body))
}

fn parse_let(pair: Pair<Rule>) -> Result<LetBind, String> {
    let mut iter = pair.into_inner();
    let name = String::from(iter.next().unwrap().as_str());
    let init = parse_cmd(iter.next().unwrap())?;
    Ok(LetBind::new(name, init))
}

fn parse_cmd(pair: Pair<Rule>) -> Result<Cmd, String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::cmd_print => parse_print(inner),
        Rule::cmd_create => parse_create(inner),
        Rule::cmd_resume => parse_resume(inner),
        Rule::cmd_yield => parse_yield(inner),
        Rule::cmd_while => parse_while(inner),
        Rule::cmd_if => parse_if(inner),
        Rule::expr => Ok(Cmd::Expr(parse_expr(inner)?)),
        _ => unreachable!(),
    }
}

fn parse_print(pair: Pair<Rule>) -> Result<Cmd, String> {
    let inner = pair.into_inner().next().unwrap();
    let expr = parse_expr(inner)?;
    Ok(Cmd::Print(expr))
}

fn parse_create(pair: Pair<Rule>) -> Result<Cmd, String> {
    let inner = pair.into_inner().next().unwrap();
    let ident = String::from(inner.as_str());
    Ok(Cmd::Create(ident))
}

fn parse_resume(pair: Pair<Rule>) -> Result<Cmd, String> {
    let mut iter = pair.into_inner();
    let co = parse_expr(iter.next().unwrap())?;

    let mut args = Vec::new();
    for next in iter {
        args.push(parse_expr(next)?);
    }

    Ok(Cmd::Resume(co, args))
}

fn parse_yield(pair: Pair<Rule>) -> Result<Cmd, String> {
    let inner = pair.into_inner().next().unwrap();
    let expr = parse_expr(inner)?;
    Ok(Cmd::Yield(expr))
}

fn parse_while(pair: Pair<Rule>) -> Result<Cmd, String> {
    let mut iter = pair.into_inner();
    let expr = parse_expr(iter.next().unwrap())?;
    let body = parse_expr(iter.next().unwrap())?;
    Ok(Cmd::While(expr, body))
}

fn parse_if(pair: Pair<Rule>) -> Result<Cmd, String> {
    let mut iter = pair.into_inner();
    let cond = parse_expr(iter.next().unwrap())?;
    let then = parse_expr(iter.next().unwrap())?;
    let alt = parse_expr(iter.next().unwrap())?;
    Ok(Cmd::If(cond, then, alt))
}

fn parse_expr(pair: Pair<Rule>) -> Result<Expr, String> {
    let inner = pair.into_inner().next().unwrap();
    parse_relation(inner)
}

fn parse_relation(pair: Pair<Rule>) -> Result<Expr, String> {
    let mut iter = pair.into_inner();
    let mut expr = parse_term(iter.next().unwrap())?;
    if let Some(next) = iter.next() {
        let mut rhs_iter = next.into_inner();
        let op = rhs_iter.next().unwrap();
        let rhs = parse_term(rhs_iter.next().unwrap())?;
        match op.as_str() {
            "==" => expr = Expr::Eq(Box::new(expr), Box::new(rhs)),
            "<" => expr = Expr::Lt(Box::new(expr), Box::new(rhs)),
            _ => unreachable!(),
        }
    }
    Ok(expr)
}

fn parse_term(pair: Pair<Rule>) -> Result<Expr, String> {
    let mut iter = pair.into_inner();
    let mut expr = parse_factor(iter.next().unwrap())?;
    for next in iter {
        let mut rhs_iter = next.into_inner();
        let op = rhs_iter.next().unwrap();
        let rhs = parse_factor(rhs_iter.next().unwrap())?;
        match op.as_str() {
            "+" => expr = Expr::Add(Box::new(expr), Box::new(rhs)),
            "-" => expr = Expr::Sub(Box::new(expr), Box::new(rhs)),
            _ => unreachable!(),
        }
    }
    Ok(expr)
}

fn parse_factor(pair: Pair<Rule>) -> Result<Expr, String> {
    let mut iter = pair.into_inner();
    let mut expr = parse_unary(iter.next().unwrap())?;
    for next in iter {
        let mut rhs_iter = next.into_inner();
        let op = rhs_iter.next().unwrap();
        let rhs = parse_unary(rhs_iter.next().unwrap())?;
        match op.as_str() {
            "*" => expr = Expr::Mul(Box::new(expr), Box::new(rhs)),
            "/" => expr = Expr::Div(Box::new(expr), Box::new(rhs)),
            _ => unreachable!(),
        }
    }
    Ok(expr)
}

fn parse_unary(pair: Pair<Rule>) -> Result<Expr, String> {
    let mut iter = pair.into_inner();
    let lhs = iter.next().unwrap();
    if lhs.as_rule() == Rule::atom {
        return parse_atom(lhs);
    }

    let op = lhs;
    let rhs = iter.next().unwrap();
    let expr = parse_unary(rhs)?;
    match op.as_str() {
        "not" => Ok(Expr::Not(Box::new(expr))),
        "-" => Ok(Expr::Neg(Box::new(expr))),
        _ => unreachable!(),
    }
}

fn parse_atom(pair: Pair<Rule>) -> Result<Expr, String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::block => parse_block(inner),
        Rule::group => parse_group(inner),
        Rule::ident => parse_ident(inner),
        Rule::bool => Ok(Expr::Bool(inner.as_str() == "true")),
        Rule::num => {
            let res = inner.as_str().parse::<f64>();
            Ok(Expr::Num(res.unwrap()))
        }
        Rule::str => {
            let res = inner
                .as_str()
                .strip_prefix('"')
                .unwrap()
                .strip_suffix('"')
                .unwrap();
            Ok(Expr::Str(String::from(res)))
        }
        Rule::unit => Ok(Expr::Unit),
        _ => unreachable!(),
    }
}

fn parse_block(pair: Pair<Rule>) -> Result<Expr, String> {
    let mut binds = Vec::new();
    for next in pair.into_inner() {
        binds.push(parse_bind(next)?);
    }
    assert!(!binds.is_empty(), "block need to be non-empty");
    Ok(Expr::Block(binds))
}

fn parse_group(pair: Pair<Rule>) -> Result<Expr, String> {
    let inner = pair.into_inner().next().unwrap();
    let cmd = parse_cmd(inner)?;
    Ok(Expr::Group(Box::new(cmd)))
}

fn parse_ident(pair: Pair<Rule>) -> Result<Expr, String> {
    match pair.as_str() {
        "def" | "let" => Err(String::from("expected proper binding")),
        "print" | "create" | "resume" | "yield" | "while" | "do" | "if" | "then" | "else"
        | "end" => Err(String::from("expected proper command")),
        "true" | "false" => Err(String::from("expected proper expression")),
        name => Ok(Expr::Ident(String::from(name))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! ast_eq {
        ($src:expr, $expected:expr) => {
            let ast = CoParser::parse($src).unwrap();
            let dbg = format!("{:?}", ast);
            let exp = format!("Ast {{ items: [{}] }}", $expected);
            assert_eq!(dbg, exp);
        };
    }

    #[test]
    fn empty() {
        let src = "";
        let ast = CoParser::parse(src).unwrap();
        assert!(ast.items.is_empty());
    }

    #[test]
    fn blanks() {
        let src = "    \t  \t \r \n \r\n";
        let ast = CoParser::parse(src).unwrap();
        assert!(ast.items.is_empty());
    }

    #[test]
    fn comments() {
        let src = r##"
            # a comment
            # another line
        "##;
        let ast = CoParser::parse(src).unwrap();
        assert!(ast.items.is_empty());
    }

    #[test]
    fn atom_unit() {
        ast_eq!("()", "Cmd(Expr(Unit))");
    }

    #[test]
    fn atom_bool() {
        ast_eq!("true", "Cmd(Expr(Bool(true)))");
    }

    #[test]
    fn atom_num() {
        ast_eq!("3.14", "Cmd(Expr(Num(3.14)))");
    }

    #[test]
    fn atom_str() {
        ast_eq!(r#" "foo" "#, r#"Cmd(Expr(Str("foo")))"#);
    }

    #[test]
    fn atom_ident() {
        ast_eq!("_bar123", r#"Cmd(Expr(Ident("_bar123")))"#);
    }

    #[test]
    fn unary_negate() {
        ast_eq!("- - 2", "Cmd(Expr(Neg(Neg(Num(2.0)))))");
    }

    #[test]
    fn unary_not() {
        ast_eq!("not not true", "Cmd(Expr(Not(Not(Bool(true)))))");
    }

    #[test]
    fn binary_factor() {
        let src = "1 * 2 / 3";
        let exp = "Cmd(Expr(Div(Mul(Num(1.0), Num(2.0)), Num(3.0))))";
        ast_eq!(src, exp);
    }

    #[test]
    fn binary_term() {
        let src = "1 + 2 - 3";
        let exp = "Cmd(Expr(Sub(Add(Num(1.0), Num(2.0)), Num(3.0))))";
        ast_eq!(src, exp);
    }

    #[test]
    fn binary_relation() {
        let src = "1 == 2";
        let exp = "Cmd(Expr(Eq(Num(1.0), Num(2.0))))";
        ast_eq!(src, exp);
    }

    #[test]
    fn precedence() {
        let src = "1 + 2 / 3 - 4 < -5 * 6";
        let exp = "Cmd(Expr(Lt(Sub(Add(Num(1.0), \
            Div(Num(2.0), Num(3.0))), Num(4.0)), \
            Mul(Neg(Num(5.0)), Num(6.0)))))";
        ast_eq!(src, exp);
    }

    #[test]
    fn command_if() {
        let src = "if true then 1 else 2 end";
        let exp = "Cmd(If(Bool(true), Num(1.0), Num(2.0)))";
        ast_eq!(src, exp);
    }

    #[test]
    fn command_while() {
        let src = "while 1 < 2 do 3 end";
        let exp = "Cmd(While(Lt(Num(1.0), Num(2.0)), Num(3.0)))";
        ast_eq!(src, exp);
    }

    #[test]
    fn command_yield() {
        ast_eq!("yield 1", "Cmd(Yield(Num(1.0)))");
    }

    #[test]
    fn command_resume() {
        ast_eq!(
            "resume co 1 2",
            r#"Cmd(Resume(Ident("co"), [Num(1.0), Num(2.0)]))"#
        );
    }

    #[test]
    fn command_create() {
        ast_eq!("create foo", r#"Cmd(Create("foo"))"#);
    }

    #[test]
    fn command_print() {
        ast_eq!("print 1", "Cmd(Print(Num(1.0)))");
    }

    #[test]
    fn let_binding() {
        let src = "let a = true";
        let exp = "Let(LetBind { \
            name: \"a\", \
            init: Expr(Bool(true)) })";
        ast_eq!(src, exp);
    }

    #[test]
    fn define_binding() {
        let src = "def fn a b c = true";
        let exp = "Def(DefBind { \
            name: \"fn\", \
            params: [\"a\", \"b\", \"c\"], \
            body: Expr(Bool(true)) })";
        ast_eq!(src, exp);
    }

    #[test]
    fn group() {
        let src = "(1 + 2) * 3";
        let exp = "Cmd(Expr(Mul(Group(Expr(Add(Num(1.0), Num(2.0)))), Num(3.0))))";
        ast_eq!(src, exp);
    }

    #[test]
    fn block() {
        let src = "{ 1; 2; }";
        let exp = "Cmd(Expr(Block([Cmd(Expr(Num(1.0))), Cmd(Expr(Num(2.0)))])))";
        ast_eq!(src, exp);
    }

    #[test]
    fn block_semi_optional() {
        let src = "{ 1 }";
        let exp = "Cmd(Expr(Block([Cmd(Expr(Num(1.0)))])))";
        ast_eq!(src, exp);
    }

    #[test]
    #[should_panic]
    fn binary_relation_no_associativity() {
        let src = "1 == 2 < 3";
        CoParser::parse(src).unwrap();
    }

    #[test]
    #[should_panic]
    fn command_create_only_ident() {
        let src = "create (not an_ident)";
        CoParser::parse(src).unwrap();
    }

    #[test]
    #[should_panic]
    fn bad_input() {
        let src = "if true then missing_rest_of_if ";
        CoParser::parse(src).unwrap();
    }
}
