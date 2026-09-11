use pyo3::prelude::*;
use std::sync::Arc;
use swc_core::common::Span;
use swc_core::ecma::ast::{Ident, TsEnumMemberId, TsExprWithTypeArgs, TsTypeElement};

use crate::{
    conversions::{
        conv_bool, conv_ctxt, conv_ident, conv_span, conv_str,
        conv_ts_expr_with_type_args_vec, conv_ts_type_elements,
    },
    macros::{arc_variant_node, ast_node_variant},
    pyclass::{ClassData, PyClass, lower_class, wrap_class_data},
    pyenums::PyVarDeclKind,
    pyexpr::{ExprData, PyExpr, conv_option_arc_expr, lower_expr},
    pyfunction::{FunctionData, PyFunction, lower_function, wrap_function_data},
    pyident::PyIdent,
    pypat::{PatData, PyPat, conv_arc_pat, lower_pat},
    pyspan::PySpan,
    pytypeinfo::{
        PyTsExprWithTypeArgs, PyTsType, PyTsTypeParamDecl, TsTypeData, TsTypeParamDeclData,
        conv_arc_tstype, conv_option_arc_ts_type_param_decl, lower_ts_type_param_decl, lower_tstype,
    },
};

#[pyclass(subclass)]
pub struct PyDecl {}

pub struct ClassDeclData {
    pub ident: Ident,
    pub declare: bool,
    pub class: Arc<ClassData>,
}

pub struct FnDeclData {
    pub ident: Ident,
    pub declare: bool,
    pub function: Arc<FunctionData>,
}

#[derive(Clone)]
pub struct VarDeclaratorData {
    pub span: Span,
    pub name: Arc<PatData>,
    pub init: Option<Arc<ExprData>>,
    pub definite: bool,
}

pub struct VarDeclData {
    pub span: Span,
    pub ctxt: swc_core::common::SyntaxContext,
    pub kind: swc_core::ecma::ast::VarDeclKind,
    pub declare: bool,
    pub decls: Vec<VarDeclaratorData>,
}

pub struct UsingDeclData {
    pub span: Span,
    pub is_await: bool,
    pub decls: Vec<VarDeclaratorData>,
}

pub struct TsInterfaceBodyData {
    pub span: Span,
    pub body: Vec<TsTypeElement>,
}

pub struct TsInterfaceDeclData {
    pub span: Span,
    pub id: Ident,
    pub declare: bool,
    pub type_params: Option<Arc<TsTypeParamDeclData>>,
    pub extends: Vec<TsExprWithTypeArgs>,
    pub body: Arc<TsInterfaceBodyData>,
}

pub struct TsTypeAliasDeclData {
    pub span: Span,
    pub declare: bool,
    pub id: Ident,
    pub type_params: Option<Arc<TsTypeParamDeclData>>,
    pub type_ann: Arc<TsTypeData>,
}

#[derive(Clone)]
pub struct TsEnumMemberData {
    pub span: Span,
    pub id: TsEnumMemberId,
    pub init: Option<Arc<ExprData>>,
}

pub struct TsEnumDeclData {
    pub span: Span,
    pub declare: bool,
    pub is_const: bool,
    pub id: Ident,
    pub members: Vec<TsEnumMemberData>,
}

pub fn lower_var_declarator(v: swc_core::ecma::ast::VarDeclarator) -> VarDeclaratorData {
    VarDeclaratorData {
        span: v.span,
        name: Arc::new(lower_pat(v.name)),
        init: v.init.map(|e| Arc::new(lower_expr(*e))),
        definite: v.definite,
    }
}

pub fn lower_var_decl(v: swc_core::ecma::ast::VarDecl) -> VarDeclData {
    VarDeclData {
        span: v.span,
        ctxt: v.ctxt,
        kind: v.kind,
        declare: v.declare,
        decls: v.decls.into_iter().map(lower_var_declarator).collect(),
    }
}

pub fn lower_using_decl(u: swc_core::ecma::ast::UsingDecl) -> UsingDeclData {
    UsingDeclData {
        span: u.span,
        is_await: u.is_await,
        decls: u.decls.into_iter().map(lower_var_declarator).collect(),
    }
}

pub fn lower_ts_interface_decl(i: swc_core::ecma::ast::TsInterfaceDecl) -> TsInterfaceDeclData {
    TsInterfaceDeclData {
        span: i.span,
        id: i.id,
        declare: i.declare,
        type_params: i.type_params.map(|tp| Arc::new(lower_ts_type_param_decl(*tp))),
        extends: i.extends,
        body: Arc::new(TsInterfaceBodyData {
            span: i.body.span,
            body: i.body.body,
        }),
    }
}

pub fn lower_ts_enum_member(m: swc_core::ecma::ast::TsEnumMember) -> TsEnumMemberData {
    TsEnumMemberData {
        span: m.span,
        id: m.id,
        init: m.init.map(|e| Arc::new(lower_expr(*e))),
    }
}

pub fn wrap_var_declarator_data(py: Python<'_>, data: VarDeclaratorData) -> PyResult<Py<PyVarDeclarator>> {
    Py::new(py, PyVarDeclarator { data })
}

pub fn conv_arc_var_declarators(
    py: Python<'_>,
    data: Vec<VarDeclaratorData>,
) -> PyResult<Vec<Py<PyVarDeclarator>>> {
    data.into_iter().map(|d| wrap_var_declarator_data(py, d)).collect()
}

pub fn wrap_ts_enum_member_data(py: Python<'_>, data: TsEnumMemberData) -> PyResult<Py<PyTsEnumMember>> {
    Py::new(py, PyTsEnumMember { data })
}

pub fn conv_arc_ts_enum_members(
    py: Python<'_>,
    data: Vec<TsEnumMemberData>,
) -> PyResult<Vec<Py<PyTsEnumMember>>> {
    data.into_iter().map(|d| wrap_ts_enum_member_data(py, d)).collect()
}

#[pyclass(extends=PyDecl)]
pub struct PyClassDecl {
    inner: Arc<ClassDeclData>,
}

impl PyClassDecl {
    pub fn from_arc(inner: Arc<ClassDeclData>) -> Self {
        PyClassDecl { inner }
    }
}

#[pymethods]
impl PyClassDecl {
    #[getter]
    fn ident(&self, py: Python<'_>) -> PyResult<PyIdent> {
        conv_ident(py, self.inner.ident.clone())
    }

    #[getter]
    fn declare(&self) -> bool {
        self.inner.declare
    }

    #[getter]
    fn class(&self, py: Python<'_>) -> PyResult<Py<PyClass>> {
        wrap_class_data(py, Arc::clone(&self.inner.class))
    }
}

#[pyclass(extends=PyDecl)]
pub struct PyFnDecl {
    inner: Arc<FnDeclData>,
}

impl PyFnDecl {
    pub fn from_arc(inner: Arc<FnDeclData>) -> Self {
        PyFnDecl { inner }
    }
}

#[pymethods]
impl PyFnDecl {
    #[getter]
    fn ident(&self, py: Python<'_>) -> PyResult<PyIdent> {
        conv_ident(py, self.inner.ident.clone())
    }

    #[getter]
    fn declare(&self) -> bool {
        self.inner.declare
    }

    #[getter]
    fn function(&self, py: Python<'_>) -> PyResult<Py<PyFunction>> {
        wrap_function_data(py, Arc::clone(&self.inner.function))
    }
}

#[pyclass]
pub struct PyVarDeclarator {
    data: VarDeclaratorData,
}

#[pymethods]
impl PyVarDeclarator {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.data.span)
    }

    #[getter]
    fn name(&self, py: Python<'_>) -> PyResult<Py<PyPat>> {
        conv_arc_pat(py, Arc::clone(&self.data.name))
    }

    #[getter]
    fn init(&self, py: Python<'_>) -> PyResult<Option<Py<PyExpr>>> {
        conv_option_arc_expr(py, self.data.init.clone())
    }

    #[getter]
    fn definite(&self) -> bool {
        self.data.definite
    }
}

arc_variant_node!(PyDecl, PyVarDecl, DeclData, DeclData::Var, VarDeclData, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    kind: PyVarDeclKind = crate::conversions::conv_var_decl_kind,
    declare: bool = conv_bool,
    decls: Vec<Py<PyVarDeclarator>> = conv_arc_var_declarators,
});

arc_variant_node!(PyDecl, PyUsingDecl, DeclData, DeclData::Using, UsingDeclData, {
    span: PySpan = conv_span,
    is_await: bool = conv_bool,
    decls: Vec<Py<PyVarDeclarator>> = conv_arc_var_declarators,
});

#[pyclass]
pub struct PyTsInterfaceBody {
    inner: Arc<TsInterfaceBodyData>,
}

#[pymethods]
impl PyTsInterfaceBody {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.inner.span)
    }

    #[getter]
    fn body(&self, py: Python<'_>) -> PyResult<Vec<Py<crate::pytypeinfo::PyTsTypeElement>>> {
        conv_ts_type_elements(py, self.inner.body.clone())
    }
}

pub fn wrap_ts_interface_body_data(
    py: Python<'_>,
    data: Arc<TsInterfaceBodyData>,
) -> PyResult<Py<PyTsInterfaceBody>> {
    Py::new(py, PyTsInterfaceBody { inner: data })
}

#[pyclass(extends=PyDecl)]
pub struct PyTsInterfaceDecl {
    inner: Arc<DeclData>,
}

impl PyTsInterfaceDecl {
    pub fn from_arc(inner: Arc<DeclData>) -> Self {
        PyTsInterfaceDecl { inner }
    }

    fn data(&self) -> &TsInterfaceDeclData {
        match &*self.inner {
            DeclData::TsInterface(d) => d,
            _ => unreachable!("DeclData/PyTsInterfaceDecl mismatch"),
        }
    }
}

#[pymethods]
impl PyTsInterfaceDecl {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.data().span)
    }

    #[getter]
    fn id(&self, py: Python<'_>) -> PyResult<PyIdent> {
        conv_ident(py, self.data().id.clone())
    }

    #[getter]
    fn declare(&self) -> bool {
        self.data().declare
    }

    #[getter]
    fn type_params(&self, py: Python<'_>) -> PyResult<Option<Py<PyTsTypeParamDecl>>> {
        conv_option_arc_ts_type_param_decl(py, self.data().type_params.clone())
    }

    #[getter]
    fn extends(&self, py: Python<'_>) -> PyResult<Vec<Py<PyTsExprWithTypeArgs>>> {
        conv_ts_expr_with_type_args_vec(py, self.data().extends.clone())
    }

    #[getter]
    fn body(&self, py: Python<'_>) -> PyResult<Py<PyTsInterfaceBody>> {
        wrap_ts_interface_body_data(py, Arc::clone(&self.data().body))
    }
}

arc_variant_node!(PyDecl, PyTsTypeAliasDecl, DeclData, DeclData::TsTypeAlias, TsTypeAliasDeclData, {
    span: PySpan = conv_span,
    declare: bool = conv_bool,
    id: PyIdent = conv_ident,
    type_params: Option<Py<PyTsTypeParamDecl>> = conv_option_arc_ts_type_param_decl,
    type_ann: Py<PyTsType> = conv_arc_tstype,
});

#[pyclass]
pub struct PyTsEnumMember {
    data: TsEnumMemberData,
}

#[pymethods]
impl PyTsEnumMember {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.data.span)
    }

    #[getter]
    fn id_ident(&self, py: Python<'_>) -> PyResult<Option<PyIdent>> {
        match &self.data.id {
            TsEnumMemberId::Ident(i) => conv_ident(py, i.clone()).map(Some),
            TsEnumMemberId::Str(_) => Ok(None),
        }
    }

    #[getter]
    fn id_str_span(&self, py: Python<'_>) -> PyResult<Option<PySpan>> {
        match &self.data.id {
            TsEnumMemberId::Str(s) => conv_span(py, s.span).map(Some),
            TsEnumMemberId::Ident(_) => Ok(None),
        }
    }

    #[getter]
    fn id_str_value(&self, py: Python<'_>) -> PyResult<Option<String>> {
        match &self.data.id {
            TsEnumMemberId::Str(s) => conv_str(py, s.clone()).map(Some),
            TsEnumMemberId::Ident(_) => Ok(None),
        }
    }

    #[getter]
    fn init(&self, py: Python<'_>) -> PyResult<Option<Py<PyExpr>>> {
        conv_option_arc_expr(py, self.data.init.clone())
    }
}

arc_variant_node!(PyDecl, PyTsEnumDecl, DeclData, DeclData::TsEnum, TsEnumDeclData, {
    span: PySpan = conv_span,
    declare: bool = conv_bool,
    is_const: bool = conv_bool,
    id: PyIdent = conv_ident,
    members: Vec<Py<PyTsEnumMember>> = conv_arc_ts_enum_members,
});

#[derive(Clone)]
#[pyclass]
pub struct PyTsModuleName {
    #[pyo3(get)]
    pub ident: Option<PyIdent>,
    #[pyo3(get)]
    pub str_span: Option<PySpan>,
    #[pyo3(get)]
    pub str_value: Option<String>,
}

ast_node_variant!(PyDecl, PyTsModuleDecl, swc_core::ecma::ast::TsModuleDecl, {
    span: PySpan = conv_span,
    declare: bool = conv_bool,
    global: bool = conv_bool,
    namespace: bool = conv_bool,
    id: PyTsModuleName = crate::conversions::conv_ts_module_name,
    body: Option<Py<PyTsNamespaceBody>> = crate::conversions::conv_option_ts_namespace_body
});

#[pyclass(subclass)]
pub struct PyTsNamespaceBody {}

ast_node_variant!(PyTsNamespaceBody, PyTsModuleBlock, swc_core::ecma::ast::TsModuleBlock, {
    span: PySpan = conv_span,
    body: Vec<Py<crate::pymodule::PyModuleItem>> = crate::conversions::conv_module_items
});

ast_node_variant!(PyTsNamespaceBody, PyTsNamespaceDecl, swc_core::ecma::ast::TsNamespaceDecl, {
    span: PySpan = conv_span,
    declare: bool = conv_bool,
    id: PyIdent = conv_ident,
    body: Py<PyTsNamespaceBody> = crate::conversions::conv_boxed_ts_namespace_body
});

pub enum DeclData {
    Class(Arc<ClassDeclData>),
    Fn(Arc<FnDeclData>),
    Var(VarDeclData),
    Using(UsingDeclData),
    TsInterface(TsInterfaceDeclData),
    TsTypeAlias(TsTypeAliasDeclData),
    TsEnum(TsEnumDeclData),
    TsModule(swc_core::ecma::ast::TsModuleDecl),
}

pub fn lower_decl(decl: swc_core::ecma::ast::Decl) -> DeclData {
    use swc_core::ecma::ast::Decl;
    match decl {
        Decl::Class(c) => DeclData::Class(Arc::new(ClassDeclData {
            ident: c.ident,
            declare: c.declare,
            class: Arc::new(lower_class(*c.class)),
        })),
        Decl::Fn(f) => DeclData::Fn(Arc::new(FnDeclData {
            ident: f.ident,
            declare: f.declare,
            function: Arc::new(lower_function(*f.function)),
        })),
        Decl::Var(v) => DeclData::Var(lower_var_decl(*v)),
        Decl::Using(u) => DeclData::Using(lower_using_decl(*u)),
        Decl::TsInterface(i) => DeclData::TsInterface(lower_ts_interface_decl(*i)),
        Decl::TsTypeAlias(a) => DeclData::TsTypeAlias(TsTypeAliasDeclData {
            span: a.span,
            declare: a.declare,
            id: a.id,
            type_params: a.type_params.map(|tp| Arc::new(lower_ts_type_param_decl(*tp))),
            type_ann: Arc::new(lower_tstype(*a.type_ann)),
        }),
        Decl::TsEnum(e) => DeclData::TsEnum(TsEnumDeclData {
            span: e.span,
            declare: e.declare,
            is_const: e.is_const,
            id: e.id,
            members: e.members.into_iter().map(lower_ts_enum_member).collect(),
        }),
        Decl::TsModule(m) => DeclData::TsModule(*m),
    }
}

pub fn wrap_decl_data(py: Python<'_>, data: Arc<DeclData>) -> PyResult<Py<PyDecl>> {
    let base = PyDecl {};
    Ok(match &*data {
        DeclData::Class(c) => Py::new(py, (PyClassDecl::from_arc(Arc::clone(c)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        DeclData::Fn(f) => Py::new(py, (PyFnDecl::from_arc(Arc::clone(f)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        DeclData::Var(_) => Py::new(py, (PyVarDecl::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        DeclData::Using(_) => Py::new(py, (PyUsingDecl::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        DeclData::TsInterface(_) => Py::new(py, (PyTsInterfaceDecl::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        DeclData::TsTypeAlias(_) => Py::new(py, (PyTsTypeAliasDecl::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        DeclData::TsEnum(_) => Py::new(py, (PyTsEnumDecl::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        DeclData::TsModule(m) => Py::new(py, (PyTsModuleDecl::build(py, m.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
    })
}

pub fn conv_decl(py: Python<'_>, decl: swc_core::ecma::ast::Decl) -> PyResult<Py<PyDecl>> {
    wrap_decl_data(py, Arc::new(lower_decl(decl)))
}
