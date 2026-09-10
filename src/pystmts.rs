use pyo3::prelude::*;
use swc_core::common::{Span, SyntaxContext};
use swc_core::ecma::ast::*;

use crate::conversions::*;
use crate::macros::ast_node_variant;
use crate::pyexpr::PyExpr;
use crate::pyident::PyIdent;
use crate::pypat::PyPat;
use crate::pyspan::PySpan;

#[derive(Clone)]
#[pyclass(subclass)]
pub struct PyStmt {
    pub stmt: Stmt,
}
#[pyclass]
pub struct PyCatchClause {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub param: Option<Py<PyPat>>,
    #[pyo3(get)]
    pub body: Py<PyBlockStmt>,
}

ast_node_variant!(PyStmt, PyBlockStmt, BlockStmt, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    stmts: Vec<Py<PyStmt>> = conv_stmts,
});

ast_node_variant!(PyStmt, PyEmptyStmt, EmptyStmt, {
    span: PySpan = conv_span
});

ast_node_variant!(PyStmt, PyTryStmt, TryStmt, {
    span: PySpan = conv_span,
    block: Py<PyBlockStmt> = conv_block_stmt,
    handler: Option<Py<PyCatchClause>> = conv_option_catch_clause,
    finalizer: Option<Py<PyBlockStmt>> = conv_option_block_stmt
});

ast_node_variant!(PyStmt, PyExprStmt, ExprStmt, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyStmt, PyDebuggerStmt, DebuggerStmt, {
    span: PySpan = conv_span
});

ast_node_variant!(PyStmt, PyWithStmt, WithStmt, {
    span: PySpan = conv_span,
    obj: Py<PyExpr> = conv_boxed_expr,
    body: Py<PyStmt> = conv_boxed_stmt
});

ast_node_variant!(PyStmt, PyReturnStmt, ReturnStmt, {
    span: PySpan = conv_span,
    arg: Option<Py<PyExpr>> = conv_option_boxed_expr
});

ast_node_variant!(PyStmt, PyLabeledStmt, LabeledStmt, {
    span: PySpan = conv_span,
    label: PyIdent = conv_ident,
    body: Py<PyStmt> = conv_boxed_stmt
});

ast_node_variant!(PyStmt, PyBreakStmt, BreakStmt, {
    span: PySpan = conv_span,
    label: Option<PyIdent> = conv_option_ident
});

ast_node_variant!(PyStmt, PyContinueStmt, ContinueStmt, {
    span: PySpan = conv_span,
    label: Option<PyIdent> = conv_option_ident
});

ast_node_variant!(PyStmt, PyIfStmt, IfStmt, {
    span: PySpan = conv_span,
    test: Py<PyExpr> = conv_boxed_expr,
    cons: Py<PyStmt> = conv_boxed_stmt,
    alt: Option<Py<PyStmt>> = conv_option_boxed_stmt
});

#[pyclass]
pub struct PySwitchCase {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub test: Option<Py<PyExpr>>,
    #[pyo3(get)]
    pub cons: Vec<Py<PyStmt>>,
}

ast_node_variant!(PyStmt, PySwitchStmt, SwitchStmt, {
    span: PySpan = conv_span,
    body_ctxt: u32 = conv_ctxt,
    discriminant: Py<PyExpr> = conv_boxed_expr,
    cases: Vec<Py<PySwitchCase>> = conv_switch_cases
});

ast_node_variant!(PyStmt, PyThrowStmt, ThrowStmt, {
    span: PySpan = conv_span,
    arg: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyStmt, PyWhileStmt, WhileStmt, {
    span: PySpan = conv_span,
    test: Py<PyExpr> = conv_boxed_expr,
    body: Py<PyStmt> = conv_boxed_stmt
});

ast_node_variant!(PyStmt, PyDoWhileStmt, DoWhileStmt, {
    span: PySpan = conv_span,
    test: Py<PyExpr> = conv_boxed_expr,
    body: Py<PyStmt> = conv_boxed_stmt
});

#[pyclass]
pub struct PyVarDeclOrExpr {
    #[pyo3(get)]
    pub var_decl: Option<Py<crate::pydecl::PyVarDecl>>,
    #[pyo3(get)]
    pub expr: Option<Py<PyExpr>>,
}

ast_node_variant!(PyStmt, PyForStmt, ForStmt, {
    span: PySpan = conv_span,
    init: Option<Py<PyVarDeclOrExpr>> = conv_option_var_decl_or_expr,
    test: Option<Py<PyExpr>> = conv_option_boxed_expr,
    update: Option<Py<PyExpr>> = conv_option_boxed_expr,
    body: Py<PyStmt> = conv_boxed_stmt
});

#[pyclass]
pub struct PyForHead {
    #[pyo3(get)]
    pub var_decl: Option<Py<crate::pydecl::PyVarDecl>>,
    #[pyo3(get)]
    pub using_decl: Option<Py<crate::pydecl::PyUsingDecl>>,
    #[pyo3(get)]
    pub pat: Option<Py<PyPat>>,
}

ast_node_variant!(PyStmt, PyForInStmt, ForInStmt, {
    span: PySpan = conv_span,
    left: Py<PyForHead> = conv_for_head,
    right: Py<PyExpr> = conv_boxed_expr,
    body: Py<PyStmt> = conv_boxed_stmt
});

ast_node_variant!(PyStmt, PyForOfStmt, ForOfStmt, {
    span: PySpan = conv_span,
    is_await: bool = conv_bool,
    left: Py<PyForHead> = conv_for_head,
    right: Py<PyExpr> = conv_boxed_expr,
    body: Py<PyStmt> = conv_boxed_stmt
});

#[pyclass(extends=PyStmt)]
pub struct PyDeclStmt {
    #[pyo3(get)]
    pub decl: Py<crate::pydecl::PyDecl>,
}

impl PyDeclStmt {
    pub fn build(py: Python<'_>, node: Decl) -> PyResult<Self> {
        Ok(PyDeclStmt {
            decl: crate::pydecl::conv_decl(py, node)?,
        })
    }
}

pub fn stmt_to_py(py: Python<'_>, stmt: Stmt) -> PyResult<Py<PyStmt>> {
    let base = PyStmt { stmt: stmt.clone() };
    Ok(match stmt {
        Stmt::Block(s) => Py::new(py, (PyBlockStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::Empty(s) => Py::new(py, (PyEmptyStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::Debugger(s) => Py::new(py, (PyDebuggerStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::With(s) => Py::new(py, (PyWithStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::Return(s) => Py::new(py, (PyReturnStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::Labeled(s) => Py::new(py, (PyLabeledStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::Break(s) => Py::new(py, (PyBreakStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::Continue(s) => Py::new(py, (PyContinueStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::If(s) => Py::new(py, (PyIfStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::Switch(s) => Py::new(py, (PySwitchStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::Throw(s) => Py::new(py, (PyThrowStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::Try(s) => Py::new(py, (PyTryStmt::build(py, *s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::While(s) => Py::new(py, (PyWhileStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::DoWhile(s) => Py::new(py, (PyDoWhileStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::For(s) => Py::new(py, (PyForStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::ForIn(s) => Py::new(py, (PyForInStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::ForOf(s) => Py::new(py, (PyForOfStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::Decl(s) => Py::new(py, (PyDeclStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        Stmt::Expr(s) => Py::new(py, (PyExprStmt::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
    })
}
