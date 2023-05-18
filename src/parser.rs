use std::{fmt::Debug, iter::Peekable, slice::Iter, vec};

use crate::{
    lexer::{Datatype, Token},
    span::{Located, Span},
    table::{PropertyType, SymbolTable, Type},
};

pub enum ClassDecl {
    Defined(ClassIdentifier, Vec<Property>),
    Primitive(ClassIdentifier, Vec<Property>),
    Enumerated(Vec<ClassIdentifier>),
    Disjoint(Vec<ClassIdentifier>),
}

#[derive(PartialEq, Eq)]
pub enum Class {
    Identifier(ClassIdentifier),
    Defined(ClassIdentifier, Vec<Property>),
    Enumerated(Vec<ClassIdentifier>),
    Disjoint(Vec<ClassIdentifier>),
}

#[derive(PartialEq, Eq)]
pub struct ClassIdentifier {
    index: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Property {
    index: usize,
    description: PropertyDescription,
}

#[derive(PartialEq, Eq)]
pub enum PropertyDescription {
    Object(ObjectDescription),
    Data(DataDescription),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectDescription {
    Some(Class),
    Only(Class),
    Value(ClassIdentifier),
    Min(usize, Class),
    Max(usize, Class),
    Exactly(usize, Class),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DataDescription {
    Some(Data),
    Only(Data),
    Value(Literal),
    Min(usize, Data),
    Max(usize, Data),
    Exactly(usize, Data),
}

#[derive(PartialEq, Eq)]
pub struct Literal {
    index: usize,
}

#[derive(PartialEq, Eq)]
pub struct Data {
    datatype: Datatype,
    restriction: Option<Restriction>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RestrictionType {
    LessThanEqual,
    GreaterThanEqual,
    LessThan,
    GreaterThan,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Restriction(RestrictionType, Literal);

pub enum ParserErr<'t> {
    UnexpectedEndOfInput,
    RepeatedProperty(Located<'t, Property>),
    RepeatedClass(Located<'t, ClassIdentifier>),
    UnrecognizedToken {
        expected: &'static str,
        found: &'t Located<'t, Token>,
    },
    TypeMismatch {
        location: Span<'t>,
        expected: Type,
        found: Type,
    },
}

use crate::lexer::Keyword::*;
use crate::lexer::Punctuation::*;

type Tokens<'t> = Peekable<Iter<'t, Located<'t, Token>>>;

type ParserResult<'t, T> = Result<(Tokens<'t>, Located<'t, T>), ParserErr<'t>>;

pub fn parse<'t>(
    tokens: &'t [Located<'t, Token>],
    table: &mut SymbolTable,
) -> Result<Vec<Located<'t, ClassDecl>>, ParserErr<'t>> {
    let mut tokens = tokens.iter().peekable();
    let mut decls = Vec::new();

    while let Some(_) = tokens.peek() {
        let (remaining, decl) = parse_decl(tokens.clone(), table)?;
        tokens = remaining;
        decls.push(decl);
    }

    Ok(decls)
}

fn parse_decl<'t>(mut tokens: Tokens<'t>, table: &mut SymbolTable) -> ParserResult<'t, ClassDecl> {
    match tokens.peek().cloned() {
        Some(token) => match token.value {
            Token::ClassIdentifier { index } => {
                tokens.next();
                let class_identifier = ClassIdentifier { index };
                let span = token.span;

                match tokens.peek() {
                    Some(token) => match token.value {
                        Token::Keyword(AND) => {
                            tokens.next();
                            let (remaining, property) = parse_property(tokens, table)?;
                            tokens = remaining;
                            let mut span = span.merge(&property.span);
                            assert_property(&property, table)?;
                            let mut properties = vec![property.value];

                            while let Some(Token::Keyword(AND)) = tokens.peek().map(|t| &t.value) {
                                tokens.next();
                                let (remaining, property) = parse_property(tokens, table)?;
                                tokens = remaining;
                                span = span.merge(&property.span);
                                assert_property(&property, table)?;
                                if properties.iter().any(|p| *p == property.value) {
                                    return Err(ParserErr::RepeatedProperty(property));
                                }

                                properties.push(property.value);
                            }

                            let class_decl = ClassDecl::Defined(class_identifier, properties);
                            Ok((tokens, Located::new(class_decl, span)))
                        }
                        Token::Keyword(OR) => {
                            tokens.next();
                            let (remaining, class) = parse_class_identifier(tokens)?;
                            tokens = remaining;
                            let mut span = span.merge(&class.span);
                            if class_identifier == class.value {
                                return Err(ParserErr::RepeatedClass(class));
                            }
                            let mut classes = vec![class_identifier, class.value];

                            while let Some(Token::Keyword(OR)) = tokens.peek().map(|t| &t.value) {
                                tokens.next();
                                let (remaining, class) = parse_class_identifier(tokens)?;
                                tokens = remaining;
                                span = span.merge(&class.span);
                                if classes.iter().any(|c| *c == class.value) {
                                    return Err(ParserErr::RepeatedClass(class));
                                }
                                classes.push(class.value);
                            }

                            let class = ClassDecl::Disjoint(classes);
                            Ok((tokens, Located::new(class, span)))
                        }
                        Token::PropertyIdentifier { .. } => {
                            let (remaining, property) = parse_property(tokens, table)?;
                            tokens = remaining;
                            let mut span = span.merge(&property.span);
                            assert_property(&property, table)?;
                            let mut properties = vec![property.value];

                            while let Some(Token::PropertyIdentifier { .. }) =
                                tokens.peek().map(|t| &t.value)
                            {
                                let (remaining, property) = parse_property(tokens, table)?;
                                tokens = remaining;
                                span = span.merge(&property.span);
                                assert_property(&property, table)?;
                                if properties.iter().any(|p| *p == property.value) {
                                    return Err(ParserErr::RepeatedProperty(property));
                                }

                                properties.push(property.value);
                            }

                            let class_decl = ClassDecl::Primitive(class_identifier, properties);
                            Ok((tokens, Located::new(class_decl, span)))
                        }
                        _ => Err(ParserErr::UnrecognizedToken {
                            expected: "'AND' or 'OR' or PropertyIdentifier",
                            found: token,
                        }),
                    },
                    None => Err(ParserErr::UnexpectedEndOfInput),
                }
            }
            Token::Punctuation(OpenBrace) => parse_enumerated_class(tokens)
                .map(|(tokens, classes)| (tokens, classes.map(|cs| ClassDecl::Enumerated(cs)))),
            _ => Err(ParserErr::UnrecognizedToken {
                expected: "ClassIdentifier or '(' or '{'",
                found: token,
            }),
        },
        None => Err(ParserErr::UnexpectedEndOfInput),
    }
}

fn parse_property<'t>(
    mut tokens: Tokens<'t>,
    table: &mut SymbolTable,
) -> ParserResult<'t, Property> {
    match tokens.next() {
        Some(token) => match token.value {
            Token::Punctuation(OpenParen) => {
                let (mut tokens, property) = parse_property(tokens.clone(), table)?;
                let span = token.span.merge(&property.span);

                match tokens.next() {
                    Some(token) => match token.value {
                        Token::Punctuation(CloseParen) => {
                            let span = span.merge(&token.span);
                            Ok((tokens, Located::new(property.value, span)))
                        }
                        _ => Err(ParserErr::UnrecognizedToken {
                            expected: "')'",
                            found: token,
                        }),
                    },
                    None => Err(ParserErr::UnexpectedEndOfInput),
                }
            }
            Token::PropertyIdentifier { index } => {
                let mut span = token.span;
                match tokens.next() {
                    Some(token) => match token.value {
                        Token::Keyword(SOME) => match parse_class(tokens.clone(), table) {
                            Ok((tokens, class)) => {
                                span = span.merge(&class.span);
                                let description = PropertyDescription::Object(
                                    ObjectDescription::Some(class.value),
                                );
                                let property = Property { index, description };
                                Ok((tokens, Located::new(property, span)))
                            }
                            Err(ParserErr::UnrecognizedToken { .. }) => {
                                match parse_data(tokens.clone()) {
                                    Ok((tokens, data)) => {
                                        span = span.merge(&data.span);
                                        let description = PropertyDescription::Data(
                                            DataDescription::Some(data.value),
                                        );
                                        let property = Property { index, description };
                                        Ok((tokens, Located::new(property, span)))
                                    }
                                    Err(err) => Err(err),
                                }
                            }
                            Err(err) => Err(err),
                        },
                        Token::Keyword(ONLY) => match parse_class(tokens.clone(), table) {
                            Ok((tokens, class)) => {
                                span = span.merge(&class.span);
                                let description = PropertyDescription::Object(
                                    ObjectDescription::Only(class.value),
                                );
                                let property = Property { index, description };
                                Ok((tokens, Located::new(property, span)))
                            }
                            Err(ParserErr::UnrecognizedToken { .. }) => {
                                match parse_data(tokens.clone()) {
                                    Ok((tokens, data)) => {
                                        span = span.merge(&data.span);
                                        let description = PropertyDescription::Data(
                                            DataDescription::Only(data.value),
                                        );
                                        let property = Property { index, description };
                                        Ok((tokens, Located::new(property, span)))
                                    }
                                    Err(err) => Err(err),
                                }
                            }
                            Err(err) => Err(err),
                        },
                        Token::Keyword(VALUE) => match parse_class_identifier(tokens.clone()) {
                            Ok((tokens, class)) => {
                                span = span.merge(&class.span);
                                let description = PropertyDescription::Object(
                                    ObjectDescription::Value(class.value),
                                );
                                let property = Property { index, description };
                                Ok((tokens, Located::new(property, span)))
                            }
                            Err(ParserErr::UnrecognizedToken { .. }) => {
                                match parse_literal(tokens.clone()) {
                                    Ok((tokens, data)) => {
                                        span = span.merge(&data.span);
                                        let description = PropertyDescription::Data(
                                            DataDescription::Value(data.value),
                                        );
                                        let property = Property { index, description };
                                        Ok((tokens, Located::new(property, span)))
                                    }
                                    Err(err) => Err(err),
                                }
                            }
                            Err(err) => Err(err),
                        },
                        Token::Keyword(MIN) => {
                            let Some(Token::Cardinality(min)) = tokens.next().map(|t| &t.value) else {
                                return Err(ParserErr::UnexpectedEndOfInput);
                            };

                            span = span.merge(&token.span);

                            match parse_class(tokens.clone(), table) {
                                Ok((tokens, class)) => {
                                    span = span.merge(&class.span);
                                    let description = PropertyDescription::Object(
                                        ObjectDescription::Min(*min, class.value),
                                    );
                                    let property = Property { index, description };
                                    Ok((tokens, Located::new(property, span)))
                                }
                                Err(ParserErr::UnrecognizedToken { .. }) => {
                                    match parse_data(tokens.clone()) {
                                        Ok((tokens, data)) => {
                                            span = span.merge(&data.span);
                                            let description = PropertyDescription::Data(
                                                DataDescription::Min(*min, data.value),
                                            );
                                            let property = Property { index, description };
                                            Ok((tokens, Located::new(property, span)))
                                        }
                                        Err(err) => Err(err),
                                    }
                                }
                                Err(err) => Err(err),
                            }
                        }
                        Token::Keyword(MAX) => {
                            let Some(Token::Cardinality(max)) = tokens.next().map(|t| &t.value) else {
                                return Err(ParserErr::UnexpectedEndOfInput);
                            };

                            span = span.merge(&token.span);

                            match parse_class(tokens.clone(), table) {
                                Ok((tokens, class)) => {
                                    span = span.merge(&class.span);
                                    let description = PropertyDescription::Object(
                                        ObjectDescription::Max(*max, class.value),
                                    );
                                    let property = Property { index, description };
                                    Ok((tokens, Located::new(property, span)))
                                }
                                Err(ParserErr::UnrecognizedToken { .. }) => {
                                    match parse_data(tokens.clone()) {
                                        Ok((tokens, data)) => {
                                            span = span.merge(&data.span);
                                            let description = PropertyDescription::Data(
                                                DataDescription::Max(*max, data.value),
                                            );
                                            let property = Property { index, description };
                                            Ok((tokens, Located::new(property, span)))
                                        }
                                        Err(err) => Err(err),
                                    }
                                }
                                Err(err) => Err(err),
                            }
                        }
                        Token::Keyword(EXACTLY) => {
                            let Some(Token::Cardinality(exactly)) = tokens.next().map(|t| &t.value) else {
                                return Err(ParserErr::UnexpectedEndOfInput);
                            };

                            span = span.merge(&token.span);

                            match parse_class(tokens.clone(), table) {
                                Ok((tokens, class)) => {
                                    span = span.merge(&class.span);
                                    let description = PropertyDescription::Object(
                                        ObjectDescription::Exactly(*exactly, class.value),
                                    );
                                    let property = Property { index, description };
                                    Ok((tokens, Located::new(property, span)))
                                }
                                Err(ParserErr::UnrecognizedToken { .. }) => {
                                    match parse_data(tokens.clone()) {
                                        Ok((tokens, data)) => {
                                            span = span.merge(&data.span);
                                            let description = PropertyDescription::Data(
                                                DataDescription::Exactly(*exactly, data.value),
                                            );
                                            let property = Property { index, description };
                                            Ok((tokens, Located::new(property, span)))
                                        }
                                        Err(err) => Err(err),
                                    }
                                }
                                Err(err) => Err(err),
                            }
                        }
                        _ => Err(ParserErr::UnrecognizedToken {
                            expected: "'SOME' or 'ONLY' or 'VALUE' or 'MIN' or 'MAX' or 'EXACTLY'",
                            found: token,
                        }),
                    },
                    None => Err(ParserErr::UnexpectedEndOfInput),
                }
            }
            _ => Err(ParserErr::UnrecognizedToken {
                expected: "'(' or PropertyIdentifier",
                found: token,
            }),
        },
        None => Err(ParserErr::UnexpectedEndOfInput),
    }
}

fn parse_enumerated_class<'t>(mut tokens: Tokens<'t>) -> ParserResult<'t, Vec<ClassIdentifier>> {
    match tokens.next() {
        Some(token) => match token.value {
            Token::Punctuation(OpenBrace) => {
                let (remaining, class) = parse_class_identifier(tokens)?;
                tokens = remaining;
                let mut span = token.span.merge(&class.span);
                let mut classes = vec![class.value];

                while let Some(Token::Punctuation(Comma)) = tokens.peek().map(|t| &t.value) {
                    tokens.next();
                    let (remaining, class) = parse_class_identifier(tokens)?;
                    tokens = remaining;
                    span = span.merge(&class.span);
                    if classes.iter().any(|c| *c == class.value) {
                        return Err(ParserErr::RepeatedClass(class));
                    }
                    classes.push(class.value);
                }

                match tokens.next() {
                    Some(token) => match token.value {
                        Token::Punctuation(CloseBrace) => {
                            let span = span.merge(&token.span);
                            Ok((tokens, Located::new(classes, span)))
                        }
                        _ => Err(ParserErr::UnrecognizedToken {
                            expected: "'}'",
                            found: token,
                        }),
                    },
                    None => Err(ParserErr::UnexpectedEndOfInput),
                }
            }
            _ => Err(ParserErr::UnrecognizedToken {
                expected: "'{'",
                found: token,
            }),
        },
        None => Err(ParserErr::UnexpectedEndOfInput),
    }
}

fn parse_class_identifier<'t>(mut tokens: Tokens<'t>) -> ParserResult<'t, ClassIdentifier> {
    match tokens.next() {
        Some(token) => match token.value {
            Token::ClassIdentifier { index } => {
                let class_identifier = ClassIdentifier { index };
                let span = token.span;
                Ok((tokens, Located::new(class_identifier, span)))
            }
            _ => Err(ParserErr::UnrecognizedToken {
                expected: "ClassIdentifier",
                found: token,
            }),
        },
        None => Err(ParserErr::UnexpectedEndOfInput),
    }
}

fn parse_class<'t>(mut tokens: Tokens<'t>, table: &mut SymbolTable) -> ParserResult<'t, Class> {
    match tokens.peek().cloned() {
        Some(token) => match token.value {
            Token::ClassIdentifier { index } => {
                tokens.next();
                let class_identifier = ClassIdentifier { index };
                let span = token.span;

                match tokens.peek().cloned() {
                    Some(token) => match token.value {
                        Token::Keyword(AND) => {
                            tokens.next();
                            let (remaining, property) = parse_property(tokens, table)?;
                            tokens = remaining;
                            let mut span = span.merge(&property.span);

                            assert_property(&property, table)?;

                            let mut properties = vec![property.value];

                            while let Some(Token::Keyword(AND)) = tokens.peek().map(|t| &t.value) {
                                tokens.next();
                                let (remaining, property) = parse_property(tokens, table)?;
                                tokens = remaining;
                                span = span.merge(&property.span);
                                assert_property(&property, table)?;
                                if properties.iter().any(|p| *p == property.value) {
                                    return Err(ParserErr::RepeatedProperty(property));
                                }

                                properties.push(property.value);
                            }

                            let class = Class::Defined(class_identifier, properties);
                            Ok((tokens, Located::new(class, span)))
                        }
                        Token::Keyword(OR) => {
                            tokens.next();
                            let (remaining, class) = parse_class_identifier(tokens)?;
                            tokens = remaining;
                            let mut span = span.merge(&class.span);
                            if class_identifier == class.value {
                                return Err(ParserErr::RepeatedClass(class));
                            }
                            let mut classes = vec![class_identifier, class.value];

                            while let Some(Token::Keyword(OR)) = tokens.peek().map(|t| &t.value) {
                                tokens.next();
                                let (remaining, class) = parse_class_identifier(tokens)?;
                                tokens = remaining;
                                span = span.merge(&class.span);
                                if classes.iter().any(|c| *c == class.value) {
                                    return Err(ParserErr::RepeatedClass(class));
                                }
                                classes.push(class.value);
                            }

                            let class = Class::Disjoint(classes);
                            Ok((tokens, Located::new(class, span)))
                        }
                        _ => {
                            let class = Class::Identifier(class_identifier);
                            Ok((tokens, Located::new(class, span)))
                        }
                    },
                    None => Err(ParserErr::UnexpectedEndOfInput),
                }
            }
            Token::Punctuation(OpenParen) => {
                tokens.next();
                let (remaining, class) = parse_class(tokens, table)?;
                tokens = remaining;
                let mut span = token.span.merge(&class.span);

                match tokens.next() {
                    Some(token) if matches!(token.value, Token::Punctuation(CloseParen)) => {
                        span = span.merge(&token.span);
                        Ok((tokens, Located::new(class.value, span)))
                    }
                    Some(token) => Err(ParserErr::UnrecognizedToken {
                        expected: "')'",
                        found: token,
                    }),
                    None => Err(ParserErr::UnexpectedEndOfInput),
                }
            }
            Token::Punctuation(OpenBrace) => parse_enumerated_class(tokens)
                .map(|(tokens, classes)| (tokens, classes.map(|cs| Class::Enumerated(cs)))),
            _ => Err(ParserErr::UnrecognizedToken {
                expected: "ClassIdentifier or '(' or '{'",
                found: token,
            }),
        },
        None => Err(ParserErr::UnexpectedEndOfInput),
    }
}

fn parse_data<'t>(mut tokens: Tokens<'t>) -> ParserResult<'t, Data> {
    match tokens.next() {
        Some(token) => match token.value {
            Token::Datatype(datatype) => {
                let mut data = Data {
                    datatype,
                    restriction: None,
                };

                let mut span = token.span;

                match tokens.peek() {
                    Some(token) if matches!(token.value, Token::Punctuation(OpenBracket)) => {
                        tokens.next();
                        let restriction = match tokens.next() {
                            Some(token) => match token.value {
                                Token::Punctuation(LessThanEqual) => RestrictionType::LessThanEqual,
                                Token::Punctuation(LessThan) => RestrictionType::LessThan,
                                Token::Punctuation(GreaterThanEqual) => {
                                    RestrictionType::GreaterThanEqual
                                }
                                Token::Punctuation(GreaterThan) => RestrictionType::GreaterThan,
                                _ => {
                                    return Err(ParserErr::UnrecognizedToken {
                                        expected: "'<' or '<=' or '>' or '>='",
                                        found: token,
                                    })
                                }
                            },
                            None => return Err(ParserErr::UnexpectedEndOfInput),
                        };

                        let (mut tokens, literal) = parse_literal(tokens)?;

                        data.restriction = Some(Restriction(restriction, literal.value));
                        match tokens.next() {
                            Some(token) => match token.value {
                                Token::Punctuation(CloseBracket) => {
                                    span = span.merge(&token.span);
                                    Ok((tokens, Located::new(data, span)))
                                }
                                _ => Err(ParserErr::UnrecognizedToken {
                                    expected: "']'",
                                    found: token,
                                }),
                            },
                            None => Err(ParserErr::UnexpectedEndOfInput),
                        }
                    }
                    _ => Ok((tokens, Located::new(data, span))),
                }
            }
            _ => Err(ParserErr::UnrecognizedToken {
                expected: "Datatype",
                found: token,
            }),
        },
        None => Err(ParserErr::UnexpectedEndOfInput),
    }
}

fn parse_literal<'t>(mut tokens: Tokens<'t>) -> ParserResult<'t, Literal> {
    match tokens.next() {
        Some(token) => match token.value {
            Token::Literal { index } => {
                let literal = Literal { index };
                let span = token.span;
                Ok((tokens, Located::new(literal, span)))
            }

            _ => Err(ParserErr::UnrecognizedToken {
                expected: "Literal",
                found: token,
            }),
        },
        None => Err(ParserErr::UnexpectedEndOfInput),
    }
}

fn assert_property<'t>(
    property: &Located<'t, Property>,
    table: &mut SymbolTable,
) -> Result<(), ParserErr<'t>> {
    if let Some(symbol) = table.get(property.value.index).cloned() {
        let found = Type::Property(Some(property.value.type_()));
        match symbol.type_() {
            Type::Property(type_) => {
                let type_ = type_.unwrap_or_else(|| {
                    table.update_property_type(property.value.index, property.value.type_());
                    property.value.type_()
                });

                if type_ != property.value.type_() {
                    return Err(ParserErr::TypeMismatch {
                        location: property.span,
                        expected: symbol.type_(),
                        found,
                    });
                }
            }
            type_ => {
                return Err(ParserErr::TypeMismatch {
                    location: property.span,
                    expected: type_,
                    found,
                })
            }
        }
    }
    Ok(())
}

impl Debug for ClassDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defined(super_class, properties) => f
                .debug_struct("DefinedClass")
                .field("super_class", super_class)
                .field("properties", properties)
                .finish(),
            Self::Primitive(super_class, properties) => f
                .debug_struct("PrimitiveClass")
                .field("super_class", super_class)
                .field("properties", properties)
                .finish(),
            Self::Enumerated(classes) => f.debug_tuple("EnumeratedClass").field(classes).finish(),
            Self::Disjoint(classes) => f.debug_tuple("DisjointClass").field(classes).finish(),
        }
    }
}

impl Debug for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(class) => class.fmt(f),
            Self::Defined(super_class, properties) => f
                .debug_struct("DefinedClass")
                .field("super_class", super_class)
                .field("properties", properties)
                .finish(),
            Self::Enumerated(classes) => f.debug_tuple("EnumeratedClass").field(classes).finish(),
            Self::Disjoint(classes) => f.debug_tuple("DisjointClass").field(classes).finish(),
        }
    }
}

impl Debug for ClassIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClassIdentifier {{ index: {} }}", self.index)
    }
}

impl Debug for PropertyDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object(description) => description.fmt(f),
            Self::Data(description) => description.fmt(f),
        }
    }
}

impl Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Literal {{ index: {} }}", self.index)
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.restriction {
            Some(restriction) => f
                .debug_struct("Data")
                .field("datatype", &self.datatype)
                .field("restriction", restriction)
                .finish(),
            None => f
                .debug_struct("Data")
                .field("datatype", &self.datatype)
                .finish(),
        }
    }
}

impl Debug for ParserErr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEndOfInput => write!(f, "UnexpectedEndOfInput"),
            Self::RepeatedProperty(property) => {
                write!(
                    f,
                    "{}: RepeatedProperty: {:?}",
                    property.span.location(),
                    property.span.fragment()
                )
            }
            Self::RepeatedClass(arg0) => {
                write!(
                    f,
                    "{}: RepeatedClass: {:?}",
                    arg0.span.location(),
                    arg0.span.fragment()
                )
            }
            Self::UnrecognizedToken { expected, found } => {
                write!(
                    f,
                    "{}: UnrecognizedToken: expected {}, found {:?}",
                    found.span.location(),
                    expected,
                    found.value
                )
            }
            Self::TypeMismatch {
                location,
                expected,
                found,
            } => {
                write!(
                    f,
                    "{}: TypeMismatch: expected {:?}, found {:?} at '{}'",
                    location.location(),
                    expected,
                    found,
                    location.fragment()
                )
            }
        }
    }
}

impl Property {
    pub fn type_(&self) -> PropertyType {
        match &self.description {
            PropertyDescription::Object(_) => PropertyType::Object,
            PropertyDescription::Data(_) => PropertyType::Data,
        }
    }
}
