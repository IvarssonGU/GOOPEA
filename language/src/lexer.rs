use logos::Logos;
use std::num::ParseIntError;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    #[default]
    InvalidToken,
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

#[derive(Logos, Debug, Clone)]
#[logos(skip r"[ \t\n\f]+", skip r"#.*\n?", error = LexicalError)]
// #[logos(error = String)]
pub enum Token {
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBrack,
    #[token("]")]
    RBrack,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("=")]
    Equal,
    #[token("_", priority = 3)]
    Wildcard,

    #[token("fip")]
    Fip,
    #[token("match")]
    Match,
    #[token("enum")]
    Enum,

    #[regex("[+-]", |lex| lex.slice().to_string())]
    PlusMinus(String),
    #[regex("[*/]", |lex| lex.slice().to_string())]
    MultiplyDivide(String),
    #[regex("<|>|<=|>=|==|!=", |lex| lex.slice().to_string())]
    Comparator(String),

    #[regex("[1-9][0-9]*", |lex| lex.slice().parse())]
    Integer(i64),
    // #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().to_owned())]
    // String(String),
    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice().to_string())]
    Identifier(String),
}

pub fn lexer(src: &str) -> Vec<Token> {
    //creates a lexer instance from the input
    let lexer = Token::lexer(src);

    //splits the input into tokens, using the lexer
    let mut tokens = vec![];
    for (token, span) in lexer.spanned() {
        match token {
            Ok(token) => tokens.push(token),
            Err(e) => {
                println!("lexer error at {:?} {:?}", span, e)
            }
        }
    }

    tokens
}
