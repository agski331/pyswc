use pyo3::prelude::*;
use swc_core::ecma::ast::{
    AutoAccessor, ClassMethod, ClassProp, Constructor, EmptyStmt, PrivateMethod, PrivateProp,
    StaticBlock, TsIndexSignature,
};

use crate::{
    conversions::{
        conv_block_stmt, conv_bool, conv_boxed_expr, conv_boxed_function, conv_ctxt,
        conv_decorators, conv_fn_params, conv_key, conv_method_kind, conv_option_accessibility,
        conv_option_boxed_expr, conv_option_function_body, conv_option_tstypeann, conv_private_name,
        conv_propname, conv_span,
    },
    macros::ast_node_variant,
    pyexpr::PyExpr,
    pyfunction::{PyDecorator, PyFunction, PyFunctionBody},
    pyident::PyPrivateName,
    pypat::PyPat,
    pyprop::PyPropName,
    pyspan::PySpan,
    pystmts::PyBlockStmt,
    pytypeinfo::PyTsTypeAnn,
};

#[pyclass]
pub struct PyKey{
    #[pyo3(get)]
    pub private: Option<PyPrivateName>,
    #[pyo3(get)]
    pub public: Option<Py<PyPropName>>
}

#[pyclass(subclass)]
pub struct PyClassMember{

}

ast_node_variant!(PyClassMember, PyConstructor, Constructor, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    key: Py<PyPropName> = conv_propname,
    params: Vec<Py<PyParamOrTsParamProp>> = conv_param_or_ts_param_props,
    body: Option<Py<PyFunctionBody>> = conv_option_function_body,
    accessibility: Option<u32> = conv_option_accessibility,
    is_optional: bool = conv_bool
});

ast_node_variant!(PyClassMember, PyClassMethod, ClassMethod, {
    span: PySpan = conv_span,
    key: Py<PyPropName> = conv_propname,
    function: Py<PyFunction> = conv_boxed_function,
    kind: u32 = conv_method_kind,
    is_static: bool = conv_bool,
    accessibility: Option<u32> = conv_option_accessibility,
    is_abstract: bool = conv_bool,
    is_optional: bool = conv_bool,
    is_override: bool = conv_bool
});

ast_node_variant!(PyClassMember, PyPrivateMethod, PrivateMethod, {
    span: PySpan = conv_span,
    key: PyPrivateName = conv_private_name,
    function: Py<PyFunction> = conv_boxed_function,
    kind: u32 = conv_method_kind,
    is_static: bool = conv_bool,
    accessibility: Option<u32> = conv_option_accessibility,
    is_abstract: bool = conv_bool,
    is_optional: bool = conv_bool,
    is_override: bool = conv_bool
});

ast_node_variant!(PyClassMember, PyClassProp, ClassProp, {
    span: PySpan = conv_span,
    key: Py<PyPropName> = conv_propname,
    value: Option<Py<PyExpr>> = conv_option_boxed_expr,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann,
    is_static: bool = conv_bool,
    decorators: Vec<Py<PyDecorator>> = conv_decorators,
    accessibility: Option<u32> = conv_option_accessibility,
    is_abstract: bool = conv_bool,
    is_optional: bool = conv_bool,
    is_override: bool = conv_bool,
    readonly: bool = conv_bool,
    declare: bool = conv_bool,
    definite: bool = conv_bool
});

ast_node_variant!(PyClassMember, PyPrivateProp, PrivateProp, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    key: PyPrivateName = conv_private_name,
    value: Option<Py<PyExpr>> = conv_option_boxed_expr,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann,
    is_static: bool = conv_bool,
    decorators: Vec<Py<PyDecorator>> = conv_decorators,
    accessibility: Option<u32> = conv_option_accessibility,
    is_optional: bool = conv_bool,
    is_override: bool = conv_bool,
    readonly: bool = conv_bool,
    definite: bool = conv_bool
});

ast_node_variant!(PyClassMember, PyClassIndexSignature, TsIndexSignature, {
    params: Vec<Py<PyPat>> = conv_fn_params,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann,
    readonly: bool = conv_bool,
    is_static: bool = conv_bool,
    span: PySpan = conv_span
});

ast_node_variant!(PyClassMember, PyClassEmptyMember, EmptyStmt, {
    span: PySpan = conv_span
});

ast_node_variant!(PyClassMember, PyStaticBlock, StaticBlock, {
    span: PySpan = conv_span,
    body: Py<PyBlockStmt> = conv_block_stmt
});

ast_node_variant!(PyClassMember, PyAutoAccessor, AutoAccessor, {
    span: PySpan = conv_span,
    key: Py<PyKey> = conv_key,
    value: Option<Py<PyExpr>> = conv_option_boxed_expr,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann,
    is_static: bool = conv_bool,
    decorators: Vec<Py<PyDecorator>> = conv_decorators,
    accessibility: Option<u32> = conv_option_accessibility,
    is_abstract: bool = conv_bool,
    is_override: bool = conv_bool,
    definite: bool = conv_bool
});

#[pyclass(subclass)]
pub struct PyParamOrTsParamProp{

}

#[pyclass(extends=PyParamOrTsParamProp)]
pub struct PyParamOrTsParamPropParam{
    #[pyo3(get)]
    pub param: Py<PyParam>
}

#[pyclass(extends=PyParamOrTsParamProp)]
pub struct PyParamOrTsParamPropTsParamProp{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub decorators: Vec<Py<PyDecorator>>,
    #[pyo3(get)]
    pub accessibility: Option<u32>,
    #[pyo3(get)]
    pub is_override: bool,
    #[pyo3(get)]
    pub readonly: bool,
    #[pyo3(get)]
    pub param_ident: Option<Py<crate::pyident::PyBindingIdent>>,
    #[pyo3(get)]
    pub param_assign: Option<Py<crate::pypat::PyAssignPat>>
}

impl PyParamOrTsParamPropParam {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::Param) -> PyResult<Self> {
        Ok(PyParamOrTsParamPropParam { param: crate::conversions::conv_param(py, node)? })
    }
}

impl PyParamOrTsParamPropTsParamProp {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::TsParamProp) -> PyResult<Self> {
        let (param_ident, param_assign) = match node.param {
            swc_core::ecma::ast::TsParamPropParam::Ident(i) => (Some(crate::conversions::conv_bindingident(py, i)?), None),
            swc_core::ecma::ast::TsParamPropParam::Assign(a) => {
                let base = PyPat { };
                let sub = crate::pypat::PyAssignPat::build(py, a)?;
                (None, Some(Py::new(py, (sub, base))?))
            }
        };
        Ok(PyParamOrTsParamPropTsParamProp {
            span: conv_span(py, node.span)?,
            decorators: conv_decorators(py, node.decorators)?,
            accessibility: conv_option_accessibility(py, node.accessibility)?,
            is_override: node.is_override,
            readonly: node.readonly,
            param_ident,
            param_assign
        })
    }
}

pub fn conv_param_or_ts_param_prop(py: Python<'_>, node: swc_core::ecma::ast::ParamOrTsParamProp) -> PyResult<Py<PyParamOrTsParamProp>>{
    let base = PyParamOrTsParamProp { };
    Ok(match node {
        swc_core::ecma::ast::ParamOrTsParamProp::Param(p) => Py::new(py, (PyParamOrTsParamPropParam::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::ParamOrTsParamProp::TsParamProp(p) => Py::new(py, (PyParamOrTsParamPropTsParamProp::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_param_or_ts_param_props(py: Python<'_>, nodes: Vec<swc_core::ecma::ast::ParamOrTsParamProp>) -> PyResult<Vec<Py<PyParamOrTsParamProp>>>{
    nodes.into_iter().map(|n| conv_param_or_ts_param_prop(py, n)).collect()
}

use crate::pyfunction::PyParam;

#[pyclass]
pub struct PyClass{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub ctxt: u32,
    #[pyo3(get)]
    pub decorators: Vec<Py<PyDecorator>>,
    #[pyo3(get)]
    pub body: Vec<Py<PyClassMember>>,
    #[pyo3(get)]
    pub super_class: Option<Py<PyExpr>>,
    #[pyo3(get)]
    pub is_abstract: bool,
    #[pyo3(get)]
    pub type_params: Option<Py<crate::pytypeinfo::PyTsTypeParamDecl>>,
    #[pyo3(get)]
    pub super_type_params: Option<Py<crate::pytypeinfo::PyTsTypeParamInstantiation>>,
    #[pyo3(get)]
    pub implements: Vec<Py<crate::pytypeinfo::PyTsExprWithTypeArgs>>
}
