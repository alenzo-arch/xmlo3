use pyo3::{exceptions::PyRuntimeError, prelude::*};
use std::fs;
use xmlparser::Token;

#[derive(Debug)]
enum Error {
    /* */
    XMLError(xmlparser::Error),
}

impl From<Error> for PyErr {
    fn from(value: Error) -> Self {
        match value {
            Error::XMLError(value) => PyRuntimeError::new_err(value.to_string()),
        }
    }
}

impl From<xmlparser::Error> for Error {
    fn from(value: xmlparser::Error) -> Self {
        Self::XMLError(value)
    }
}

#[pyfunction]
fn parse_file_roxmltree(path: &str) -> PyResult<()> {
    let input = fs::read_to_string(path)?;
    if let Ok(doc) = roxmltree::Document::parse(&input) {
        println!("parsed from rust");
        Ok(())
    } else {
        Err(PyErr::new::<PyRuntimeError, _>("args"))
    }
}

fn process_tokens(token: Result<Token, xmlparser::Error>) -> Result<(), Error> {
    let token = token?;
    match token {
        Token::ElementStart {
            prefix,
            local,
            span,
        } => {}
        Token::Attribute {
            prefix,
            local,
            value,
            span,
        } => {}
        Token::ElementEnd { end, span } => {}
        Token::Text { text } => {}
        _ => (),
    }
    Ok(())
}

#[pyfunction]
fn parse_file_xmlparser(path: &str) -> PyResult<()> {
    let input = fs::read_to_string(path)?;
    for token in xmlparser::Tokenizer::from(&*input) {
        process_tokens(token)?;
    }
    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn xmlo3(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_file_roxmltree, m)?)?;
    m.add_function(wrap_pyfunction!(parse_file_xmlparser, m)?)?;
    Ok(())
}
