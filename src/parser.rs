use serde::{Deserialize, Serialize};
use std::{borrow::BorrowMut, collections::HashMap};
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

        let text_vec = self.text_stack.pop().unwrap();
        let text = text_vec.join("|");

        let stack_depth = self.element_stack.len();
        match stack_depth {
            0 => {
                // root element, push to body
                let mut root =
                    elem.is_map_or(Error::MalformedXml("Root element cannot be a single value"))?;
                if self.body.is_empty() {
                    // xml dec not present, so we add a hash map for the root key
                    let root_key = root
                        .remove("$key")
                        .ok_or(Error::MalformedXml("Root element must have a key"))?
                        .is_value_or(Error::MalformedXml("Root key must be a string"))?;
                    self.body.push(HashMap::from([(
                        String::from("rootTagName"),
                        RecursiveHashMap::Value(root_key),
                    )]))
                }
                self.body.push(root)
            }
            _ => {
                // nested element, pull out key, nest on next item in stack
                let mut nested = elem.is_map_or(Error::MalformedXml("Not sure how we got here"))?;
                let key = nested
                    .remove("$key")
                    .unwrap()
                    .is_value_or(Error::MalformedXml("$key must be a string"))?;

                if text.is_empty() && nested.is_empty() {
                    // text is empty and nested is empty (skip)
                    return Ok(());
                }

                let mut updated = self
                    .element_stack
                    .pop()
                    .unwrap()
                    .is_map_or(Error::MalformedXml("Cannot nest on value"))?;

                if !text.is_empty() && nested.is_empty() {
                    // We have text and nested is  empty (append text as value to updated)
                    updated.insert(key, RecursiveHashMap::Value(text));
                } else if !text.is_empty() && !nested.is_empty() {
                    // we have text and nested is not empty (insert text as ..cdata in nested before updating)
                    nested.insert(String::from("..cdata"), RecursiveHashMap::Value(text));
                    updated.insert(key, RecursiveHashMap::Map(nested));
                } else if text.is_empty() && !nested.is_empty() {
                    updated.insert(key, RecursiveHashMap::Map(nested));
                }

                self.element_stack.push(RecursiveHashMap::Map(updated));
            }
        }

        Ok(())
    }

    fn handle_elem_end(&mut self, end: ElementEnd, span: &str) -> Result<(), Error> {
        match end {
            ElementEnd::Close(_prefix, local) => self.handle_close_empty(&local),
            ElementEnd::Empty => self.handle_close_empty("/>"),
            ElementEnd::Open => Ok(()), // don't need to do anything if left open
        }
    }

    fn handle_attribute(
        &mut self,
        prefix: &str,
        local: &str,
        value: &str,
        span: &str,
    ) -> Result<(), Error> {
        let mut elem = self
            .element_stack
            .pop()
            .ok_or(Error::MalformedXml(
                "Received attributes outside of an open tag.",
            ))?
            .is_map_or(Error::MalformedXml("Cannot apply attributes to value"))?;
        let key = [".", local].join("");

        elem.insert(key, RecursiveHashMap::Value(String::from(value.trim())));
        self.element_stack.push(RecursiveHashMap::Map(elem));
        Ok(())
    }

    fn handle_text(&mut self, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
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
            } => self.handle_attribute(&prefix, &local, &value, &span)?,
            Token::ElementEnd { end, span } => self.handle_elem_end(end, &span)?,
            Token::Text { text } => self.handle_text(&text),
            _ => (),
        };
        Ok(())
    }
}

pub fn parse(doc: String) -> Result<Vec<HashMap<String, RecursiveHashMap>>, Error> {
    let mut handler = TokenHanlder::new();
    for token in xmlparser::Tokenizer::from(&*doc) {
        handler.handle_token(token)?;
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
