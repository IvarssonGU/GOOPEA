use logos::Logos;
use logos::SpannedIter;
use std::num::ParseIntError;

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    // instead of an iterator over characters, we have a token iterator
    token_stream: SpannedIter<'input, Token>,
}
impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        // the Token::lexer() method is provided by the Logos trait
        Self {
            token_stream: Token::lexer(input).spanned(),
        }
    }
}
impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream
            .next()
            .map(|(token, span)| Ok((span.start, token?, span.end)))
    }
}

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

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+", skip r"//.*\n?", error = LexicalError)]
// #[logos(error = String)]
pub enum Token {
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
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
    let lexer = Token::lexer(&src);

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
