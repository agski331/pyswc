use crate::{pyspan::PySpan, pytypeinfo::PyTsTypeAnn};
use pyo3::prelude::*;
use swc_core::ecma::ast::Lit;

#[derive(Clone)]
#[pyclass]
pub struct PyIdentName {
    pub span: PySpan,
    pub sym: String,
}

#[derive(Clone)]
#[pyclass]
pub struct PyPrivateName {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub name: String,
}

#[derive(Clone)]
#[pyclass]
pub struct PyIdent {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub ctxt: u32,
    #[pyo3(get)]
    pub sym: String,
}

#[pyclass]
pub struct PyBindingIdent {
    #[pyo3(get)]
    pub id: PyIdent,
    #[pyo3(get)]
    pub type_ann: Option<Py<PyTsTypeAnn>>,
}
