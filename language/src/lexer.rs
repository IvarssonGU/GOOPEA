use logos::Logos;

#[derive(Logos, Debug, Clone)]
// #[logos(skip r"[ \t\n]+")]
// #[logos(error = String)]
pub enum Token {
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[regex("[0-9]+", |lex| lex.slice().parse::<isize>().unwrap())]
    Integer(isize),

    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().to_owned())]
    String(String),
}

pub fn lexer(src: &str) -> Vec<Token> {
    //creates a lexer instance from the input
    let lexer = Token::lexer(src);

    //splits the input into tokens, using the lexer
    let mut tokens = vec![];
    for (token, span) in lexer.spanned() {
        match token {
            Ok(token) => tokens.push(token),
            Err(_) => {
                println!("lexer error at {:?}", span)
            }
        }
    }

    tokens
}
