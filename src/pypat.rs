use pyo3::prelude::*;
use swc_core::ecma::ast::{
    ArrayPat, AssignPat, AssignPatProp, BindingIdent, Invalid, KeyValuePatProp, ObjectPat, Pat,
    RestPat,
};

use crate::{
    conversions::{
        conv_bindingident, conv_bool, conv_boxed_expr, conv_boxed_pat, conv_object_pat_props,
        conv_option_boxed_expr, conv_option_tstypeann, conv_pat, conv_pat_elems, conv_propname,
        conv_span,
    },
    macros::ast_node_variant,
    pyexpr::PyExpr,
    pyident::PyBindingIdent,
    pyprop::PyPropName,
    pyspan::PySpan,
    pytypeinfo::PyTsTypeAnn,
};

#[pyclass(subclass)]
pub struct PyPat{

}

#[pyclass(extends=PyPat)]
pub struct PyBindingIdentPat{
    #[pyo3(get)]
    pub ident: Py<PyBindingIdent>
}

impl PyBindingIdentPat {
    pub fn build(py: Python<'_>, node: BindingIdent) -> PyResult<Self> {
        Ok(PyBindingIdentPat { ident: conv_bindingident(py, node)? })
    }
}

ast_node_variant!(PyPat, PyArrayPat, ArrayPat, {
    span: PySpan = conv_span,
    elems: Vec<Option<Py<PyPat>>> = conv_pat_elems,
    optional: bool = conv_bool,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann
});

ast_node_variant!(PyPat, PyRestPat, RestPat, {
    span: PySpan = conv_span,
    dot3_token: PySpan = conv_span,
    arg: Py<PyPat> = conv_boxed_pat,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann
});

ast_node_variant!(PyPat, PyObjectPat, ObjectPat, {
    span: PySpan = conv_span,
    props: Vec<Py<PyObjectPatProp>> = conv_object_pat_props,
    optional: bool = conv_bool,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann
});

ast_node_variant!(PyPat, PyAssignPat, AssignPat, {
    span: PySpan = conv_span,
    left: Py<PyPat> = conv_boxed_pat,
    right: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyPat, PyInvalidPat, Invalid, {
    span: PySpan = conv_span
});

#[pyclass(extends=PyPat)]
pub struct PyExprPat{
    #[pyo3(get)]
    pub expr: Py<PyExpr>
}

impl PyExprPat {
    pub fn build(py: Python<'_>, expr: Box<swc_core::ecma::ast::Expr>) -> PyResult<Self> {
        Ok(PyExprPat { expr: conv_boxed_expr(py, expr)? })
    }
}


pub fn pat_to_py(py: Python<'_>, pat: Pat) -> PyResult<Py<PyPat>> {
    conv_pat(py, pat)
}

#[pyclass(subclass)]
pub struct PyObjectPatProp{

}

ast_node_variant!(PyObjectPatProp, PyKeyValuePatProp, KeyValuePatProp, {
    key: Py<PyPropName> = conv_propname,
    value: Py<PyPat> = conv_boxed_pat
});

ast_node_variant!(PyObjectPatProp, PyAssignPatProp, AssignPatProp, {
    span: PySpan = conv_span,
    key: Py<PyBindingIdent> = conv_bindingident,
    value: Option<Py<PyExpr>> = conv_option_boxed_expr
});

ast_node_variant!(PyObjectPatProp, PyObjectPatRestProp, RestPat, {
    span: PySpan = conv_span,
    dot3_token: PySpan = conv_span,
    arg: Py<PyPat> = conv_boxed_pat,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann
});


pub fn object_pat_prop_to_py(py: Python<'_>, prop: swc_core::ecma::ast::ObjectPatProp) -> PyResult<Py<PyObjectPatProp>> {
    crate::conversions::conv_object_pat_prop(py, prop)
}
