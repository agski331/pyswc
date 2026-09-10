use pyo3::prelude::*;
use swc_core::ecma::ast::{TsEnumDecl, TsInterfaceDecl, TsTypeAliasDecl, UsingDecl, VarDecl};

use crate::{
    conversions::{
        conv_bool, conv_boxed_class, conv_boxed_function, conv_boxed_tstype, conv_ctxt,
        conv_ident, conv_option_type_param_decl, conv_span, conv_ts_enum_members,
        conv_ts_expr_with_type_args_vec, conv_ts_interface_body, conv_var_decl_kind,
        conv_var_declarators,
    },
    macros::ast_node_variant,
    pyclass::PyClass,
    pyfunction::PyFunction,
    pyident::PyIdent,
    pypat::PyPat,
    pyspan::PySpan,
    pytypeinfo::{PyTsExprWithTypeArgs, PyTsType, PyTsTypeParamDecl},
};

#[pyclass(subclass)]
pub struct PyDecl{

}

#[pyclass(extends=PyDecl)]
pub struct PyClassDecl{
    #[pyo3(get)]
    pub ident: PyIdent,
    #[pyo3(get)]
    pub declare: bool,
    #[pyo3(get)]
    pub class: Py<PyClass>
}

impl PyClassDecl {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::ClassDecl) -> PyResult<Self> {
        Ok(PyClassDecl {
            ident: conv_ident(py, node.ident)?,
            declare: node.declare,
            class: conv_boxed_class(py, node.class)?
        })
    }
}

#[pyclass(extends=PyDecl)]
pub struct PyFnDecl{
    #[pyo3(get)]
    pub ident: PyIdent,
    #[pyo3(get)]
    pub declare: bool,
    #[pyo3(get)]
    pub function: Py<PyFunction>
}

impl PyFnDecl {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::FnDecl) -> PyResult<Self> {
        Ok(PyFnDecl {
            ident: conv_ident(py, node.ident)?,
            declare: node.declare,
            function: conv_boxed_function(py, node.function)?
        })
    }
}

#[pyclass]
pub struct PyVarDeclarator{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub name: Py<PyPat>,
    #[pyo3(get)]
    pub init: Option<Py<crate::pyexpr::PyExpr>>,
    #[pyo3(get)]
    pub definite: bool
}

ast_node_variant!(PyDecl, PyVarDecl, VarDecl, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    kind: u32 = conv_var_decl_kind,
    declare: bool = conv_bool,
    decls: Vec<Py<PyVarDeclarator>> = conv_var_declarators
});

ast_node_variant!(PyDecl, PyUsingDecl, UsingDecl, {
    span: PySpan = conv_span,
    is_await: bool = conv_bool,
    decls: Vec<Py<PyVarDeclarator>> = conv_var_declarators
});

#[pyclass]
pub struct PyTsInterfaceBody{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub body: Vec<Py<crate::pytypeinfo::PyTsTypeElement>>
}

ast_node_variant!(PyDecl, PyTsInterfaceDecl, TsInterfaceDecl, {
    span: PySpan = conv_span,
    id: PyIdent = conv_ident,
    declare: bool = conv_bool,
    type_params: Option<Py<PyTsTypeParamDecl>> = conv_option_type_param_decl,
    extends: Vec<Py<PyTsExprWithTypeArgs>> = conv_ts_expr_with_type_args_vec,
    body: Py<PyTsInterfaceBody> = conv_ts_interface_body
});

ast_node_variant!(PyDecl, PyTsTypeAliasDecl, TsTypeAliasDecl, {
    span: PySpan = conv_span,
    declare: bool = conv_bool,
    id: PyIdent = conv_ident,
    type_params: Option<Py<PyTsTypeParamDecl>> = conv_option_type_param_decl,
    type_ann: Py<PyTsType> = conv_boxed_tstype
});

#[pyclass]
pub struct PyTsEnumMember{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub id_ident: Option<PyIdent>,
    #[pyo3(get)]
    pub id_str_span: Option<PySpan>,
    #[pyo3(get)]
    pub id_str_value: Option<String>,
    #[pyo3(get)]
    pub init: Option<Py<crate::pyexpr::PyExpr>>
}

ast_node_variant!(PyDecl, PyTsEnumDecl, TsEnumDecl, {
    span: PySpan = conv_span,
    declare: bool = conv_bool,
    is_const: bool = conv_bool,
    id: PyIdent = conv_ident,
    members: Vec<Py<PyTsEnumMember>> = conv_ts_enum_members
});

#[derive(Clone)]
#[pyclass]
pub struct PyTsModuleName{
    #[pyo3(get)]
    pub ident: Option<PyIdent>,
    #[pyo3(get)]
    pub str_span: Option<PySpan>,
    #[pyo3(get)]
    pub str_value: Option<String>
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
pub struct PyTsNamespaceBody{

}

ast_node_variant!(PyTsNamespaceBody, PyTsModuleBlock, swc_core::ecma::ast::TsModuleBlock, {
    span: PySpan = conv_span,
    body: Vec<Py<crate::pymodule::PyModuleItem>> = crate::conversions::conv_module_items
});

ast_node_variant!(PyTsNamespaceBody, PyTsNamespaceDecl, swc_core::ecma::ast::TsNamespaceDecl, {
    span: PySpan = conv_span,
    declare: bool = conv_bool,
    global: bool = conv_bool,
    id: PyIdent = conv_ident,
    body: Py<PyTsNamespaceBody> = crate::conversions::conv_boxed_ts_namespace_body
});

pub fn conv_decl(py: Python<'_>, decl: swc_core::ecma::ast::Decl) -> PyResult<Py<PyDecl>>{
    let base = PyDecl { };
    Ok(match decl {
        swc_core::ecma::ast::Decl::Class(c) => Py::new(py, (PyClassDecl::build(py, c)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::Decl::Fn(f) => Py::new(py, (PyFnDecl::build(py, f)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::Decl::Var(v) => Py::new(py, (PyVarDecl::build(py, *v)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::Decl::Using(u) => Py::new(py, (PyUsingDecl::build(py, *u)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::Decl::TsInterface(i) => Py::new(py, (PyTsInterfaceDecl::build(py, *i)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::Decl::TsTypeAlias(a) => Py::new(py, (PyTsTypeAliasDecl::build(py, *a)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::Decl::TsEnum(e) => Py::new(py, (PyTsEnumDecl::build(py, *e)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::Decl::TsModule(m) => Py::new(py, (PyTsModuleDecl::build(py, *m)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn decl_to_py(py: Python<'_>, decl: swc_core::ecma::ast::Decl) -> PyResult<Py<PyDecl>> {
    conv_decl(py, decl)
}
