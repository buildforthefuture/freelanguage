# Bayan Grammar (EBNF)

Expression style: **method-style** (`receiver.op(args)`). Grammar is LL(1)-friendly:
each statement begins with a keyword or an identifier/expression, so one token of
lookahead suffices in most cases.

## Lexical tokens

```
ident       = letter { letter | digit | "_" } .
letter      = "A".."Z" | "a".."z" | "_" .
digit       = "0".."9" .
int_lit     = dec_lit | hex_lit | bin_lit [ int_suffix ] .
dec_lit     = digit { digit } .
hex_lit     = "0" ("x"|"X") hex_digit { hex_digit } .
bin_lit     = "0" ("b"|"B") bin_digit { bin_digit } .
int_suffix  = "i8" | "i16" | "i32" | "i64" .
float_lit   = digit { digit } "." digit { digit } [ exponent ] [ float_suffix ]
          | digit { digit } exponent [ float_suffix ] .
exponent    = ("e"|"E") ["+"|"-"] digit { digit } .
float_suffix= "f32" | "f64" .
string_lit  = '"' { char } '"' .
char_lit    = "'" char "'" .
char        = any_unicode_except_quote_and_newline | escape .
escape      = "\n" | "\t" | "\r" | "\\" | "\"" | "\'" | "\0" .
comment     = "//" { any_except_newline }
          | "/*" { comment | any_except "*/" } "/*" .  (* nested *)
```

## Module

```
module        = "module" ident { top_level } .
top_level     = struct_decl | fn_decl | extern_decl | export_decl | global_decl .

struct_decl   = "struct" ident "{" field { "," field } "}" [ "align" "(" int_lit ")" ] .
field         = ident ":" type .

fn_decl       = "fn" ident "(" [ param { "," param } ] ")" "->" type [ attrs ] block .
param         = ident ":" type .
extern_decl   = "extern" "fn" ident "(" [ param { "," param } ] ")" "->" type [ attrs ] .
export_decl   = "export" fn_decl .
global_decl   = "global" ident ":" type "=" const_expr [ attrs ] .
```

## Types

```
type          = "i8" | "i16" | "i32" | "i64" | "bool" | "f32" | "f64" | "ptr" | "unit"
            | ident                                         (* struct name or array *)
            | "[" type ";" int_lit "]"                      (* array *)
```

## Attributes

```
attrs       = { "[" attr "]" } .
attr        = "noinline" | "inline" | "pure" | "cold" | "volatile"
          | "cc" "(" ident ")"                              (* sysv64, win64, bayan *)
          | "align" "(" int_lit ")" .
```

## Statements

```
block       = "{" { statement } } .

statement   = var_decl | let_decl | assign | expr_stmt
          | if_stmt | loop_stmt | switch_stmt | return_stmt
          | trap_stmt | goto_stmt | labeled_block | unsafe_block
          | asm_stmt | break_stmt | continue_stmt .

var_decl    = "var" ident ":" type "=" expr ";" .
let_decl    = "let" ident ":" type "=" expr ";" .
assign      = ident "=" expr ";" .
expr_stmt   = expr ";" .
if_stmt     = "if" expr block [ "else" block ] .
loop_stmt   = "loop" block .
break_stmt  = "break" [ expr ] ";" .
continue_stmt = "continue" ";" .
return_stmt = "return" [ expr ] ";" .
switch_stmt = "switch" expr "{" switch_case { switch_case } [ "default" "=>" block ] "}" .
switch_case = "case" const_expr "=>" block .
trap_stmt   = "trap" "(" int_lit ")" ";" .
goto_stmt   = "goto" ident ";" .
labeled_block = ident ":" block .
unsafe_block= "unsafe" "{" { statement } } .
asm_stmt    = "asm" "volatile" "(" string_lit ":" [ asm_out { "," asm_out } ] ":"
            [ asm_in { "," asm_in } ] ":" [ asm_clob { "," asm_clob } ] ")" ";" .
asm_out     = ident ":" type .
asm_in      = expr ":" type .
asm_clob    = string_lit .
```

## Expressions (precedence: low → high)

```
expr        = logical_or .
logical_or  = logical_and { "or" logical_and } .
logical_and = equality { "and" equality } .
equality    = relational { ("==" | "!=") relational } .
relational  = additive { ("lt.s"|"lt.u"|"le.s"|"le.u"|"gt.s"|"gt.u"|"ge.s"|"ge.u") additive } .
additive    = multiplicative { ("add.wrap"|"add.trap"|"add.sat"|"sub.wrap"|...) multiplicative } .
multiplicative = cast { ("mul.wrap"|...) cast } .
cast        = unary [ ("zext"|"sext"|"trunc"|"fptosi"|"sitofp"|"fptou"|"uitofp"|"bitcast"|"ptr2int"|"int2ptr") "<" type ">" ] .
unary       = ("not"|"-"|"clz"|"ctz"|"popcnt") unary | postfix .
postfix     = primary { method_call | index | field_access } .
method_call = "." ident [ "<" type { "," type } ">" ] "(" [ expr { "," expr } ] ")" .
index       = "[" expr "]" .
field_access= "." ident .
primary     = ident | int_lit | float_lit | bool_lit | string_lit | char_lit
          | "null" | "(" expr ")" | load | alloc | atomic_op | x86_op | arm_op .
load        = "load" [ "." align_mode ] "<" type ">" "(" expr ")" .
store       = "store" [ "." align_mode ] "<" type ">" "(" expr "," expr ")" .
align_mode  = "unaligned" | "aligned" int_lit .
alloc       = "alloc" "(" expr ")" | "free" "(" expr ")" | "realloc" "(" expr "," expr ")" .
alloca      = "alloca" "(" expr "," int_lit ")" .
atomic_op   = "atomic" "." ("load"|"store"|"rmw"|"cmpxchg") "<" type ">" "(" ... ")" .
x86_op      = "x86" "." ident "(" ... ")" .
arm_op      = "arm" "." ident "(" ... ")" .
bool_lit    = "true" | "false" .
```

### Ordering of method calls vs binary operators

Binary operators are sugar for method calls: `a + b` ≡ `a.add.wrap(b)`. The
parser desugars them into the same IR node.

### LL(1) note

`break [expr]` and `return [expr]` use optional expression; `if i.ge.s(len)` —
method call after ident is unambiguous because `.` only appears in method calls,
field access, and operators like `add.wrap` (which contain a `.` inside the
operator token). The lexer tokenizes `add.wrap` as a single operator token when
it follows an expression in operator position.
