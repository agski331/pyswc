use num_bigint;
use pyo3::prelude::*;
use swc_core::ecma::ast::Lit;
use swc_core::ecma::ast::Lit::Regex;

#[pyclass]
pub struct PyLit {
    pub lit: Lit,
}

#[pymethods]
impl PyLit {
    #[getter]
    fn as_regex(&self) -> Option<String> {
        if let Some(regex) = self.lit.as_regex() {
            return Some(regex.exp.to_string());
        }
        return None;
    }

    #[getter]
    fn as_string(&self) -> Option<String> {
        if let Some(s) = self.lit.as_str() {
            return Some(s.value.to_atom_lossy().to_string());
        }
        return None;
    }

    #[getter]
    fn as_bool(&self) -> Option<bool> {
        if let Some(b) = self.lit.as_bool() {
            return Some(b.value);
        }
        return None;
    }

    #[getter]
    fn as_jsxtext(&self) -> Option<String> {
        if let Some(jsx) = self.lit.as_jsx_text() {
            return Some(jsx.value.to_atom_lossy().to_string());
        }
        return None;
    }

    #[getter]
    fn is_null(&self, py: Python<'_>) -> bool {
        if self.lit.as_null().is_some() {
            return true;
        }
        return false;
    }

    #[getter]
    fn as_num(&self) -> Option<f64> {
        if let Some(num) = self.lit.as_num() {
            return Some(num.value);
        }
        return None;
    }

    #[getter]
    fn as_bigint(&self) -> Option<num_bigint::BigInt> {
        if let Some(bg) = self.lit.as_big_int() {
            let big_int: num_bigint::BigInt = (*bg.value).clone();
            return Some(big_int);
        }
        return None;
    }
}
