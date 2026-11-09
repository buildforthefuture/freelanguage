use crate::ast::*;
use crate::{Diagnostic, Diagnostics, ParseResult, Span};
use crate::token::*;
use crate::token::IntSuffix;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    diags: Diagnostics,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0, diags: Diagnostics::default() }
    }

    fn peek(&self) -> &Token {
        if self.pos >= self.tokens.len() {
            &self.tokens[self.tokens.len() - 1]
        } else {
            &self.tokens[self.pos]
        }
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, Diagnostic> {
        let tok = self.peek().clone();
        if tok.kind == kind {
            Ok(self.advance())
        } else {
            Err(Diagnostic::new(
                "E0105",
                tok.span,
                format!("expected {}, found {}", kind, tok.kind),
            ))
        }
    }

    fn expect_semicolon(&mut self) -> Result<(), Diagnostic> {
        if matches!(self.peek_kind(), TokenKind::Semicolon) {
            self.advance();
            Ok(())
        } else {
            Err(Diagnostic::new(
                "E0105",
                self.peek().span,
                "expected ';'".to_string(),
            ))
        }
    }

    pub fn parse_module(&mut self) -> ParseResult<Module> {
        let module_tok = match self.expect(TokenKind::Module) {
            Ok(t) => t,
            Err(d) => return ParseResult::err(d),
        };
        let name_tok = self.advance(); // the ident after module
        let name = match &name_tok.kind {
            TokenKind::Ident(s) => s.clone(),
            _ => {
                return ParseResult::err(Diagnostic::new(
                    "E0105",
                    name_tok.span,
                    "expected module name".to_string(),
                ));
            }
        };
        let mut items = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::Eof) {
            if let Some(item) = self.parse_top_level() {
                items.push(item);
            } else {
                break;
            }
        }
        let eof = self.advance();
        let span = Span::merge(module_tok.span, eof.span);
        ParseResult::ok(Module { name, span, items })
    }

    fn parse_top_level(&mut self) -> Option<Item> {
        let tok = self.peek_kind();
        match tok {
            TokenKind::Struct => Some(Item::Struct(self.parse_struct())),
            TokenKind::Extern => Some(Item::ExternFn(self.parse_extern_fn())),
            TokenKind::Export => {
                self.advance();
                if matches!(self.peek_kind(), TokenKind::Fn) {
                    let mut f = self.parse_fn_decl_inner();
                    f.is_export = true;
                    Some(Item::Fn(f))
                } else {
                    self.diags.add(Diagnostic::new("E0105", self.peek().span, "expected 'fn' after 'export'"));
                    None
                }
            }
            TokenKind::Fn => Some(Item::Fn(self.parse_fn_decl_inner())),
            TokenKind::Global => Some(Item::Global(self.parse_global())),
            _ => {
                self.diags.add(Diagnostic::new("E0105", self.peek().span.clone(), "expected top-level item"));
                self.advance();
                None
            }
        }
    }

    fn parse_struct(&mut self) -> StructDecl {
        let kw = self.advance(); // struct
        let name_tok = self.advance();
        let name = match &name_tok.kind { TokenKind::Ident(s) => s.clone(), _ => String::new() };
        self.expect(TokenKind::LBrace).ok();
        let mut fields = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
            let fname_tok = self.advance();
            let fname = match &fname_tok.kind { TokenKind::Ident(s) => s.clone(), _ => String::new() };
            self.expect(TokenKind::Colon).ok();
            let ty = self.parse_type();
            fields.push(StructField { name: fname, ty, span: fname_tok.span });
            if matches!(self.peek_kind(), TokenKind::Comma) { self.advance(); }
        }
        self.expect(TokenKind::RBrace).ok();
        let mut align = None;
        if matches!(self.peek_kind(), TokenKind::Align) {
            self.advance();
            self.expect(TokenKind::LParen).ok();
            let al = self.expect(TokenKind::IntLit(0, IntSuffix::None));
            if let Ok(t) = al { if let TokenKind::IntLit(v, _) = t.kind { align = Some(v); } }
            self.expect(TokenKind::RParen).ok();
        }
        StructDecl { name, span: Span::merge(kw.span, self.peek().span), fields, align }
    }

    fn parse_fn_decl_inner(&mut self) -> FnDecl {
        let kw = self.advance(); // fn
        let name_tok = self.advance();
        let name = match &name_tok.kind { TokenKind::Ident(s) => s.clone(), _ => String::new() };
        self.expect(TokenKind::LParen).ok();
        let mut params = Vec::new();
        if !matches!(self.peek_kind(), TokenKind::RParen) {
            loop {
                let pname_tok = self.advance();
                let pname = match &pname_tok.kind { TokenKind::Ident(s) => s.clone(), _ => String::new() };
                self.expect(TokenKind::Colon).ok();
                let ty = self.parse_type();
                params.push(Param { name: pname, ty, span: pname_tok.span });
                if !matches!(self.peek_kind(), TokenKind::Comma) { break; }
                self.advance();
            }
        }
        self.expect(TokenKind::RParen).ok();
        self.expect(TokenKind::Arrow).ok();
        let ret = self.parse_type();
        let attrs = self.parse_attrs();
        let body = self.parse_block();
        FnDecl { name, span: Span::merge(kw.span, body.span), params, ret, attrs, body, is_export: false, is_extern: false }
    }

    fn parse_extern_fn(&mut self) -> ExternFnDecl {
        self.advance(); // extern
        self.expect(TokenKind::Fn).ok();
        let name_tok = self.advance();
        let name = match &name_tok.kind { TokenKind::Ident(s) => s.clone(), _ => String::new() };
        self.expect(TokenKind::LParen).ok();
        let mut params = Vec::new();
        if !matches!(self.peek_kind(), TokenKind::RParen) {
            loop {
                let pname_tok = self.advance();
                let pname = match &pname_tok.kind { TokenKind::Ident(s) => s.clone(), _ => String::new() };
                self.expect(TokenKind::Colon).ok();
                let ty = self.parse_type();
                params.push(Param { name: pname, ty, span: pname_tok.span });
                if !matches!(self.peek_kind(), TokenKind::Comma) { break; }
                self.advance();
            }
        }
        self.expect(TokenKind::RParen).ok();
        self.expect(TokenKind::Arrow).ok();
        let ret = self.parse_type();
        let attrs = self.parse_attrs();
        ExternFnDecl { name, span: Span::none(), params, ret, attrs }
    }

    fn parse_global(&mut self) -> GlobalDecl {
        self.advance(); // global
        let name_tok = self.advance();
        let name = match &name_tok.kind { TokenKind::Ident(s) => s.clone(), _ => String::new() };
        self.expect(TokenKind::Colon).ok();
        let ty = self.parse_type();
        self.expect(TokenKind::Equals).ok();
        let init = self.parse_const_expr();
        let attrs = self.parse_attrs();
        self.expect_semicolon().ok();
        GlobalDecl { name, span: Span::none(), ty, init, attrs }
    }

    fn parse_type(&mut self) -> Type {
        let tok = self.advance();
        match &tok.kind {
            TokenKind::Ident(s) => match s.as_str() {
                "i8" => Type::I8,
                "i16" => Type::I16,
                "i32" => Type::I32,
                "i64" => Type::I64,
                "bool" => Type::Bool,
                "f32" => Type::F32,
                "f64" => Type::F64,
                "ptr" => Type::Ptr,
                "unit" => Type::Unit,
                name => {
                    // could be array [T; N]
                    if matches!(self.peek_kind(), TokenKind::LBracket) {
                        self.advance(); // [
                        let inner = Type::Struct(name.to_string());
                        self.expect(TokenKind::Semicolon).ok();
                        let n_tok = self.expect(TokenKind::IntLit(0, IntSuffix::None)).ok();
                        let n = match n_tok.map(|t| t.kind) { Some(TokenKind::IntLit(v,_)) => v, _ => 0 };
                        self.expect(TokenKind::RBracket).ok();
                        Type::Array(Box::new(inner), n)
                    } else {
                        Type::Struct(name.to_string())
                    }
                }
            },
            _ => Type::Unit,
        }
    }

    fn parse_attrs(&mut self) -> Vec<Attr> {
        let mut attrs = Vec::new();
        while matches!(self.peek_kind(), TokenKind::LBracket) {
            self.advance(); // [
            let name_tok = self.advance();
            let name = match &name_tok.kind { TokenKind::Ident(s) => s.clone(), _ => String::new() };
            let mut value = None;
            if matches!(self.peek_kind(), TokenKind::LParen) {
                self.advance(); // (
                let vtok = self.advance();
                value = match &vtok.kind {
                    TokenKind::Ident(s) => Some(AttrValue::Ident(s.clone())),
                    TokenKind::IntLit(v, _) => Some(AttrValue::Int(*v)),
                    _ => None,
                };
                self.expect(TokenKind::RParen).ok();
            }
            self.expect(TokenKind::RBracket).ok();
            attrs.push(Attr { name, value, span: name_tok.span });
        }
        attrs
    }

    fn parse_block(&mut self) -> Block {
        let lbrace = self.expect(TokenKind::LBrace).ok();
        let mut stmts = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
            if let Some(s) = self.parse_stmt() {
                stmts.push(s);
            } else {
                break;
            }
        }
        let rbrace = self.expect(TokenKind::RBrace).ok();
        let span = Span::merge(lbrace.map(|t| t.span).unwrap_or_default(), rbrace.map(|t| t.span).unwrap_or_default());
        Block { span, stmts }
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        let tok = self.peek_kind();
        match tok {
            TokenKind::Var => {
                self.advance();
                self.parse_var_decl(false)
            }
            TokenKind::Let => {
                self.advance();
                self.parse_var_decl(true)
            }
            TokenKind::If => Some(Stmt::If(self.parse_if())),
            TokenKind::Loop => Some(Stmt::Loop(self.parse_loop())),
            TokenKind::Switch => Some(Stmt::Switch(self.parse_switch())),
            TokenKind::Return => {
                self.advance();
                let expr = if matches!(self.peek_kind(), TokenKind::Semicolon | TokenKind::RBrace) { None } else { Some(self.parse_expr()) };
                self.expect_semicolon().ok();
                Some(Stmt::Return(expr, self.prev_span()))
            }
            TokenKind::Trap => {
                self.advance();
                self.expect(TokenKind::LParen).ok();
                let code_tok = self.expect(TokenKind::IntLit(0, IntSuffix::None)).ok();
                let code = match code_tok.map(|t| t.kind) { Some(TokenKind::IntLit(v,_)) => v, _ => 0 };
                self.expect(TokenKind::RParen).ok();
                self.expect_semicolon().ok();
                Some(Stmt::Trap(code, self.prev_span()))
            }
            TokenKind::Goto => {
                self.advance();
                let label_tok = self.advance();
                let label = match &label_tok.kind { TokenKind::Ident(s) => s.clone(), _ => String::new() };
                self.expect_semicolon().ok();
                Some(Stmt::Goto(label, label_tok.span))
            }
            TokenKind::Unsafe => {
                self.advance();
                self.expect(TokenKind::LBrace).ok();
                let mut stmts = Vec::new();
                while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
                    if let Some(s) = self.parse_stmt() { stmts.push(s); } else { break; }
                }
                self.expect(TokenKind::RBrace).ok();
                Some(Stmt::Unsafe(Block { span: Span::none(), stmts }))
            }
            TokenKind::Asm => Some(Stmt::Asm(self.parse_asm())),
            TokenKind::Break => {
                self.advance();
                let expr = if matches!(self.peek_kind(), TokenKind::Semicolon) { None } else { Some(self.parse_expr()) };
                self.expect_semicolon().ok();
                Some(Stmt::Break(expr, self.prev_span()))
            }
            TokenKind::Continue => {
                self.advance();
                self.expect_semicolon().ok();
                Some(Stmt::Continue(self.prev_span()))
            }
            TokenKind::Ident(_) => {
                // could be labeled block: ident : { ... } or assignment or expr stmt
                // look ahead: ident : (not =) -> labeled block? Actually goto target labels are ident:
                // Let's check: ident : { => labeled block. ident = => assign. ident : not followed by { => ?
                // For simplicity: if ident followed by : and then { => labeled block; else expression statement (could be ident = ...)
                let saved = self.pos;
                let ident_tok = self.advance();
                if matches!(self.peek_kind(), TokenKind::Colon) {
                    // peek next after colon
                    let next_pos = self.pos + 1;
                    let is_block = next_pos < self.tokens.len() && matches!(self.tokens[next_pos].kind, TokenKind::LBrace);
                    if is_block {
                        self.advance(); // :
                        let label = match &ident_tok.kind { TokenKind::Ident(s) => s.clone(), _ => String::new() };
                        let body = self.parse_block();
                        return Some(Stmt::LabeledBlock(label, body));
                    }
                }
                // not a labeled block; rewind and parse as expression/assignment
                self.pos = saved;
                let expr = self.parse_expr();
                // check if it's a simple assignment ident = expr
                if let Expr::Ident(name, _) = &expr {
                    if matches!(self.peek_kind(), TokenKind::Equals) {
                        self.advance();
                        let value = self.parse_expr();
                        self.expect_semicolon().ok();
                        return Some(Stmt::Assign(Assign { name: name.clone(), value, span: expr.span() }));
                    }
                }
                self.expect_semicolon().ok();
                Some(Stmt::ExprStmt(expr))
            }
            _ => {
                let expr = self.parse_expr();
                self.expect_semicolon().ok();
                Some(Stmt::ExprStmt(expr))
            }
        }
    }

    fn prev_span(&self) -> Span {
        if self.pos == 0 { Span::none() } else { self.tokens[self.pos - 1].span }
    }

    fn parse_var_decl(&mut self, is_let: bool) -> Option<Stmt> {
        let name_tok = self.advance();
        let name = match &name_tok.kind { TokenKind::Ident(s) => s.clone(), _ => String::new() };
        self.expect(TokenKind::Colon).ok();
        let ty = self.parse_type();
        self.expect(TokenKind::Equals).ok();
        let init = self.parse_expr();
        self.expect_semicolon().ok();
        let d = VarDecl { name, ty, init, span: name_tok.span, is_let };
        if is_let { Some(Stmt::LetDecl(d)) } else { Some(Stmt::VarDecl(d)) }
    }

    fn parse_if(&mut self) -> IfStmt {
        self.advance(); // if
        let cond = self.parse_expr();
        let then_block = self.parse_block();
        let else_block = if matches!(self.peek_kind(), TokenKind::Else) {
            self.advance();
            Some(self.parse_block())
        } else { None };
        IfStmt { cond, then_block, else_block, span: Span::none() }
    }

    fn parse_loop(&mut self) -> LoopStmt {
        self.advance(); // loop
        let body = self.parse_block();
        LoopStmt { body, span: Span::none() }
    }

    fn parse_switch(&mut self) -> SwitchStmt {
        self.advance(); // switch
        let discr = self.parse_expr();
        self.expect(TokenKind::LBrace).ok();
        let mut cases = Vec::new();
        let mut default = None;
        while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
            if matches!(self.peek_kind(), TokenKind::Default) {
                self.advance();
                self.expect(TokenKind::FatArrow).ok();
                let body = self.parse_block();
                default = Some(body);
            } else {
                self.expect(TokenKind::Case).ok();
                let val = self.parse_const_expr();
                self.expect(TokenKind::FatArrow).ok();
                let body = self.parse_block();
                cases.push((val, body));
            }
        }
        self.expect(TokenKind::RBrace).ok();
        SwitchStmt { discr, cases, default, span: Span::none() }
    }

    fn parse_asm(&mut self) -> AsmStmt {
        self.advance(); // asm
        self.expect(TokenKind::Volatile).ok();
        self.expect(TokenKind::LParen).ok();
        let template_tok = self.expect(TokenKind::StringLit(String::new())).ok();
        let template = match template_tok.map(|t| t.kind) { Some(TokenKind::StringLit(s)) => s, _ => String::new() };
        self.expect(TokenKind::Colon).ok();
        let mut outs = Vec::new();
        if !matches!(self.peek_kind(), TokenKind::Colon) {
            loop {
                let oname_tok = self.advance();
                let oname = match &oname_tok.kind { TokenKind::Ident(s) => s.clone(), _ => String::new() };
                self.expect(TokenKind::Colon).ok();
                let oty = self.parse_type();
                outs.push((oname, oty));
                if !matches!(self.peek_kind(), TokenKind::Comma) { break; }
                self.advance();
            }
        }
        self.expect(TokenKind::Colon).ok();
        let mut ins = Vec::new();
        if !matches!(self.peek_kind(), TokenKind::Colon) {
            loop {
                let expr = self.parse_expr();
                self.expect(TokenKind::Colon).ok();
                let ity = self.parse_type();
                ins.push((expr, ity));
                if !matches!(self.peek_kind(), TokenKind::Comma) { break; }
                self.advance();
            }
        }
        self.expect(TokenKind::Colon).ok();
        let mut clobbers = Vec::new();
        if !matches!(self.peek_kind(), TokenKind::RParen) {
            loop {
                let ctok = self.expect(TokenKind::StringLit(String::new())).ok();
                let c = match ctok.map(|t| t.kind) { Some(TokenKind::StringLit(s)) => s, _ => String::new() };
                clobbers.push(c);
                if !matches!(self.peek_kind(), TokenKind::Comma) { break; }
                self.advance();
            }
        }
        self.expect(TokenKind::RParen).ok();
        self.expect_semicolon().ok();
        AsmStmt { volatile: true, template, outs, ins, clobbers, span: Span::none() }
    }

    // ---------- expressions ----------

    fn parse_expr(&mut self) -> Expr {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Expr {
        let mut left = self.parse_logical_and();
        while matches!(self.peek_kind(), TokenKind::Or) {
            let op = self.advance();
            let right = self.parse_logical_and();
            let right_span = right.span();
            left = Expr::Binary(BinaryOp::OrLogical, Box::new(left), Box::new(right), Span::merge(op.span, right_span));
        }
        left
    }

    fn parse_logical_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while matches!(self.peek_kind(), TokenKind::And) {
            let op = self.advance();
            let right = self.parse_equality();
            let right_span = right.span();
            left = Expr::Binary(BinaryOp::AndLogical, Box::new(left), Box::new(right), Span::merge(op.span, right_span));
        }
        left
    }

    fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_relational();
        while matches!(self.peek_kind(), TokenKind::Eq | TokenKind::Ne) {
            let op = self.advance();
            let bo = match op.kind { TokenKind::Eq => BinaryOp::Eq, TokenKind::Ne => BinaryOp::Ne, _ => BinaryOp::Eq };
            let right = self.parse_relational();
            let right_span = right.span();
            left = Expr::Binary(bo, Box::new(left), Box::new(right), Span::merge(op.span, right_span));
        }
        left
    }

    fn parse_relational(&mut self) -> Expr {
        let mut left = self.parse_additive();
        while matches!(self.peek_kind(), TokenKind::LtS | TokenKind::LtU | TokenKind::LeS | TokenKind::LeU | TokenKind::GtS | TokenKind::GtU | TokenKind::GeS | TokenKind::GeU) {
            let op = self.advance();
            let bo = match op.kind {
                TokenKind::LtS => BinaryOp::LtS, TokenKind::LtU => BinaryOp::LtU,
                TokenKind::LeS => BinaryOp::LeS, TokenKind::LeU => BinaryOp::LeU,
                TokenKind::GtS => BinaryOp::GtS, TokenKind::GtU => BinaryOp::GtU,
                TokenKind::GeS => BinaryOp::GeS, TokenKind::GeU => BinaryOp::GeU,
                _ => BinaryOp::LtS,
            };
            let right = self.parse_additive();
            let right_span = right.span();
            left = Expr::Binary(bo, Box::new(left), Box::new(right), Span::merge(op.span, right_span));
        }
        left
    }

    fn parse_additive(&mut self) -> Expr {
        let mut left = self.parse_multiplicative();
        while matches!(self.peek_kind(), TokenKind::Plus | TokenKind::Minus) {
            let op = self.advance();
            let bo = match op.kind { TokenKind::Plus => BinaryOp::AddWrap, TokenKind::Minus => BinaryOp::SubWrap, _ => BinaryOp::AddWrap };
            let right = self.parse_multiplicative();
            let right_span = right.span();
            left = Expr::Binary(bo, Box::new(left), Box::new(right), Span::merge(op.span, right_span));
        }
        left
    }

    fn parse_multiplicative(&mut self) -> Expr {
        let mut left = self.parse_cast();
        while matches!(self.peek_kind(), TokenKind::Star | TokenKind::Slash | TokenKind::Percent) {
            let op = self.advance();
            let bo = match op.kind {
                TokenKind::Star => BinaryOp::MulWrap,
                TokenKind::Slash => BinaryOp::DivS,
                TokenKind::Percent => BinaryOp::RemS,
                _ => BinaryOp::MulWrap,
            };
            let right = self.parse_cast();
            let right_span = right.span();
            left = Expr::Binary(bo, Box::new(left), Box::new(right), Span::merge(op.span, right_span));
        }
        left
    }

    fn parse_cast(&mut self) -> Expr {
        let inner = self.parse_unary();
        // cast: postfix <type> applied? Actually casts are CastOp<T>(expr) prefix style in examples: sext<i64>(x)
        // Those are parsed as method calls / function calls. Here we just return inner.
        inner
    }

    fn parse_unary(&mut self) -> Expr {
        if matches!(self.peek_kind(), TokenKind::Minus | TokenKind::Not) {
            let op = self.advance();
            let uo = match op.kind { TokenKind::Minus => UnaryOp::Neg, TokenKind::Not => UnaryOp::Not, _ => UnaryOp::Neg };
            let inner = self.parse_unary();
            let inner_span = inner.span();
            return Expr::Unary(uo, Box::new(inner), Span::merge(op.span, inner_span));
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Expr {
        let mut base = self.parse_primary();
        loop {
            if matches!(self.peek_kind(), TokenKind::Dot) {
                // Could be method call x.op(args), field access x.f, or indexed
                self.advance(); // .
                // consume dotted chain of idents
                let mut chain = Vec::new();
                loop {
                    let id_tok = self.advance();
                    match &id_tok.kind {
                        TokenKind::Ident(s) => chain.push(s.clone()),
                        _ => break,
                    }
                    if !matches!(self.peek_kind(), TokenKind::Dot) { break; }
                    self.advance(); // .
                }
                if matches!(self.peek_kind(), TokenKind::LParen) {
                    // method call: op = chain joined by "."
                    self.advance(); // (
                    let op = chain.join(".");
                    let (type_args, args) = self.parse_call_args();
                    let rparen = self.expect(TokenKind::RParen).ok();
                    let base_span = base.span();
                    base = Expr::MethodCall {
                        receiver: Box::new(base),
                        op,
                        type_args,
                        args,
                        span: Span::merge(base_span, rparen.map(|t| t.span).unwrap_or_default()),
                    };
                    continue;
                } else {
                    // field access: chain of fields
                    let mut cur = base;
                    for f in chain {
                        cur = Expr::Field(Box::new(cur), f, Span::none());
                    }
                    base = cur;
                    continue;
                }
            }
            if matches!(self.peek_kind(), TokenKind::LBracket) {
                self.advance(); // [
                let index = self.parse_expr();
                self.expect(TokenKind::RBracket).ok();
                base = Expr::Index(Box::new(base), Box::new(index), Span::none());
                continue;
            }
            break;
        }
        base
    }

    fn parse_call_args(&mut self) -> (Vec<Type>, Vec<Expr>) {
        let mut type_args = Vec::new();
        let mut args = Vec::new();
        if matches!(self.peek_kind(), TokenKind::LBracket) {
            self.advance(); // <
            loop {
                let ty = self.parse_type();
                type_args.push(ty);
                if !matches!(self.peek_kind(), TokenKind::Comma) { break; }
                self.advance();
            }
            self.expect(TokenKind::RBracket).ok(); // >
        }
        if !matches!(self.peek_kind(), TokenKind::RParen) {
            loop {
                args.push(self.parse_expr());
                if !matches!(self.peek_kind(), TokenKind::Comma) { break; }
                self.advance();
            }
        }
        (type_args, args)
    }

    fn parse_primary(&mut self) -> Expr {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Ident(s) => Expr::Ident(s, tok.span),
            TokenKind::IntLit(v, suf) => Expr::IntLit(v, suf, tok.span),
            TokenKind::FloatLit(text, suf) => Expr::FloatLit(text.parse::<f64>().unwrap_or(0.0), suf, tok.span),
            TokenKind::True => Expr::BoolLit(true, tok.span),
            TokenKind::False => Expr::BoolLit(false, tok.span),
            TokenKind::StringLit(s) => Expr::StringLit(s, tok.span),
            TokenKind::CharLit(c) => Expr::CharLit(c, tok.span),
            TokenKind::Null => Expr::Null(tok.span),
            TokenKind::LParen => {
                let e = self.parse_expr();
                self.expect(TokenKind::RParen).ok();
                let e_span = e.span();
                Expr::Paren(Box::new(e), Span::merge(tok.span, e_span))
            }
            _ => {
                self.diags.add(Diagnostic::new("E0105", tok.span, format!("unexpected token in expression: {}", tok.kind)));
                Expr::Null(tok.span)
            }
        }
    }

    fn parse_const_expr(&mut self) -> ConstExpr {
        // simple: int / bool / null / unary / binary
        let tok = self.peek_kind().clone();
        match tok {
            TokenKind::IntLit(v, _) => {
                self.advance();
                ConstExpr::Int(v, Span::none())
            }
            TokenKind::True => {
                self.advance();
                ConstExpr::Bool(true, Span::none())
            }
            TokenKind::False => {
                self.advance();
                ConstExpr::Bool(false, Span::none())
            }
            TokenKind::Null => {
                self.advance();
                ConstExpr::Null(Span::none())
            }
            _ => {
                self.diags.add(Diagnostic::new("E0105", self.peek().span, "expected constant expression"));
                ConstExpr::Int(0, Span::none())
            }
        }
    }
}
