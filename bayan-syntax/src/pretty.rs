use crate::ast::*;

pub fn pretty_module(m: &Module) -> String {
    let mut out = String::new();
    out.push_str(&format!("module {}\n", m.name));
    for item in &m.items {
        out.push_str(&pretty_item(item));
        out.push('\n');
    }
    out
}

fn pretty_item(item: &Item) -> String {
    match item {
        Item::Struct(s) => pretty_struct(s),
        Item::Fn(f) => pretty_fn(f),
        Item::ExternFn(e) => pretty_extern_fn(e),
        Item::Global(g) => pretty_global(g),
    }
}

fn pretty_struct(s: &StructDecl) -> String {
    let mut out = format!("struct {} {{", s.name);
    for f in &s.fields {
        out.push_str(&format!(" {}: {}", f.name, f.ty));
    }
    out.push_str("}");
    if let Some(a) = s.align {
        out.push_str(&format!(" align({})", a));
    }
    out
}

fn pretty_fn(f: &FnDecl) -> String {
    let mut out = String::new();
    if f.is_extern { out.push_str("extern "); }
    if f.is_export { out.push_str("export "); }
    out.push_str(&format!("fn {}(", f.name));
    for (i, p) in f.params.iter().enumerate() {
        if i > 0 { out.push_str(", "); }
        out.push_str(&format!("{}: {}", p.name, p.ty));
    }
    out.push_str(&format!(") -> {}", f.ret));
    for a in &f.attrs {
        out.push_str(&format!(" [{}]", a.name));
    }
    out.push_str(" {\n");
    for s in &f.body.stmts {
        out.push_str("  ");
        out.push_str(&pretty_stmt(s));
        out.push('\n');
    }
    out.push('}');
    out
}

fn pretty_extern_fn(e: &ExternFnDecl) -> String {
    let mut out = String::new();
    out.push_str(&format!("extern fn {}(", e.name));
    for (i, p) in e.params.iter().enumerate() {
        if i > 0 { out.push_str(", "); }
        out.push_str(&format!("{}: {}", p.name, p.ty));
    }
    out.push_str(&format!(") -> {}", e.ret));
    for a in &e.attrs {
        out.push_str(&format!(" [{}]", a.name));
    }
    out
}

fn pretty_global(g: &GlobalDecl) -> String {
    format!("global {}: {} = {:?}", g.name, g.ty, g.init)
}

fn pretty_stmt(s: &Stmt) -> String {
    match s {
        Stmt::VarDecl(d) => format!("var {}: {} = {};", d.name, d.ty, pretty_expr(&d.init)),
        Stmt::LetDecl(d) => format!("let {}: {} = {};", d.name, d.ty, pretty_expr(&d.init)),
        Stmt::Assign(a) => format!("{} = {};", a.name, pretty_expr(&a.value)),
        Stmt::ExprStmt(e) => format!("{};", pretty_expr(e)),
        Stmt::If(i) => {
            let mut out = format!("if {} {}", pretty_expr(&i.cond), pretty_block(&i.then_block));
            if let Some(eb) = &i.else_block {
                out.push_str(&format!(" else {}", pretty_block(eb)));
            }
            out
        }
        Stmt::Loop(l) => format!("loop {}", pretty_block(&l.body)),
        Stmt::Return(e, _) => match e {
            Some(e) => format!("return {};", pretty_expr(e)),
            None => "return;".to_string(),
        },
        Stmt::Break(e, _) => match e {
            Some(e) => format!("break {};", pretty_expr(e)),
            None => "break;".to_string(),
        },
        Stmt::Continue(_) => "continue;".to_string(),
        Stmt::Trap(code, _) => format!("trap({});", code),
        Stmt::Goto(label, _) => format!("goto {};", label),
        Stmt::LabeledBlock(label, b) => format!("{}: {}", label, pretty_block(b)),
        Stmt::Unsafe(_) => format!("unsafe {{}}"),
        Stmt::Asm(a) => format!("asm volatile(\"{}\")", a.template),
        Stmt::Switch(sw) => {
            let mut out = format!("switch {}", pretty_expr(&sw.discr));
            out.push_str(" {");
            for (c, b) in &sw.cases {
                out.push_str(&format!(" case {} => {}", pretty_const(c), pretty_block(b)));
            }
            if let Some(d) = &sw.default {
                out.push_str(&format!(" default => {}", pretty_block(d)));
            }
            out.push('}');
            out
        }
    }
}

fn pretty_block(b: &Block) -> String {
    let mut out = String::new();
    out.push('{');
    for s in &b.stmts {
        out.push_str("\n  ");
        out.push_str(&pretty_stmt(s));
    }
    if !b.stmts.is_empty() {
        out.push('\n');
    }
    out.push('}');
    out
}

fn pretty_expr(e: &Expr) -> String {
    match e {
        Expr::Ident(s, _) => s.clone(),
        Expr::IntLit(v, _, _) => v.to_string(),
        Expr::FloatLit(v, _, _) => v.to_string(),
        Expr::BoolLit(b, _) => b.to_string(),
        Expr::StringLit(s, _) => format!("\"{}\"", s),
        Expr::CharLit(c, _) => format!("'{}'", c),
        Expr::Null(_) => "null".to_string(),
        Expr::MethodCall { receiver, op, type_args, args, .. } => {
            let mut out = format!("{}.{}", pretty_expr(receiver), op);
            if !type_args.is_empty() {
                out.push('<');
                for (i, t) in type_args.iter().enumerate() {
                    if i > 0 { out.push(','); }
                    out.push_str(&t.to_string());
                }
                out.push('>');
            }
            out.push('(');
            for (i, a) in args.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                out.push_str(&pretty_expr(a));
            }
            out.push(')');
            out
        }
        Expr::Unary(op, inner, _) => format!("{}{}", pretty_unary_op(op), pretty_expr(inner)),
        Expr::Binary(op, l, r, _) => format!("{} {} {}", pretty_expr(l), pretty_binary_op(op), pretty_expr(r)),
        Expr::Cast(op, ty, inner, _) => format!("{}<{}>({})", pretty_cast_op(op), ty, pretty_expr(inner)),
        Expr::Index(a, i, _) => format!("{}[{}]", pretty_expr(a), pretty_expr(i)),
        Expr::Field(a, f, _) => format!("{}.{}", pretty_expr(a), f),
        Expr::Paren(e, _) => format!("({})", pretty_expr(e)),
    }
}

fn pretty_unary_op(op: &UnaryOp) -> &str {
    match op { UnaryOp::Not => "not ", UnaryOp::Neg => "-", UnaryOp::Clz => "clz", UnaryOp::Ctz => "ctz", UnaryOp::Popcnt => "popcnt" }
}

fn pretty_binary_op(op: &BinaryOp) -> &str {
    match op {
        BinaryOp::AddWrap => "+", BinaryOp::SubWrap => "-", BinaryOp::MulWrap => "*",
        BinaryOp::DivS => "/", BinaryOp::RemS => "%", BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=", BinaryOp::LtS => "<", BinaryOp::LtU => "<u",
        BinaryOp::LeS => "<=", BinaryOp::LeU => "<=u", BinaryOp::GtS => ">",
        BinaryOp::GtU => ">u", BinaryOp::GeS => ">=", BinaryOp::GeU => ">=u",
        BinaryOp::AndLogical => "and", BinaryOp::OrLogical => "or",
        BinaryOp::And => "&", BinaryOp::Or => "|", BinaryOp::Xor => "^",
        _ => "op",
    }
}

fn pretty_cast_op(op: &CastOp) -> &str {
    match op {
        CastOp::Zext => "zext", CastOp::Sext => "sext", CastOp::Trunc => "trunc",
        CastOp::Fptosi => "fptosi", CastOp::Sitofp => "sitofp", CastOp::Fptou => "fptou",
        CastOp::Uitofp => "uitofp", CastOp::Bitcast => "bitcast",
        CastOp::Ptr2int => "ptr2int", CastOp::Int2ptr => "int2ptr",
    }
}

fn pretty_const(c: &ConstExpr) -> String {
    match c {
        ConstExpr::Int(v, _) => v.to_string(),
        ConstExpr::Bool(b, _) => b.to_string(),
        ConstExpr::Null(_) => "null".to_string(),
        _ => "const".to_string(),
    }
}
