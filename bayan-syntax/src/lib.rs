pub mod ast;
pub mod diag;
pub mod parser;
pub mod pretty;
pub mod span;
pub mod token;

pub use ast::*;
pub use span::*;
pub use token::*;

pub fn parse_source(src: &str) -> ParseResult<Module> {
    let mut lexer = token::Lexer::new(src);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            return ParseResult::err(Diagnostic::new(
                "E0101",
                Span::none(),
                format!("lexer error: {}", e),
            ));
        }
    };
    let mut parser = parser::Parser::new(tokens);
    parser.parse_module()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_demo() {
        let src = r#"module demo
extern fn print_i64(v: i64)

fn sum_array(data: ptr, len: i64) -> i64 {
  var total: i64 = 0
  var i: i64 = 0
  loop {
    if i.ge.s(len) { break }
    let x: i32 = load.unaligned<i32>(data.add.wrap(i.mul.wrap(4)))
    total = total.add.wrap(sext<i64>(x))
    i = i.add.wrap(1)
  }
  return total
}
"#;
        let r = parse_source(src);
        assert!(!r.diags.has_errors(), "{:?}", r.diags);
        assert!(r.value.is_some());
        let m = r.value.unwrap();
        assert_eq!(m.name, "demo");
    }
}
