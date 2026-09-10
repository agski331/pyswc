use pyo3::prelude::*;
use swc_core::ecma::ast::{
    ExportAll, ExportDecl, ExportDefaultDecl, ExportDefaultExpr, ImportDecl, NamedExport,
    TsExportAssignment, TsNamespaceExportDecl,
};

use crate::{
    conversions::{
        conv_bool, conv_boxed_class, conv_boxed_expr, conv_boxed_function, conv_boxed_str,
        conv_ident, conv_import_phase, conv_option_boxed_object_lit, conv_span,
    },
    macros::ast_node_variant,
    pydecl::{conv_decl, PyDecl},
    pyexpr::PyExpr,
    pyfunction::PyFunction,
    pyident::PyIdent,
    pyspan::PySpan,
    pytypeinfo::PyTsEntityName,
};

#[derive(Clone)]
#[pyclass]
pub struct PyModuleExportName{
    #[pyo3(get)]
    pub ident: Option<PyIdent>,
    #[pyo3(get)]
    pub str_span: Option<PySpan>,
    #[pyo3(get)]
    pub str_value: Option<String>
}

#[pyclass(subclass)]
pub struct PyModuleItem{

}

#[pyclass(extends=PyModuleItem)]
pub struct PyModuleItemStmt{
    #[pyo3(get)]
    pub stmt: Py<crate::pystmts::PyStmt>
}

impl PyModuleItemStmt {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::Stmt) -> PyResult<Self> {
        Ok(PyModuleItemStmt { stmt: crate::pystmts::stmt_to_py(py, node)? })
    }
}

#[pyclass(subclass)]
pub struct PyModuleDecl{

}

#[pyclass(extends=PyModuleItem)]
pub struct PyModuleItemDecl{
    #[pyo3(get)]
    pub decl: Py<PyModuleDecl>
}

impl PyModuleItemDecl {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::ModuleDecl) -> PyResult<Self> {
        Ok(PyModuleItemDecl { decl: conv_module_decl(py, node)? })
    }
}

pub fn conv_module_item(py: Python<'_>, item: swc_core::ecma::ast::ModuleItem) -> PyResult<Py<PyModuleItem>>{
    let base = PyModuleItem { };
    Ok(match item {
        swc_core::ecma::ast::ModuleItem::Stmt(s) => Py::new(py, (PyModuleItemStmt::build(py, s)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ModuleItem::ModuleDecl(d) => Py::new(py, (PyModuleItemDecl::build(py, d)?, base))?.into_bound(py).into_super().unbind(),
    })
}


#[pyclass(subclass)]
pub struct PyImportSpecifier{

}

ast_node_variant!(PyImportSpecifier, PyImportNamedSpecifier, swc_core::ecma::ast::ImportNamedSpecifier, {
    span: PySpan = conv_span,
    local: PyIdent = conv_ident,
    imported: Option<PyModuleExportName> = crate::conversions::conv_option_module_export_name,
    is_type_only: bool = conv_bool
});

ast_node_variant!(PyImportSpecifier, PyImportDefaultSpecifier, swc_core::ecma::ast::ImportDefaultSpecifier, {
    span: PySpan = conv_span,
    local: PyIdent = conv_ident
});

ast_node_variant!(PyImportSpecifier, PyImportStarAsSpecifier, swc_core::ecma::ast::ImportStarAsSpecifier, {
    span: PySpan = conv_span,
    local: PyIdent = conv_ident
});

pub fn conv_import_specifier(py: Python<'_>, spec: swc_core::ecma::ast::ImportSpecifier) -> PyResult<Py<PyImportSpecifier>>{
    let base = PyImportSpecifier { };
    Ok(match spec {
        swc_core::ecma::ast::ImportSpecifier::Named(n) => Py::new(py, (PyImportNamedSpecifier::build(py, n)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ImportSpecifier::Default(d) => Py::new(py, (PyImportDefaultSpecifier::build(py, d)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ImportSpecifier::Namespace(n) => Py::new(py, (PyImportStarAsSpecifier::build(py, n)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_import_specifiers(py: Python<'_>, specs: Vec<swc_core::ecma::ast::ImportSpecifier>) -> PyResult<Vec<Py<PyImportSpecifier>>>{
    specs.into_iter().map(|s| conv_import_specifier(py, s)).collect()
}

ast_node_variant!(PyModuleDecl, PyImportDecl, ImportDecl, {
    span: PySpan = conv_span,
    specifiers: Vec<Py<PyImportSpecifier>> = conv_import_specifiers,
    src: String = conv_boxed_str,
    type_only: bool = conv_bool,
    with: Option<Py<PyExpr>> = conv_option_boxed_object_lit,
    phase: u32 = conv_import_phase
});


ast_node_variant!(PyModuleDecl, PyExportDecl, ExportDecl, {
    span: PySpan = conv_span,
    decl: Py<PyDecl> = conv_decl
});

ast_node_variant!(PyModuleDecl, PyExportDefaultExpr, ExportDefaultExpr, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyModuleDecl, PyExportAll, ExportAll, {
    span: PySpan = conv_span,
    src: String = conv_boxed_str,
    type_only: bool = conv_bool,
    with: Option<Py<PyExpr>> = conv_option_boxed_object_lit
});

#[pyclass(subclass)]
pub struct PyExportSpecifier{

}

ast_node_variant!(PyExportSpecifier, PyExportNamespaceSpecifier, swc_core::ecma::ast::ExportNamespaceSpecifier, {
    span: PySpan = conv_span,
    name: PyModuleExportName = crate::conversions::conv_module_export_name
});

ast_node_variant!(PyExportSpecifier, PyExportDefaultSpecifier, swc_core::ecma::ast::ExportDefaultSpecifier, {
    exported: PyIdent = conv_ident
});

ast_node_variant!(PyExportSpecifier, PyExportNamedSpecifier, swc_core::ecma::ast::ExportNamedSpecifier, {
    span: PySpan = conv_span,
    orig: PyModuleExportName = crate::conversions::conv_module_export_name,
    exported: Option<PyModuleExportName> = crate::conversions::conv_option_module_export_name,
    is_type_only: bool = conv_bool
});

pub fn conv_export_specifier(py: Python<'_>, spec: swc_core::ecma::ast::ExportSpecifier) -> PyResult<Py<PyExportSpecifier>>{
    let base = PyExportSpecifier { };
    Ok(match spec {
        swc_core::ecma::ast::ExportSpecifier::Namespace(n) => Py::new(py, (PyExportNamespaceSpecifier::build(py, n)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ExportSpecifier::Default(d) => Py::new(py, (PyExportDefaultSpecifier::build(py, d)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ExportSpecifier::Named(n) => Py::new(py, (PyExportNamedSpecifier::build(py, n)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_export_specifiers(py: Python<'_>, specs: Vec<swc_core::ecma::ast::ExportSpecifier>) -> PyResult<Vec<Py<PyExportSpecifier>>>{
    specs.into_iter().map(|s| conv_export_specifier(py, s)).collect()
}

ast_node_variant!(PyModuleDecl, PyNamedExport, NamedExport, {
    span: PySpan = conv_span,
    specifiers: Vec<Py<PyExportSpecifier>> = conv_export_specifiers,
    src: Option<String> = crate::conversions::conv_option_boxed_str,
    type_only: bool = conv_bool,
    with: Option<Py<PyExpr>> = conv_option_boxed_object_lit
});

#[pyclass(subclass)]
pub struct PyDefaultDecl{

}

#[pyclass(extends=PyDefaultDecl)]
pub struct PyDefaultDeclClass{
    #[pyo3(get)]
    pub ident: Option<PyIdent>,
    #[pyo3(get)]
    pub class: Py<crate::pyclass::PyClass>
}

impl PyDefaultDeclClass {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::ClassExpr) -> PyResult<Self> {
        Ok(PyDefaultDeclClass {
            ident: match node.ident { Some(i) => Some(conv_ident(py, i)?), None => None },
            class: conv_boxed_class(py, node.class)?
        })
    }
}

#[pyclass(extends=PyDefaultDecl)]
pub struct PyDefaultDeclFn{
    #[pyo3(get)]
    pub ident: Option<PyIdent>,
    #[pyo3(get)]
    pub function: Py<PyFunction>
}

impl PyDefaultDeclFn {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::FnExpr) -> PyResult<Self> {
        Ok(PyDefaultDeclFn {
            ident: match node.ident { Some(i) => Some(conv_ident(py, i)?), None => None },
            function: conv_boxed_function(py, node.function)?
        })
    }
}

#[pyclass(extends=PyDefaultDecl)]
pub struct PyDefaultDeclTsInterface{
    #[pyo3(get)]
    pub interface: Py<crate::pydecl::PyTsInterfaceDecl>
}

impl PyDefaultDeclTsInterface {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::TsInterfaceDecl) -> PyResult<Self> {
        let base = PyDecl { };
        let sub = crate::pydecl::PyTsInterfaceDecl::build(py, node)?;
        Ok(PyDefaultDeclTsInterface { interface: Py::new(py, (sub, base))? })
    }
}

pub fn conv_default_decl(py: Python<'_>, decl: swc_core::ecma::ast::DefaultDecl) -> PyResult<Py<PyDefaultDecl>>{
    let base = PyDefaultDecl { };
    Ok(match decl {
        swc_core::ecma::ast::DefaultDecl::Class(c) => Py::new(py, (PyDefaultDeclClass::build(py, c)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::DefaultDecl::Fn(f) => Py::new(py, (PyDefaultDeclFn::build(py, f)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::DefaultDecl::TsInterfaceDecl(i) => Py::new(py, (PyDefaultDeclTsInterface::build(py, *i)?, base))?.into_bound(py).into_super().unbind(),
    })
}

ast_node_variant!(PyModuleDecl, PyExportDefaultDecl, ExportDefaultDecl, {
    span: PySpan = conv_span,
    decl: Py<PyDefaultDecl> = conv_default_decl
});

#[pyclass]
pub struct PyTsExternalModuleRef{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub expr: String
}

#[pyclass]
pub struct PyTsModuleRef{
    #[pyo3(get)]
    pub entity_name: Option<Py<PyTsEntityName>>,
    #[pyo3(get)]
    pub external_module_ref: Option<Py<PyTsExternalModuleRef>>
}

ast_node_variant!(PyModuleDecl, PyTsImportEqualsDecl, swc_core::ecma::ast::TsImportEqualsDecl, {
    span: PySpan = conv_span,
    is_export: bool = conv_bool,
    is_type_only: bool = conv_bool,
    id: PyIdent = conv_ident,
    module_ref: Py<PyTsModuleRef> = crate::conversions::conv_ts_module_ref
});

ast_node_variant!(PyModuleDecl, PyTsExportAssignment, TsExportAssignment, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyModuleDecl, PyTsNamespaceExportDecl, TsNamespaceExportDecl, {
    span: PySpan = conv_span,
    id: PyIdent = conv_ident
});

pub fn conv_module_decl(py: Python<'_>, decl: swc_core::ecma::ast::ModuleDecl) -> PyResult<Py<PyModuleDecl>>{
    let base = PyModuleDecl { };
    Ok(match decl {
        swc_core::ecma::ast::ModuleDecl::Import(d) => Py::new(py, (PyImportDecl::build(py, d)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ModuleDecl::ExportDecl(d) => Py::new(py, (PyExportDecl::build(py, d)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ModuleDecl::ExportNamed(d) => Py::new(py, (PyNamedExport::build(py, d)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ModuleDecl::ExportDefaultDecl(d) => Py::new(py, (PyExportDefaultDecl::build(py, d)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ModuleDecl::ExportDefaultExpr(d) => Py::new(py, (PyExportDefaultExpr::build(py, d)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ModuleDecl::ExportAll(d) => Py::new(py, (PyExportAll::build(py, d)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ModuleDecl::TsImportEquals(d) => Py::new(py, (PyTsImportEqualsDecl::build(py, *d)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ModuleDecl::TsExportAssignment(d) => Py::new(py, (PyTsExportAssignment::build(py, d)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ModuleDecl::TsNamespaceExport(d) => Py::new(py, (PyTsNamespaceExportDecl::build(py, d)?, base))?.into_bound(py).into_super().unbind(),
    })
}

#[pyclass]
pub struct PySourceModule{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub body: Vec<Py<PyModuleItem>>,
    #[pyo3(get)]
    pub shebang: Option<String>
}

pub fn conv_module(py: Python<'_>, module: swc_core::ecma::ast::Module) -> PyResult<Py<PySourceModule>>{
    Py::new(py, PySourceModule{
        span: conv_span(py, module.span)?,
        body: crate::conversions::conv_module_items(py, module.body)?,
        shebang: module.shebang.map(|s| s.to_string())
    })
}
