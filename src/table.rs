use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    Object,
    Data,
}

#[derive(Clone, Copy)]
pub enum Type {
    Class,
    Property(Option<PropertyType>),
    Literal,
}

impl Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Class => write!(f, "Class"),
            Self::Property(type_) => {
                write!(f, "Property")?;
                if let Some(type_) = type_ {
                    write!(f, "({:?})", type_)?;
                }
                Ok(())
            }
            Self::Literal => write!(f, "Literal"),
        }
    }
}

#[derive(Clone)]
pub struct Symbol {
    type_: Type,
    id: String,
}

impl Symbol {
    pub fn new(type_: Type, id: String) -> Self {
        Self { type_, id }
    }

    pub fn type_(&self) -> Type {
        self.type_
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

pub struct SymbolTable {
    symbols: HashMap<usize, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Symbol> {
        self.symbols.get(&index)
    }

    pub fn get_or_insert(&mut self, type_: Type, id: String) -> usize {
        if let Some(index) = self
            .symbols
            .iter()
            .find_map(|(index, symbol)| (symbol.id() == id).then_some(index))
        {
            *index
        } else {
            let index = self.symbols.len();
            self.symbols.insert(index, Symbol::new(type_, id));
            index
        }
    }

    pub fn update_property_type(&mut self, index: usize, type_: PropertyType) -> bool {
        if let Some(symbol) = self.symbols.get_mut(&index) {
            if let Type::Property(None) = symbol.type_ {
                symbol.type_ = Type::Property(Some(type_));
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn symbols(&self) -> &HashMap<usize, Symbol> {
        &self.symbols
    }
}
