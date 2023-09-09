#[derive(Debug)]
enum Error {
    /*  */
    XmlParsingError(xmlparser::Error),
}

impl From<Error> for PyErr {
    fn from(value: Error) -> Self {
        match value {
            Error::XmlParsingError(value) => PyRuntimeError::new_err(value.to_string()),
        }
    }
}

impl From<xmlparser::Error> for Error {
    fn from(value: xmlparser::Error) -> Self {
        Self::XmlParsingError(value)
    }
}
