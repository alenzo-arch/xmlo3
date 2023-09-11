use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use xmlparser::{ElementEnd, Token};

use pyo3::{exceptions::PyRuntimeError, prelude::*};

#[derive(Debug)]
pub enum Error {
    /* */
    ParserError(xmlparser::Error),
    MalformedXml(&'static str),
}

impl From<Error> for PyErr {
    fn from(value: Error) -> Self {
        match value {
            Error::ParserError(value) => PyRuntimeError::new_err(value.to_string()),
            Error::MalformedXml(value) => PyRuntimeError::new_err(value),
        }
    }
}

impl From<xmlparser::Error> for Error {
    fn from(value: xmlparser::Error) -> Self {
        Self::ParserError(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RecursiveHashMap {
    Value(String),
    Map(HashMap<String, RecursiveHashMap>),
}

impl RecursiveHashMap {
    fn is_map_or<E>(self, err: E) -> Result<HashMap<String, RecursiveHashMap>, E> {
        match self {
            RecursiveHashMap::Map(map) => Ok(map),
            RecursiveHashMap::Value(_) => Err(err),
        }
    }
    fn is_value_or<E>(self, err: E) -> Result<String, E> {
        match self {
            RecursiveHashMap::Value(val) => Ok(val),
            RecursiveHashMap::Map(_) => Err(err),
        }
    }
}

struct TokenHanlder {
    data: RecursiveHashMap,
    element_stack: Vec<RecursiveHashMap>, // TODO: Combine these to Vec<(RecursiveHashMap,Vec<String>)>?
    text_stack: Vec<Vec<String>>,
    body: Vec<HashMap<String, RecursiveHashMap>>,
}
impl TokenHanlder {
    pub fn new() -> Self {
        Self {
            data: RecursiveHashMap::Map(HashMap::new()),
            element_stack: Vec::new(),
            text_stack: Vec::new(),
            body: Vec::new(),
        }
    }

    fn handle_elem_start(&mut self, _prefix: &str, local: &str, _span: &str) {
        self.element_stack
            .push(RecursiveHashMap::Map(HashMap::from([(
                String::from("$key"),
                RecursiveHashMap::Value(String::from(local)),
            )])));
        self.text_stack.push(Vec::new());
    }

    fn handle_close_empty(&mut self, _local: &str) -> Result<(), Error> {
        let elem = self.element_stack.pop().ok_or(Error::MalformedXml(
            "Closing element found without matching open element",
        ))?;
        println!("{:?}", elem);

        let text = self.text_stack.pop();

        let stack_depth = self.element_stack.len();
        match stack_depth {
            0 => {
                // root element, push to body
                let root =
                    elem.is_map_or(Error::MalformedXml("Root element cannot be a single value"))?;
                self.body.push(root)
            }
            _ => {
                // nested element, pull out key, nest on next item in stack
                let mut nested = elem.is_map_or(Error::MalformedXml("Not sure how we got here"))?;
                let key = nested
                    .remove("$key")
                    .unwrap()
                    .is_value_or(Error::MalformedXml("$key must be a string"))?;
                let mut updated = self
                    .element_stack
                    .pop()
                    .unwrap()
                    .is_map_or(Error::MalformedXml("Cannot nest on value"))?;

                updated.insert(key, RecursiveHashMap::Map(nested));
                self.element_stack.push(RecursiveHashMap::Map(updated));
            }
        }

        Ok(())
    }

    fn handle_elem_end(&mut self, end: ElementEnd, span: &str) {
        match end {
            ElementEnd::Close(_prefix, local) => self.handle_close_empty(&local),
            ElementEnd::Empty => self.handle_close_empty("/>"),
            ElementEnd::Open => Ok(()), // don't need to do anything if left open
        };
    }

    fn handle_text(&mut self, text: &str) {
        match self.text_stack.last_mut() {
            Some(last) => last.push(String::from(text)),
            None => {}
        }
    }

    fn handle_token(&mut self, token: Result<Token, xmlparser::Error>) -> Result<(), Error> {
        let token = token?;
        match token {
            Token::ElementStart {
                prefix,
                local,
                span,
            } => self.handle_elem_start(&prefix, &local, &span),
            Token::Attribute {
                prefix,
                local,
                value,
                span,
            } => (),
            Token::ElementEnd { end, span } => self.handle_elem_end(end, &span),
            Token::Text { text } => self.handle_text(&text),
            _ => (),
        };
        Ok(())
    }
}

pub fn parse(doc: String) -> Result<Vec<HashMap<String, RecursiveHashMap>>, Error> {
    let mut handler = TokenHanlder::new();
    for token in xmlparser::Tokenizer::from(&*doc) {
        println!("{:?}", token);
        handler.handle_token(token)?;
        println!("{:?}", handler.element_stack)
    }
    Ok(handler.body)
}

#[cfg(test)]
mod tests {
    use std::fs;
    #[test]
    fn it_works() {
        let raw = fs::read_to_string("files/nasa.xml").unwrap();
        assert_eq!(raw.len() > 0, true)
    }
}
