use crate::{Diagnostic, MAX_DEPTH, MAX_NODES, MAX_SOURCE, MAX_TOKENS, Value, error};
use logos::Logos;
use std::collections::BTreeMap;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t\r\n]+")]
enum Token {
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("in")]
    In,
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Name,
    #[regex(r"\$[A-Za-z_][A-Za-z0-9_]*")]
    Variable,
    #[regex(r"[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?")]
    Number,
    #[regex(r#""([^"\\\x00-\x1F]|\\["\\/bfnrt]|\\u[0-9a-fA-F]{4})*""#)]
    String,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token("-")]
    Minus,
    #[token("==")]
    Eq,
    #[token("!=")]
    Ne,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token(">")]
    Gt,
    #[token(">=")]
    Ge,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Op {
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Ast {
    Literal(Value),
    Variable(String),
    Name(String),
    Call(String, Vec<(Option<String>, Node)>),
    Field(Box<Node>, String),
    Index(Box<Node>, Box<Node>),
    List(Vec<Node>),
    Record(BTreeMap<String, Node>),
    Not(Box<Node>),
    Neg(Box<Node>),
    Binary(Op, Box<Node>, Box<Node>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Node {
    pub ast: Ast,
    pub start: usize,
    pub end: usize,
    height: usize,
    grouped: bool,
}
#[derive(Clone, Debug)]
pub struct ParsedExpr {
    pub(crate) source: String,
    pub(crate) root: Node,
}
impl ParsedExpr {
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn canonical_source(&self) -> String {
        let formatted = format_node(&self.root);
        // Whitespace normalization must not reject a source admitted at the byte bound.
        if formatted.len() > MAX_SOURCE {
            self.source.clone()
        } else {
            formatted
        }
    }
}
struct Parser<'a> {
    source: &'a str,
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    at: usize,
    nodes: usize,
}
pub fn parse(source: &str) -> Result<ParsedExpr, Diagnostic> {
    if source.len() > MAX_SOURCE {
        return Err(error("source-budget", "source exceeds 64 KiB"));
    }
    let mut tokens = Vec::new();
    for (t, s) in Token::lexer(source).spanned() {
        let t = t.map_err(|_| Diagnostic {
            code: "lex".into(),
            message: "unrecognized or malformed token".into(),
            start: s.start,
            end: s.end,
            expression: None,
        })?;
        tokens.push((t, s));
        if tokens.len() > MAX_TOKENS {
            return Err(error("token-budget", "too many tokens"));
        }
    }
    let mut p = Parser {
        source,
        tokens,
        at: 0,
        nodes: 0,
    };
    let root = p.expr(0, 0)?;
    if p.at != p.tokens.len() {
        return Err(p.err("syntax", "unexpected trailing token"));
    }
    Ok(ParsedExpr {
        source: source.into(),
        root,
    })
}
impl Parser<'_> {
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.at).map(|t| t.0)
    }
    fn err(&self, code: &str, msg: &str) -> Diagnostic {
        let span = self
            .tokens
            .get(self.at)
            .map(|t| t.1.clone())
            .unwrap_or(self.source.len()..self.source.len());
        Diagnostic {
            code: code.into(),
            message: msg.into(),
            start: span.start,
            end: span.end,
            expression: None,
        }
    }
    fn take(&mut self) -> Result<(Token, std::ops::Range<usize>), Diagnostic> {
        let t = self
            .tokens
            .get(self.at)
            .cloned()
            .ok_or_else(|| self.err("syntax", "expected expression"))?;
        self.at += 1;
        Ok(t)
    }
    fn expect(&mut self, t: Token) -> Result<(), Diagnostic> {
        if self.peek() != Some(t) {
            return Err(self.err("syntax", &format!("expected {t:?}")));
        }
        self.at += 1;
        Ok(())
    }
    fn node(&mut self, ast: Ast, start: usize) -> Result<Node, Diagnostic> {
        self.nodes += 1;
        if self.nodes > MAX_NODES {
            return Err(self.err("node-budget", "too many expression nodes"));
        }
        let height = 1 + match &ast {
            Ast::Field(n, _) | Ast::Not(n) | Ast::Neg(n) => n.height,
            Ast::Index(a, b) | Ast::Binary(_, a, b) => a.height.max(b.height),
            Ast::Call(_, args) => args.iter().map(|(_, v)| v.height).max().unwrap_or(0),
            Ast::List(v) => v.iter().map(|v| v.height).max().unwrap_or(0),
            Ast::Record(v) => v.values().map(|v| v.height).max().unwrap_or(0),
            _ => 0,
        };
        if height > MAX_DEPTH {
            return Err(self.err("parse-depth", "expression tree exceeds 32 levels"));
        }
        let end = self.tokens[self.at - 1].1.end;
        Ok(Node {
            ast,
            start,
            end,
            height,
            grouped: false,
        })
    }
    fn expr(&mut self, min: u8, depth: usize) -> Result<Node, Diagnostic> {
        if depth > MAX_DEPTH {
            return Err(self.err("parse-depth", "expression nesting exceeds 32"));
        }
        let (token, span) = self.take()?;
        let start = span.start;
        let mut left = match token {
            Token::True => self.node(Ast::Literal(Value::Bool(true)), start)?,
            Token::False => self.node(Ast::Literal(Value::Bool(false)), start)?,
            Token::String => {
                let s = serde_json::from_str(&self.source[span])
                    .map_err(|e| self.err("string", &e.to_string()))?;
                self.node(Ast::Literal(Value::String(s)), start)?
            }
            Token::Number => {
                let n = crate::number::parse_number(&self.source[span]).map_err(|mut e| {
                    e.start = start;
                    e.end = self.tokens[self.at - 1].1.end;
                    e
                })?;
                self.node(Ast::Literal(n), start)?
            }
            Token::Variable => self.node(Ast::Variable(self.source[span][1..].into()), start)?,
            Token::Name => {
                let mut name = self.source[span].to_owned();
                while self.peek() == Some(Token::Dot)
                    && self
                        .tokens
                        .get(self.at + 1)
                        .is_some_and(|t| t.0 == Token::Name)
                {
                    self.at += 1;
                    let (_, s) = self.take()?;
                    name.push('.');
                    name.push_str(&self.source[s]);
                }
                self.node(Ast::Name(name), start)?
            }
            Token::Not => {
                let x = self.expr(3, depth + 1)?;
                self.node(Ast::Not(Box::new(x)), start)?
            }
            Token::Minus => {
                if self.peek() == Some(Token::Number) {
                    let (_, s) = self.take()?;
                    let n = crate::number::parse_number(&format!("-{}", &self.source[s]))?;
                    self.node(Ast::Literal(n), start)?
                } else {
                    let n = self.expr(5, depth + 1)?;
                    self.node(Ast::Neg(Box::new(n)), start)?
                }
            }
            Token::LParen => {
                let mut n = self.expr(0, depth + 1)?;
                self.expect(Token::RParen)?;
                n.grouped = true;
                n
            }
            Token::LBracket => {
                let mut values = Vec::new();
                while self.peek() != Some(Token::RBracket) {
                    values.push(self.expr(0, depth + 1)?);
                    if self.peek() != Some(Token::Comma) {
                        break;
                    }
                    self.at += 1;
                }
                self.expect(Token::RBracket)?;
                self.node(Ast::List(values), start)?
            }
            Token::LBrace => {
                let mut fields = BTreeMap::new();
                while self.peek() != Some(Token::RBrace) {
                    let (t, s) = self.take()?;
                    let key = match t {
                        Token::Name => self.source[s].into(),
                        Token::String => serde_json::from_str(&self.source[s])
                            .map_err(|e| self.err("string", &e.to_string()))?,
                        _ => return Err(self.err("syntax", "record key must be name or string")),
                    };
                    self.expect(Token::Colon)?;
                    let value = self.expr(0, depth + 1)?;
                    if fields.insert(key, value).is_some() {
                        return Err(self.err("duplicate-field", "duplicate record field"));
                    }
                    if self.peek() != Some(Token::Comma) {
                        break;
                    }
                    self.at += 1;
                }
                self.expect(Token::RBrace)?;
                self.node(Ast::Record(fields), start)?
            }
            _ => return Err(self.err("syntax", "expected literal, variable or function")),
        };
        loop {
            if min <= 6 {
                match self.peek() {
                    Some(Token::LParen) => {
                        let Ast::Name(name) = left.ast else {
                            return Err(self.err("syntax", "only catalog names can be called"));
                        };
                        self.at += 1;
                        let mut args = Vec::new();
                        while self.peek() != Some(Token::RParen) {
                            let name = if self.peek() == Some(Token::Name)
                                && self
                                    .tokens
                                    .get(self.at + 1)
                                    .is_some_and(|t| t.0 == Token::Colon)
                            {
                                let (_, s) = self.take()?;
                                self.at += 1;
                                Some(self.source[s].into())
                            } else {
                                None
                            };
                            args.push((name, self.expr(0, depth + 1)?));
                            if self.peek() != Some(Token::Comma) {
                                break;
                            }
                            self.at += 1;
                        }
                        self.expect(Token::RParen)?;
                        left = self.node(Ast::Call(name, args), start)?;
                        continue;
                    }
                    Some(Token::Dot) => {
                        self.at += 1;
                        let (t, s) = self.take()?;
                        if t != Token::Name {
                            return Err(self.err("syntax", "expected field name"));
                        }
                        left =
                            self.node(Ast::Field(Box::new(left), self.source[s].into()), start)?;
                        continue;
                    }
                    Some(Token::LBracket) => {
                        self.at += 1;
                        let index = self.expr(0, depth + 1)?;
                        self.expect(Token::RBracket)?;
                        left = self.node(Ast::Index(Box::new(left), Box::new(index)), start)?;
                        continue;
                    }
                    _ => {}
                }
            }
            let (op, bp) = match self.peek() {
                Some(Token::Or) => (Op::Or, 1),
                Some(Token::And) => (Op::And, 2),
                Some(Token::Eq) => (Op::Eq, 4),
                Some(Token::Ne) => (Op::Ne, 4),
                Some(Token::Lt) => (Op::Lt, 4),
                Some(Token::Le) => (Op::Le, 4),
                Some(Token::Gt) => (Op::Gt, 4),
                Some(Token::Ge) => (Op::Ge, 4),
                Some(Token::In) => (Op::In, 4),
                _ => break,
            };
            if bp < min {
                break;
            }
            if bp == 4
                && !left.grouped
                && matches!(
                    left.ast,
                    Ast::Binary(
                        Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge | Op::In,
                        _,
                        _
                    )
                )
            {
                return Err(self.err("comparison-chain", "comparisons are nonassociative"));
            }
            self.at += 1;
            let right = self.expr(bp + 1, depth + 1)?;
            left = self.node(Ast::Binary(op, Box::new(left), Box::new(right)), start)?;
        }
        Ok(left)
    }
}
fn format_node(n: &Node) -> String {
    format_at(n, 0)
}
fn format_at(n: &Node, parent: u8) -> String {
    let precedence = match &n.ast {
        Ast::Binary(Op::Or, _, _) => 1,
        Ast::Binary(Op::And, _, _) => 2,
        Ast::Not(_) => 3,
        Ast::Binary(..) => 4,
        Ast::Neg(_) => 5,
        Ast::Field(..) | Ast::Index(..) | Ast::Call(..) => 6,
        _ => 7,
    };
    let text = match &n.ast {
        Ast::Literal(Value::Bool(b)) => b.to_string(),
        Ast::Literal(Value::String(s)) => serde_json::to_string(s).expect("string encoding"),
        Ast::Literal(Value::Integer(n)) => n.clone(),
        Ast::Literal(Value::Decimal(n)) => format!("{}e-{}", n.coefficient, n.scale),
        Ast::Literal(_) => unreachable!("parser literal"),
        Ast::Variable(v) => format!("${v}"),
        Ast::Name(n) => n.clone(),
        Ast::Call(n, args) => format!(
            "{n}({})",
            args.iter()
                .map(|(k, v)| format!(
                    "{}{}",
                    k.as_ref().map(|k| format!("{k}: ")).unwrap_or_default(),
                    format_node(v)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Ast::Field(n, f) => {
            // `(fact).field` is record traversal; `fact.field` is a catalog name.
            let target = if matches!(n.ast, Ast::Name(_)) {
                format!("({})", format_at(n, 0))
            } else {
                format_at(n, 6)
            };
            format!("{target}.{f}")
        }
        Ast::Index(n, i) => format!("{}[{}]", format_at(n, 6), format_node(i)),
        Ast::List(v) => format!(
            "[{}]",
            v.iter().map(format_node).collect::<Vec<_>>().join(", ")
        ),
        Ast::Record(v) => format!(
            "{{{}}}",
            v.iter()
                .map(|(k, v)| format!(
                    "{}: {}",
                    serde_json::to_string(k).expect("string encoding"),
                    format_node(v)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Ast::Not(n) => format!("not {}", format_at(n, 3)),
        Ast::Neg(n) => format!("-{}", format_at(n, 5)),
        Ast::Binary(op, a, b) => format!(
            "{} {} {}",
            format_at(a, if precedence == 4 { 5 } else { precedence }),
            match op {
                Op::And => "and",
                Op::Or => "or",
                Op::Eq => "==",
                Op::Ne => "!=",
                Op::Lt => "<",
                Op::Le => "<=",
                Op::Gt => ">",
                Op::Ge => ">=",
                Op::In => "in",
            },
            format_at(b, precedence + 1)
        ),
    };
    if precedence < parent {
        format!("({text})")
    } else {
        text
    }
}
