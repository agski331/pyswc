use pyo3::prelude::*;
use std::sync::Arc;
use swc_core::common::Span;
use swc_core::ecma::ast::*;

use crate::conversions::*;
use crate::macros::{arc_variant_node, ast_node_variant};
use crate::pyexpr::{ExprData, PyExpr, conv_arc_expr, conv_option_arc_expr, lower_expr};
use crate::pyident::PyIdent;
use crate::pypat::{PatData, PyPat, conv_arc_pat, lower_pat};
use crate::pyspan::PySpan;

#[pyclass(subclass)]
pub struct PyStmt {}


pub struct BlockStmtData {
    pub span: Span,
    pub ctxt: swc_core::common::SyntaxContext,
    pub stmts: Vec<Arc<StmtData>>,
}

pub enum StmtData {
    Block(Arc<BlockStmtData>),
    Empty(EmptyStmt),
    Debugger(DebuggerStmt),
    With(WithStmtData),
    Return(ReturnStmtData),
    Labeled(LabeledStmtData),
    Break(BreakStmt),
    Continue(ContinueStmt),
    If(IfStmtData),
    Switch(SwitchStmtData),
    Throw(ThrowStmtData),
    Try(TryStmtData),
    While(WhileStmtData),
    DoWhile(DoWhileStmtData),
    For(ForStmtData),
    ForIn(ForInStmtData),
    ForOf(ForOfStmtData),
    Decl(Arc<crate::pydecl::DeclData>),
    Expr(ExprStmtData),
}

pub struct WithStmtData {
    pub span: Span,
    pub obj: Arc<ExprData>,
    pub body: Arc<StmtData>,
}

pub struct ReturnStmtData {
    pub span: Span,
    pub arg: Option<Arc<ExprData>>,
}

pub struct LabeledStmtData {
    pub span: Span,
    pub label: Ident,
    pub body: Arc<StmtData>,
}

pub struct IfStmtData {
    pub span: Span,
    pub test: Arc<ExprData>,
    pub cons: Arc<StmtData>,
    pub alt: Option<Arc<StmtData>>,
}

pub struct SwitchCaseData {
    pub span: Span,
    pub test: Option<Arc<ExprData>>,
    pub cons: Vec<Arc<StmtData>>,
}

pub struct SwitchStmtData {
    pub span: Span,
    pub body_ctxt: swc_core::common::SyntaxContext,
    pub discriminant: Arc<ExprData>,
    pub cases: Vec<Arc<SwitchCaseData>>,
}

pub struct ThrowStmtData {
    pub span: Span,
    pub arg: Arc<ExprData>,
}

pub struct CatchClauseData {
    pub span: Span,
    pub param: Option<Arc<PatData>>,
    pub body: Arc<BlockStmtData>,
}

pub struct TryStmtData {
    pub span: Span,
    pub block: Arc<BlockStmtData>,
    pub handler: Option<Arc<CatchClauseData>>,
    pub finalizer: Option<Arc<BlockStmtData>>,
}

pub struct WhileStmtData {
    pub span: Span,
    pub test: Arc<ExprData>,
    pub body: Arc<StmtData>,
}

pub struct DoWhileStmtData {
    pub span: Span,
    pub test: Arc<ExprData>,
    pub body: Arc<StmtData>,
}

pub struct ForStmtData {
    pub span: Span,
    pub init: Option<VarDeclOrExpr>,
    pub test: Option<Arc<ExprData>>,
    pub update: Option<Arc<ExprData>>,
    pub body: Arc<StmtData>,
}

pub struct ForInStmtData {
    pub span: Span,
    pub left: ForHead,
    pub right: Arc<ExprData>,
    pub body: Arc<StmtData>,
}

pub struct ForOfStmtData {
    pub span: Span,
    pub is_await: bool,
    pub left: ForHead,
    pub right: Arc<ExprData>,
    pub body: Arc<StmtData>,
}

pub struct ExprStmtData {
    pub span: Span,
    pub expr: Arc<ExprData>,
}


pub fn lower_block_stmt(b: BlockStmt) -> BlockStmtData {
    BlockStmtData {
        span: b.span,
        ctxt: b.ctxt,
        stmts: b.stmts.into_iter().map(|s| Arc::new(lower_stmt(s))).collect(),
    }
}

pub fn lower_catch_clause(c: CatchClause) -> CatchClauseData {
    CatchClauseData {
        span: c.span,
        param: c.param.map(|p| Arc::new(lower_pat(p))),
        body: Arc::new(lower_block_stmt(c.body)),
    }
}

fn lower_switch_case(c: SwitchCase) -> SwitchCaseData {
    SwitchCaseData {
        span: c.span,
        test: c.test.map(|t| Arc::new(lower_expr(*t))),
        cons: c.cons.into_iter().map(|s| Arc::new(lower_stmt(s))).collect(),
    }
}

pub fn lower_stmt(stmt: Stmt) -> StmtData {
    match stmt {
        Stmt::Block(s) => StmtData::Block(Arc::new(lower_block_stmt(s))),
        Stmt::Empty(s) => StmtData::Empty(s),
        Stmt::Debugger(s) => StmtData::Debugger(s),
        Stmt::With(s) => StmtData::With(WithStmtData {
            span: s.span,
            obj: Arc::new(lower_expr(*s.obj)),
            body: Arc::new(lower_stmt(*s.body)),
        }),
        Stmt::Return(s) => StmtData::Return(ReturnStmtData {
            span: s.span,
            arg: s.arg.map(|a| Arc::new(lower_expr(*a))),
        }),
        Stmt::Labeled(s) => StmtData::Labeled(LabeledStmtData {
            span: s.span,
            label: s.label,
            body: Arc::new(lower_stmt(*s.body)),
        }),
        Stmt::Break(s) => StmtData::Break(s),
        Stmt::Continue(s) => StmtData::Continue(s),
        Stmt::If(s) => StmtData::If(IfStmtData {
            span: s.span,
            test: Arc::new(lower_expr(*s.test)),
            cons: Arc::new(lower_stmt(*s.cons)),
            alt: s.alt.map(|a| Arc::new(lower_stmt(*a))),
        }),
        Stmt::Switch(s) => StmtData::Switch(SwitchStmtData {
            span: s.span,
            body_ctxt: s.body_ctxt,
            discriminant: Arc::new(lower_expr(*s.discriminant)),
            cases: s.cases.into_iter().map(|c| Arc::new(lower_switch_case(c))).collect(),
        }),
        Stmt::Throw(s) => StmtData::Throw(ThrowStmtData {
            span: s.span,
            arg: Arc::new(lower_expr(*s.arg)),
        }),
        Stmt::Try(s) => StmtData::Try(TryStmtData {
            span: s.span,
            block: Arc::new(lower_block_stmt(s.block)),
            handler: s.handler.map(|h| Arc::new(lower_catch_clause(h))),
            finalizer: s.finalizer.map(|f| Arc::new(lower_block_stmt(f))),
        }),
        Stmt::While(s) => StmtData::While(WhileStmtData {
            span: s.span,
            test: Arc::new(lower_expr(*s.test)),
            body: Arc::new(lower_stmt(*s.body)),
        }),
        Stmt::DoWhile(s) => StmtData::DoWhile(DoWhileStmtData {
            span: s.span,
            test: Arc::new(lower_expr(*s.test)),
            body: Arc::new(lower_stmt(*s.body)),
        }),
        Stmt::For(s) => StmtData::For(ForStmtData {
            span: s.span,
            init: s.init,
            test: s.test.map(|t| Arc::new(lower_expr(*t))),
            update: s.update.map(|u| Arc::new(lower_expr(*u))),
            body: Arc::new(lower_stmt(*s.body)),
        }),
        Stmt::ForIn(s) => StmtData::ForIn(ForInStmtData {
            span: s.span,
            left: s.left,
            right: Arc::new(lower_expr(*s.right)),
            body: Arc::new(lower_stmt(*s.body)),
        }),
        Stmt::ForOf(s) => StmtData::ForOf(ForOfStmtData {
            span: s.span,
            is_await: s.is_await,
            left: s.left,
            right: Arc::new(lower_expr(*s.right)),
            body: Arc::new(lower_stmt(*s.body)),
        }),
        Stmt::Decl(s) => StmtData::Decl(Arc::new(crate::pydecl::lower_decl(s))),
        Stmt::Expr(s) => StmtData::Expr(ExprStmtData {
            span: s.span,
            expr: Arc::new(lower_expr(*s.expr)),
        }),
    }
}

pub fn wrap_block_stmt_data(py: Python<'_>, data: Arc<BlockStmtData>) -> PyResult<Py<PyBlockStmt>> {
    Py::new(py, (PyBlockStmt::from_arc(Arc::new(StmtData::Block(data))), PyStmt {}))
}

pub fn wrap_stmt_data(py: Python<'_>, data: Arc<StmtData>) -> PyResult<Py<PyStmt>> {
    let base = PyStmt {};
    Ok(match &*data {
        StmtData::Block(_) => Py::new(py, (PyBlockStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::Empty(s) => Py::new(py, (PyEmptyStmt::build(py, s.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::Debugger(s) => Py::new(py, (PyDebuggerStmt::build(py, s.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::With(_) => Py::new(py, (PyWithStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::Return(_) => Py::new(py, (PyReturnStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::Labeled(_) => Py::new(py, (PyLabeledStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::Break(s) => Py::new(py, (PyBreakStmt::build(py, s.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::Continue(s) => Py::new(py, (PyContinueStmt::build(py, s.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::If(_) => Py::new(py, (PyIfStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::Switch(_) => Py::new(py, (PySwitchStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::Throw(_) => Py::new(py, (PyThrowStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::Try(_) => Py::new(py, (PyTryStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::While(_) => Py::new(py, (PyWhileStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::DoWhile(_) => Py::new(py, (PyDoWhileStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::For(_) => Py::new(py, (PyForStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::ForIn(_) => Py::new(py, (PyForInStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::ForOf(_) => Py::new(py, (PyForOfStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::Decl(s) => Py::new(py, (PyDeclStmt::from_arc(Arc::clone(s)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        StmtData::Expr(_) => Py::new(py, (PyExprStmt::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
    })
}

pub fn wrap_catch_clause_data(py: Python<'_>, data: Arc<CatchClauseData>) -> PyResult<Py<PyCatchClause>> {
    Py::new(py, PyCatchClause { inner: data })
}

pub fn conv_arc_stmt(py: Python<'_>, data: Arc<StmtData>) -> PyResult<Py<PyStmt>> {
    wrap_stmt_data(py, data)
}

pub fn conv_arc_stmts(py: Python<'_>, data: Vec<Arc<StmtData>>) -> PyResult<Vec<Py<PyStmt>>> {
    data.into_iter().map(|d| wrap_stmt_data(py, d)).collect()
}

pub fn conv_option_arc_stmt(
    py: Python<'_>,
    data: Option<Arc<StmtData>>,
) -> PyResult<Option<Py<PyStmt>>> {
    data.map(|d| wrap_stmt_data(py, d)).transpose()
}

pub fn conv_arc_catch_clause(
    py: Python<'_>,
    data: Option<Arc<CatchClauseData>>,
) -> PyResult<Option<Py<PyCatchClause>>> {
    data.map(|d| wrap_catch_clause_data(py, d)).transpose()
}

pub fn conv_arc_block_stmt(
    py: Python<'_>,
    data: Option<Arc<BlockStmtData>>,
) -> PyResult<Option<Py<PyBlockStmt>>> {
    data.map(|d| wrap_block_stmt_data(py, d)).transpose()
}

pub fn conv_arc_switch_cases(
    py: Python<'_>,
    data: Vec<Arc<SwitchCaseData>>,
) -> PyResult<Vec<Py<PySwitchCase>>> {
    data.into_iter()
        .map(|d| Py::new(py, PySwitchCase { inner: d }))
        .collect()
}

pub fn stmt_to_py(py: Python<'_>, stmt: Stmt) -> PyResult<Py<PyStmt>> {
    wrap_stmt_data(py, Arc::new(lower_stmt(stmt)))
}


#[pyclass]
pub struct PyCatchClause {
    inner: Arc<CatchClauseData>,
}

#[pymethods]
impl PyCatchClause {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.inner.span)
    }

    #[getter]
    fn param(&self, py: Python<'_>) -> PyResult<Option<Py<PyPat>>> {
        match &self.inner.param {
            Some(p) => conv_arc_pat(py, Arc::clone(p)).map(Some),
            None => Ok(None),
        }
    }

    #[getter]
    fn body(&self, py: Python<'_>) -> PyResult<Py<PyBlockStmt>> {
        wrap_block_stmt_data(py, Arc::clone(&self.inner.body))
    }
}

#[pyclass(extends=PyStmt)]
pub struct PyBlockStmt {
    inner: Arc<StmtData>,
}

impl PyBlockStmt {
    pub fn from_arc(inner: Arc<StmtData>) -> Self {
        PyBlockStmt { inner }
    }

    fn data(&self) -> &BlockStmtData {
        match &*self.inner {
            StmtData::Block(d) => d,
            _ => unreachable!("StmtData/PyBlockStmt mismatch"),
        }
    }
}

#[pymethods]
impl PyBlockStmt {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.data().span)
    }

    #[getter]
    fn ctxt(&self, py: Python<'_>) -> PyResult<u32> {
        conv_ctxt(py, self.data().ctxt)
    }

    #[getter]
    fn stmts(&self, py: Python<'_>) -> PyResult<Vec<Py<PyStmt>>> {
        conv_arc_stmts(py, self.data().stmts.clone())
    }
}

ast_node_variant!(PyStmt, PyEmptyStmt, EmptyStmt, {
    span: PySpan = conv_span
});

arc_variant_node!(PyStmt, PyTryStmt, StmtData, StmtData::Try, TryStmtData, {
    span: PySpan = conv_span,
    block: Py<PyBlockStmt> = conv_arc_block_stmt_req,
    handler: Option<Py<PyCatchClause>> = conv_arc_catch_clause,
    finalizer: Option<Py<PyBlockStmt>> = conv_arc_block_stmt,
});

fn conv_arc_block_stmt_req(py: Python<'_>, data: Arc<BlockStmtData>) -> PyResult<Py<PyBlockStmt>> {
    wrap_block_stmt_data(py, data)
}

arc_variant_node!(PyStmt, PyExprStmt, StmtData, StmtData::Expr, ExprStmtData, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_arc_expr,
});

ast_node_variant!(PyStmt, PyDebuggerStmt, DebuggerStmt, {
    span: PySpan = conv_span
});

arc_variant_node!(PyStmt, PyWithStmt, StmtData, StmtData::With, WithStmtData, {
    span: PySpan = conv_span,
    obj: Py<PyExpr> = conv_arc_expr,
    body: Py<PyStmt> = conv_arc_stmt,
});

arc_variant_node!(PyStmt, PyReturnStmt, StmtData, StmtData::Return, ReturnStmtData, {
    span: PySpan = conv_span,
    arg: Option<Py<PyExpr>> = conv_option_arc_expr,
});

arc_variant_node!(PyStmt, PyLabeledStmt, StmtData, StmtData::Labeled, LabeledStmtData, {
    span: PySpan = conv_span,
    label: PyIdent = conv_ident,
    body: Py<PyStmt> = conv_arc_stmt,
});

ast_node_variant!(PyStmt, PyBreakStmt, BreakStmt, {
    span: PySpan = conv_span,
    label: Option<PyIdent> = conv_option_ident
});

ast_node_variant!(PyStmt, PyContinueStmt, ContinueStmt, {
    span: PySpan = conv_span,
    label: Option<PyIdent> = conv_option_ident
});

arc_variant_node!(PyStmt, PyIfStmt, StmtData, StmtData::If, IfStmtData, {
    span: PySpan = conv_span,
    test: Py<PyExpr> = conv_arc_expr,
    cons: Py<PyStmt> = conv_arc_stmt,
    alt: Option<Py<PyStmt>> = conv_option_arc_stmt,
});

#[pyclass]
pub struct PySwitchCase {
    inner: Arc<SwitchCaseData>,
}

#[pymethods]
impl PySwitchCase {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.inner.span)
    }

    #[getter]
    fn test(&self, py: Python<'_>) -> PyResult<Option<Py<PyExpr>>> {
        conv_option_arc_expr(py, self.inner.test.clone())
    }

    #[getter]
    fn cons(&self, py: Python<'_>) -> PyResult<Vec<Py<PyStmt>>> {
        conv_arc_stmts(py, self.inner.cons.clone())
    }
}

arc_variant_node!(PyStmt, PySwitchStmt, StmtData, StmtData::Switch, SwitchStmtData, {
    span: PySpan = conv_span,
    body_ctxt: u32 = conv_ctxt,
    discriminant: Py<PyExpr> = conv_arc_expr,
    cases: Vec<Py<PySwitchCase>> = conv_arc_switch_cases,
});

arc_variant_node!(PyStmt, PyThrowStmt, StmtData, StmtData::Throw, ThrowStmtData, {
    span: PySpan = conv_span,
    arg: Py<PyExpr> = conv_arc_expr,
});

arc_variant_node!(PyStmt, PyWhileStmt, StmtData, StmtData::While, WhileStmtData, {
    span: PySpan = conv_span,
    test: Py<PyExpr> = conv_arc_expr,
    body: Py<PyStmt> = conv_arc_stmt,
});

arc_variant_node!(PyStmt, PyDoWhileStmt, StmtData, StmtData::DoWhile, DoWhileStmtData, {
    span: PySpan = conv_span,
    test: Py<PyExpr> = conv_arc_expr,
    body: Py<PyStmt> = conv_arc_stmt,
});

#[pyclass]
pub struct PyVarDeclOrExpr {
    #[pyo3(get)]
    pub var_decl: Option<Py<crate::pydecl::PyVarDecl>>,
    #[pyo3(get)]
    pub expr: Option<Py<PyExpr>>,
}

arc_variant_node!(PyStmt, PyForStmt, StmtData, StmtData::For, ForStmtData, {
    span: PySpan = conv_span,
    init: Option<Py<PyVarDeclOrExpr>> = conv_option_var_decl_or_expr,
    test: Option<Py<PyExpr>> = conv_option_arc_expr,
    update: Option<Py<PyExpr>> = conv_option_arc_expr,
    body: Py<PyStmt> = conv_arc_stmt,
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

arc_variant_node!(PyStmt, PyForInStmt, StmtData, StmtData::ForIn, ForInStmtData, {
    span: PySpan = conv_span,
    left: Py<PyForHead> = conv_for_head,
    right: Py<PyExpr> = conv_arc_expr,
    body: Py<PyStmt> = conv_arc_stmt,
});

arc_variant_node!(PyStmt, PyForOfStmt, StmtData, StmtData::ForOf, ForOfStmtData, {
    span: PySpan = conv_span,
    is_await: bool = conv_bool,
    left: Py<PyForHead> = conv_for_head,
    right: Py<PyExpr> = conv_arc_expr,
    body: Py<PyStmt> = conv_arc_stmt,
});

#[pyclass(extends=PyStmt)]
pub struct PyDeclStmt {
    inner: Arc<crate::pydecl::DeclData>,
}

impl PyDeclStmt {
    pub fn from_arc(inner: Arc<crate::pydecl::DeclData>) -> Self {
        PyDeclStmt { inner }
    }
}

#[pymethods]
impl PyDeclStmt {
    #[getter]
    fn decl(&self, py: Python<'_>) -> PyResult<Py<crate::pydecl::PyDecl>> {
        crate::pydecl::wrap_decl_data(py, Arc::clone(&self.inner))
    }
}
