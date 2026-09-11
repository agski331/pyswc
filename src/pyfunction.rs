use pyo3::prelude::*;
use std::sync::Arc;
use swc_core::common::Span;
use swc_core::ecma::ast::{Decorator, Function, FunctionBody, Param, TsThisParam};

use crate::{
    pyenums::PyImportPhase,
    pyexpr::{ExprData, PyExpr, conv_arc_expr, lower_expr},
    pypat::{PatData, PyPat, conv_arc_pat, lower_pat},
    pyspan::PySpan,
    pystmts::{StmtData, conv_arc_stmts, lower_stmt},
    pytypeinfo::{
        PyTsTypeAnn, PyTsTypeParamDecl, TsTypeAnnData, TsTypeParamDeclData,
        conv_option_arc_ts_type_param_decl, conv_option_arc_tstypeann, lower_tstypeann,
        lower_ts_type_param_decl,
    },
};

#[derive(Clone)]
pub struct DecoratorData {
    pub span: Span,
    pub expr: Arc<ExprData>,
}

#[derive(Clone)]
pub struct ParamData {
    pub span: Span,
    pub decorators: Vec<DecoratorData>,
    pub pat: Arc<PatData>,
}

#[derive(Clone)]
pub struct TsThisParamData {
    pub span: Span,
    pub this_span: Span,
    pub type_ann: Option<Arc<TsTypeAnnData>>,
}

pub struct FunctionBodyData {
    pub span: Span,
    pub stmts: Vec<Arc<StmtData>>,
}

pub struct FunctionData {
    pub this_param: Option<TsThisParamData>,
    pub params: Vec<ParamData>,
    pub decorators: Vec<DecoratorData>,
    pub span: Span,
    pub ctxt: swc_core::common::SyntaxContext,
    pub body: Option<Arc<FunctionBodyData>>,
    pub is_generator: bool,
    pub is_async: bool,
    pub type_params: Option<Arc<TsTypeParamDeclData>>,
    pub return_type: Option<Arc<TsTypeAnnData>>,
}

pub fn lower_decorator(d: Decorator) -> DecoratorData {
    DecoratorData {
        span: d.span,
        expr: Arc::new(lower_expr(*d.expr)),
    }
}

pub fn lower_param(p: Param) -> ParamData {
    ParamData {
        span: p.span,
        decorators: p.decorators.into_iter().map(lower_decorator).collect(),
        pat: Arc::new(lower_pat(p.pat)),
    }
}

pub fn lower_ts_this_param(t: TsThisParam) -> TsThisParamData {
    TsThisParamData {
        span: t.span,
        this_span: t.this_span,
        type_ann: t.type_ann.map(|a| Arc::new(lower_tstypeann(*a))),
    }
}

pub fn lower_function_body(b: FunctionBody) -> FunctionBodyData {
    FunctionBodyData {
        span: b.span,
        stmts: b.stmts.into_iter().map(|s| Arc::new(lower_stmt(s))).collect(),
    }
}

pub fn lower_function(f: Function) -> FunctionData {
    FunctionData {
        this_param: f.this_param.map(|t| lower_ts_this_param(*t)),
        params: f.params.into_iter().map(lower_param).collect(),
        decorators: f.decorators.into_iter().map(lower_decorator).collect(),
        span: f.span,
        ctxt: f.ctxt,
        body: f.body.map(|b| Arc::new(lower_function_body(b))),
        is_generator: f.is_generator,
        is_async: f.is_async,
        type_params: f.type_params.map(|tp| Arc::new(lower_ts_type_param_decl(*tp))),
        return_type: f.return_type.map(|r| Arc::new(lower_tstypeann(*r))),
    }
}

#[pyclass]
pub struct PyDecorator {
    data: DecoratorData,
}

impl PyDecorator {
    pub fn from_data(data: DecoratorData) -> Self {
        PyDecorator { data }
    }
}

#[pymethods]
impl PyDecorator {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        crate::conversions::conv_span(py, self.data.span)
    }

    #[getter]
    fn expr(&self, py: Python<'_>) -> PyResult<Py<PyExpr>> {
        conv_arc_expr(py, Arc::clone(&self.data.expr))
    }
}

pub fn wrap_decorators(py: Python<'_>, data: Vec<DecoratorData>) -> PyResult<Vec<Py<PyDecorator>>> {
    data.into_iter()
        .map(|d| Py::new(py, PyDecorator::from_data(d)))
        .collect()
}

#[pyclass]
pub struct PyCallee {
    #[pyo3(get)]
    pub span: Option<PySpan>,
    #[pyo3(get)]
    pub is_super: bool,
    #[pyo3(get)]
    pub phase: Option<PyImportPhase>,
    #[pyo3(get)]
    pub expr: Option<Py<PyExpr>>,
}

#[pyclass]
pub struct PyTsThisParam {
    data: TsThisParamData,
}

impl PyTsThisParam {
    pub fn from_data(data: TsThisParamData) -> Self {
        PyTsThisParam { data }
    }
}

#[pymethods]
impl PyTsThisParam {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        crate::conversions::conv_span(py, self.data.span)
    }

    #[getter]
    fn this_span(&self, py: Python<'_>) -> PyResult<PySpan> {
        crate::conversions::conv_span(py, self.data.this_span)
    }

    #[getter]
    fn type_ann(&self, py: Python<'_>) -> PyResult<Option<Py<PyTsTypeAnn>>> {
        conv_option_arc_tstypeann(py, self.data.type_ann.clone())
    }
}

#[pyclass]
pub struct PyParam {
    data: ParamData,
}

impl PyParam {
    pub fn from_data(data: ParamData) -> Self {
        PyParam { data }
    }
}

#[pymethods]
impl PyParam {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        crate::conversions::conv_span(py, self.data.span)
    }

    #[getter]
    fn decorators(&self, py: Python<'_>) -> PyResult<Vec<Py<PyDecorator>>> {
        wrap_decorators(py, self.data.decorators.clone())
    }

    #[getter]
    fn pat(&self, py: Python<'_>) -> PyResult<Py<PyPat>> {
        conv_arc_pat(py, Arc::clone(&self.data.pat))
    }
}

pub fn wrap_params(py: Python<'_>, data: Vec<ParamData>) -> PyResult<Vec<Py<PyParam>>> {
    data.into_iter()
        .map(|p| Py::new(py, PyParam::from_data(p)))
        .collect()
}

#[pyclass]
pub struct PyFunctionBody {
    inner: Arc<FunctionBodyData>,
}

impl PyFunctionBody {
    pub fn from_arc(inner: Arc<FunctionBodyData>) -> Self {
        PyFunctionBody { inner }
    }
}

#[pymethods]
impl PyFunctionBody {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        crate::conversions::conv_span(py, self.inner.span)
    }

    #[getter]
    fn stmts(&self, py: Python<'_>) -> PyResult<Vec<Py<crate::pystmts::PyStmt>>> {
        conv_arc_stmts(py, self.inner.stmts.clone())
    }
}

pub fn wrap_function_body_data(
    py: Python<'_>,
    data: Arc<FunctionBodyData>,
) -> PyResult<Py<PyFunctionBody>> {
    Py::new(py, PyFunctionBody::from_arc(data))
}

pub fn conv_option_function_body_arc(
    py: Python<'_>,
    data: Option<Arc<FunctionBodyData>>,
) -> PyResult<Option<Py<PyFunctionBody>>> {
    data.map(|d| wrap_function_body_data(py, d)).transpose()
}

#[pyclass]
pub struct PyFunction {
    inner: Arc<FunctionData>,
}

impl PyFunction {
    pub fn from_arc(inner: Arc<FunctionData>) -> Self {
        PyFunction { inner }
    }
}

#[pymethods]
impl PyFunction {
    #[getter]
    fn this_param(&self, py: Python<'_>) -> PyResult<Option<Py<PyTsThisParam>>> {
        match &self.inner.this_param {
            Some(t) => Py::new(py, PyTsThisParam::from_data(t.clone())).map(Some),
            None => Ok(None),
        }
    }

    #[getter]
    fn params(&self, py: Python<'_>) -> PyResult<Vec<Py<PyParam>>> {
        wrap_params(py, self.inner.params.clone())
    }

    #[getter]
    fn decorators(&self, py: Python<'_>) -> PyResult<Vec<Py<PyDecorator>>> {
        wrap_decorators(py, self.inner.decorators.clone())
    }

    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        crate::conversions::conv_span(py, self.inner.span)
    }

    #[getter]
    fn ctxt(&self, py: Python<'_>) -> PyResult<u32> {
        crate::conversions::conv_ctxt(py, self.inner.ctxt)
    }

    #[getter]
    fn body(&self, py: Python<'_>) -> PyResult<Option<Py<PyFunctionBody>>> {
        conv_option_function_body_arc(py, self.inner.body.clone())
    }

    #[getter]
    fn is_generator(&self) -> bool {
        self.inner.is_generator
    }

    #[getter]
    fn is_async(&self) -> bool {
        self.inner.is_async
    }

    #[getter]
    fn type_params(&self, py: Python<'_>) -> PyResult<Option<Py<PyTsTypeParamDecl>>> {
        conv_option_arc_ts_type_param_decl(py, self.inner.type_params.clone())
    }

    #[getter]
    fn return_type(&self, py: Python<'_>) -> PyResult<Option<Py<PyTsTypeAnn>>> {
        conv_option_arc_tstypeann(py, self.inner.return_type.clone())
    }
}

pub fn wrap_function_data(py: Python<'_>, data: Arc<FunctionData>) -> PyResult<Py<PyFunction>> {
    Py::new(py, PyFunction::from_arc(data))
}

pub fn function_to_py(py: Python<'_>, func: Function) -> PyResult<Py<PyFunction>> {
    wrap_function_data(py, Arc::new(lower_function(func)))
}
