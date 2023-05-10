use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Class,
    Property,
    Cardinality,
    Literal,
}

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

    pub fn symbols(&self) -> &HashMap<usize, Symbol> {
        &self.symbols
    }
}
