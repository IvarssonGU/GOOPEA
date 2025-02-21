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
    #[token("()")]
    Unit,

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
    #[token("=>")]
    ThickArrow,
    #[token(",")]
    Comma,
    #[token("=")]
    Equal,
    #[token("+")]
    Plus,

    #[regex("[1-9][0-9]*", |lex| lex.slice().parse())]
    Integer(i64),

    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().to_owned())]
    String(String),

    #[regex("fip|enum|match", |lex| lex.slice().to_owned())]
    Keyword(String),

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
