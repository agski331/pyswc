use pyo3::prelude::*;
use swc_core::ecma::ast::ImportPhase;

use crate::{
    pyexpr::PyExpr,
    pypat::PyPat,
    pyspan::PySpan,
    pystmts::PyStmt,
    pytypeinfo::{PyTsTypeAnn, PyTsTypeParamDecl},
};
#[pyclass]
pub struct PyDecorator {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub expr: Py<PyExpr>,
}

#[pyclass]
pub struct PyCallee {
    pub span: Option<PySpan>,
    pub is_super: bool,
    pub phase: Option<ImportPhase>,
    pub expr: Option<Py<PyExpr>>,
}

#[pyclass]
pub struct PyTsThisParam {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub this_span: PySpan,
    #[pyo3(get)]
    pub type_ann: Option<Py<PyTsTypeAnn>>,
}

#[pyclass]
pub struct PyParam {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub decorators: Vec<Py<PyDecorator>>,
    #[pyo3(get)]
    pub pat: Py<PyPat>,
}

#[pyclass]
pub struct PyFunctionBody {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub stmts: Vec<Py<PyStmt>>,
}

#[pyclass]
pub struct PyFunction {
    #[pyo3(get)]
    pub this_param: Option<Py<PyTsThisParam>>,
    #[pyo3(get)]
    pub params: Vec<Py<PyParam>>,
    #[pyo3(get)]
    pub decorators: Vec<Py<PyDecorator>>,
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub ctxt: u32,
    #[pyo3(get)]
    pub body: Option<Py<PyFunctionBody>>,
    #[pyo3(get)]
    pub is_generator: bool,
    #[pyo3(get)]
    pub is_async: bool,
    #[pyo3(get)]
    pub type_params: Option<Py<PyTsTypeParamDecl>>,
    #[pyo3(get)]
    pub return_type: Option<Py<PyTsTypeAnn>>,
}
