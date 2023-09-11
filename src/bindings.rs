use crate::parse;
use crate::RecursiveHashMap;
use pyo3::prelude::*;
use std::fs;

impl IntoPy<PyObject> for RecursiveHashMap {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            Self::Map(val) => val.into_py(py),
            Self::Value(val) => val.into_py(py),
        }
    }
}

#[pyfunction]
fn parse_file(path: &str) -> PyResult<PyObject> {
    let doc = fs::read_to_string(path).unwrap();
    Python::with_gil(|py| {
        let pyobj = parse(doc)?.into_py(py);
        Ok(pyobj)
    })
}

#[pyfunction]
fn parse_string(input: String) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        let pyobj = parse(input)?.into_py(py);
        Ok(pyobj)
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn xmlo3(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_string, m)?)?;
    Ok(())
}
