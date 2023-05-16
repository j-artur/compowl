use crate::{
    span::{Located, Source, Span},
    table::{SymbolTable, Type},
};

#[derive(Debug, Clone, Copy)]
pub enum Keyword {
    SOME,
    ALL,
    VALUE,
    MIN,
    MAX,
    EXACTLY,
    THAT,
    ONLY,
    NOT,
    AND,
    OR,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Datatype {
    Integer,
    Decimal,
    Float,
    String,
}

#[derive(Debug, Clone, Copy)]
pub enum Punctuation {
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Comma,
    LessThanEqual,
    GreaterThanEqual,
    LessThan,
    GreaterThan,
}

pub enum TokenType {
    Keyword(Keyword),
    Datatype(Datatype),
    Punctuation(Punctuation),
    ClassIdentifier(String),
    PropertyIdentifier(String),
    Cardinality(usize),
    Literal(String),
}

#[derive(Debug, Clone, Copy)]
pub enum LexerErr {
    UnrecognizedToken,
}

pub struct LexerResult<'s, T> {
    remaining: Span<'s>,
    result: Result<T, LexerErr>,
}

impl<'s, T> LexerResult<'s, T> {
    fn new(remaining: Span<'s>, result: Result<T, LexerErr>) -> Self {
        Self { remaining, result }
    }

    fn ok(remaining: Span<'s>, value: T) -> Self {
        Self::new(remaining, Ok(value))
    }

    fn err(remaining: Span<'s>, err: LexerErr) -> Self {
        Self::new(remaining, Err(err))
    }

    fn result(self) -> Result<(Span<'s>, T), (Span<'s>, LexerErr)> {
        match self.result {
            Ok(value) => Ok((self.remaining, value)),
            Err(err) => Err((self.remaining, err)),
        }
    }

    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> LexerResult<'s, U> {
        LexerResult {
            remaining: self.remaining,
            result: self.result.map(f),
        }
    }

    fn or_else<F: FnOnce(Span<'s>) -> LexerResult<'s, T>>(self, f: F) -> LexerResult<'s, T> {
        match self.result {
            Ok(value) => LexerResult::ok(self.remaining, value),
            Err(_) => f(self.remaining),
        }
    }

    fn and_then<U, F: FnOnce(Span<'s>, T) -> LexerResult<'s, U>>(self, f: F) -> LexerResult<'s, U> {
        match self.result {
            Ok(value) => f(self.remaining, value),
            Err(err) => LexerResult::err(self.remaining, err),
        }
    }
}

fn skip_whitespace<'s>(src: Span<'s>) -> Span<'s> {
    let whitespace_len = src
        .fragment()
        .chars()
        .take_while(|c| c.is_whitespace())
        .count();

    src.shift(whitespace_len)
}

fn parse_char<'s>(src: Span<'s>, c: char) -> LexerResult<'s, Located<'s, char>> {
    match src.fragment().chars().next().filter(|&c_| c_ == c) {
        None => LexerResult::err(src, LexerErr::UnrecognizedToken),
        Some(matched) => {
            let (remaining, span) = src.split(1);
            LexerResult::ok(remaining, Located::new(matched, span))
        }
    }
}

fn parse_seq<'s>(src: Span<'s>, sequence: &str) -> LexerResult<'s, Located<'s, &'s str>> {
    match src
        .fragment()
        .get(0..sequence.len())
        .filter(|substr| *substr == sequence)
    {
        None => LexerResult::err(src, LexerErr::UnrecognizedToken),
        Some(matched) => {
            let (remaining, span) = src.split(matched.len());
            LexerResult::ok(remaining, Located::new(matched, span))
        }
    }
}

fn parse_seq_any_casing<'s>(
    src: Span<'s>,
    sequence: &str,
) -> LexerResult<'s, Located<'s, &'s str>> {
    match src.fragment().get(0..sequence.len()).filter(|substr| {
        substr.to_string() == sequence.to_lowercase()
            || substr.to_string() == sequence.to_uppercase()
    }) {
        None => LexerResult::err(src, LexerErr::UnrecognizedToken),
        Some(matched) => {
            let (remaining, span) = src.split(matched.len());
            LexerResult::ok(remaining, Located::new(matched, span))
        }
    }
}

fn parse_if<'s, F: FnMut(char) -> bool>(
    src: Span<'s>,
    mut predicate: F,
) -> LexerResult<'s, Located<'s, char>> {
    match src.fragment().chars().next().filter(|c| predicate(*c)) {
        None => LexerResult::err(src, LexerErr::UnrecognizedToken),
        Some(matched) => {
            let (remaining, span) = src.split(1);
            LexerResult::ok(remaining, Located::new(matched, span))
        }
    }
}

fn parse_while<'s, F: FnMut(char) -> bool>(
    src: Span<'s>,
    mut predicate: F,
) -> (Span<'s>, Located<'s, &'s str>) {
    let len = src.fragment().chars().take_while(|c| predicate(*c)).count();

    let (remaining, located) = src.split(len);

    (remaining, Located::new(located.fragment(), located))
}

fn parse_keyword<'s>(src: Span<'s>) -> LexerResult<'s, Located<'s, Keyword>> {
    LexerResult::err(src, LexerErr::UnrecognizedToken)
        .or_else(|src| parse_seq_any_casing(src, "SOME").map(|k| k.map(|_| Keyword::SOME)))
        .or_else(|src| parse_seq_any_casing(src, "ALL").map(|k| k.map(|_| Keyword::ALL)))
        .or_else(|src| parse_seq_any_casing(src, "VALUE").map(|k| k.map(|_| Keyword::VALUE)))
        .or_else(|src| parse_seq_any_casing(src, "MIN").map(|k| k.map(|_| Keyword::MIN)))
        .or_else(|src| parse_seq_any_casing(src, "MAX").map(|k| k.map(|_| Keyword::MAX)))
        .or_else(|src| parse_seq_any_casing(src, "EXACTLY").map(|k| k.map(|_| Keyword::EXACTLY)))
        .or_else(|src| parse_seq_any_casing(src, "THAT").map(|k| k.map(|_| Keyword::THAT)))
        .or_else(|src| parse_seq_any_casing(src, "ONLY").map(|k| k.map(|_| Keyword::ONLY)))
        .or_else(|src| parse_seq_any_casing(src, "NOT").map(|k| k.map(|_| Keyword::NOT)))
        .or_else(|src| parse_seq_any_casing(src, "AND").map(|k| k.map(|_| Keyword::AND)))
        .or_else(|src| parse_seq_any_casing(src, "OR").map(|k| k.map(|_| Keyword::OR)))
        .and_then(|remaining, k| {
            parse_if(remaining, |c| !c.is_alphabetic() && c != '_')
                .and_then(|_, _| LexerResult::ok(remaining, k))
                .or_else(|_| LexerResult::err(src, LexerErr::UnrecognizedToken))
        })
}

fn parse_datatype<'s>(src: Span<'s>) -> LexerResult<'s, Located<'s, Datatype>> {
    LexerResult::err(src, LexerErr::UnrecognizedToken)
        .or_else(|src| parse_seq_any_casing(src, "INTEGER").map(|l| l.map(|_| Datatype::Integer)))
        .or_else(|src| parse_seq_any_casing(src, "DECIMAL").map(|l| l.map(|_| Datatype::Decimal)))
        .or_else(|src| parse_seq_any_casing(src, "FLOAT").map(|l| l.map(|_| Datatype::Float)))
        .or_else(|src| parse_seq_any_casing(src, "STRING").map(|l| l.map(|_| Datatype::String)))
        .and_then(|remaining, d| {
            parse_if(remaining, |c| !c.is_alphabetic() && c != '_')
                .and_then(|_, _| LexerResult::ok(remaining, d))
                .or_else(|_| LexerResult::err(src, LexerErr::UnrecognizedToken))
        })
}

fn parse_punctuation<'s>(src: Span<'s>) -> LexerResult<'s, Located<'s, Punctuation>> {
    LexerResult::err(src, LexerErr::UnrecognizedToken)
        .or_else(|src| parse_char(src, '(').map(|l| l.map(|_| Punctuation::OpenParen)))
        .or_else(|src| parse_char(src, ')').map(|l| l.map(|_| Punctuation::CloseParen)))
        .or_else(|src| parse_char(src, '{').map(|l| l.map(|_| Punctuation::OpenBrace)))
        .or_else(|src| parse_char(src, '}').map(|l| l.map(|_| Punctuation::CloseBrace)))
        .or_else(|src| parse_char(src, '[').map(|l| l.map(|_| Punctuation::OpenBracket)))
        .or_else(|src| parse_char(src, ']').map(|l| l.map(|_| Punctuation::CloseBracket)))
        .or_else(|src| parse_char(src, ',').map(|l| l.map(|_| Punctuation::Comma)))
        .or_else(|src| parse_seq(src, "<=").map(|l| l.map(|_| Punctuation::LessThanEqual)))
        .or_else(|src| parse_seq(src, ">=").map(|l| l.map(|_| Punctuation::GreaterThanEqual)))
        .or_else(|src| parse_char(src, '<').map(|l| l.map(|_| Punctuation::LessThan)))
        .or_else(|src| parse_char(src, '>').map(|l| l.map(|_| Punctuation::GreaterThan)))
}

fn parse_identifier<'s>(src: Span<'s>) -> LexerResult<'s, Located<'s, String>> {
    parse_if(src, |c| c.is_uppercase()).and_then(|r, c| {
        let mut prev = c.value;

        let (_, rest) = parse_while(r, |c| {
            let should_continue = (prev.is_uppercase() && (c.is_lowercase()))
                || (prev.is_lowercase() && (c.is_alphabetic()));

            prev = c;

            should_continue
        });

        let len = rest.value.len() + 1;

        let (remaining, span) = src.split(len);

        if span.fragment().is_empty() {
            LexerResult::err(src, LexerErr::UnrecognizedToken)
        } else {
            parse_if(remaining, |c| !c.is_alphabetic() && c != '_').and_then(|_, _| {
                LexerResult::ok(remaining, Located::new(span.fragment().to_string(), span))
            })
        }
    })
}

fn parse_class<'s>(src: Span<'s>) -> LexerResult<'s, Located<'s, String>> {
    parse_if(src, |c| c.is_uppercase()).and_then(|_, _| {
        let mut prev = '_';

        let (remaining, located) = parse_while(src, |c| {
            let should_continue = (prev.is_uppercase() && (c.is_lowercase() || c == '_'))
                || (prev.is_lowercase() && (c.is_alphabetic() || c == '_'))
                || (prev == '_' && c.is_uppercase());

            prev = c;

            should_continue
        });

        if located.value.is_empty() {
            LexerResult::err(src, LexerErr::UnrecognizedToken)
        } else {
            parse_if(remaining, |c| !c.is_alphabetic() && c != '_')
                .and_then(|_, _| LexerResult::ok(remaining, located.map(|s| s.to_string())))
        }
    })
}

fn parse_property<'s>(src: Span<'s>) -> LexerResult<'s, Located<'s, String>> {
    LexerResult::err(src, LexerErr::UnrecognizedToken)
        .or_else(|src| {
            parse_seq(src, "has")
                .and_then(|r, _| parse_identifier(r))
                .and_then(|_, id| {
                    let len = 3 + id.value.len();

                    let (remaining, span) = src.split(len);

                    LexerResult::ok(remaining, Located::new(span.fragment().to_string(), span))
                })
        })
        .or_else(|_| {
            parse_seq(src, "is")
                .and_then(|r, _| parse_identifier(r))
                .and_then(|_, id| {
                    if id.value.ends_with("Of") {
                        let len = 2 + id.value.len();

                        let (remaining, span) = src.split(len);

                        LexerResult::ok(remaining, Located::new(span.fragment().to_string(), span))
                    } else {
                        LexerResult::err(src, LexerErr::UnrecognizedToken)
                    }
                })
        })
}

fn parse_cardinality<'s>(src: Span<'s>) -> LexerResult<'s, Located<'s, usize>> {
    let (remaining, located) = parse_while(src, |c| c.is_digit(10));

    if located.value.is_empty() {
        LexerResult::err(src, LexerErr::UnrecognizedToken)
    } else {
        let cardinality = located.map(|s| s.to_string().parse().unwrap());
        LexerResult::ok(remaining, cardinality)
    }
}

fn parse_literal<'s>(src: Span<'s>) -> LexerResult<'s, Located<'s, String>> {
    parse_char(src, '"').and_then(|r, _| {
        let (remaining, located) = parse_while(r, |c| c != '"' && c != '\n');

        parse_char(remaining, '"').and_then(|_, _| {
            let len = 2 + located.value.len();

            let (remaining, span) = src.split(len);

            LexerResult::ok(remaining, Located::new(located.value.to_string(), span))
        })
    })
}

fn parse_token<'s>(src: Span<'s>) -> LexerResult<'s, Located<'s, TokenType>> {
    LexerResult::err(src, LexerErr::UnrecognizedToken)
        .or_else(|src| parse_keyword(src).map(|l| l.map(|k| TokenType::Keyword(k))))
        .or_else(|src| parse_datatype(src).map(|l| l.map(|d| TokenType::Datatype(d))))
        .or_else(|src| parse_punctuation(src).map(|l| l.map(|p| TokenType::Punctuation(p))))
        .or_else(|src| parse_class(src).map(|l| l.map(|c| TokenType::ClassIdentifier(c))))
        .or_else(|src| parse_property(src).map(|l| l.map(|p| TokenType::PropertyIdentifier(p))))
        .or_else(|src| parse_cardinality(src).map(|l| l.map(|c| TokenType::Cardinality(c))))
        .or_else(|src| parse_literal(src).map(|l| l.map(|s| TokenType::Literal(s))))
}

#[derive(Debug, Clone)]
pub enum Token {
    Keyword(Keyword),
    Datatype(Datatype),
    Punctuation(Punctuation),
    Cardinality(usize),
    ClassIdentifier { index: usize },
    PropertyIdentifier { index: usize },
    Literal { index: usize },
}

pub fn parse<'s>(
    src: &'s Source,
) -> Result<(SymbolTable, Vec<Located<'s, Token>>), Located<'s, LexerErr>> {
    let mut src = skip_whitespace(Span::from(src));
    let mut table = SymbolTable::new();
    let mut tokens = Vec::new();

    while let Ok((remaining, token_type)) = parse_token(src).result() {
        let located = match token_type.value {
            TokenType::Keyword(k) => Located::new(Token::Keyword(k), token_type.span),
            TokenType::Datatype(d) => Located::new(Token::Datatype(d), token_type.span),
            TokenType::Punctuation(p) => Located::new(Token::Punctuation(p), token_type.span),
            TokenType::Cardinality(c) => Located::new(Token::Cardinality(c), token_type.span),
            TokenType::ClassIdentifier(c) => {
                let index = table.get_or_insert(Type::Class, c);
                Located::new(Token::ClassIdentifier { index }, token_type.span)
            }
            TokenType::PropertyIdentifier(p) => {
                let index = table.get_or_insert(Type::Property(None), p);
                Located::new(Token::PropertyIdentifier { index }, token_type.span)
            }
            TokenType::Literal(s) => {
                let index = table.get_or_insert(Type::Literal, s);
                Located::new(Token::Literal { index }, token_type.span)
            }
        };

        tokens.push(located);
        src = skip_whitespace(remaining);
    }

    if !src.fragment().is_empty() {
        return Err(Located::new(LexerErr::UnrecognizedToken, src));
    }

    Ok((table, tokens))
}
