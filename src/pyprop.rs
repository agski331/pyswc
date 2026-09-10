use pyo3::prelude::*;
use swc_core::ecma::ast::{
    AssignProp, BigInt as SwcBigInt, ComputedPropName, GetterProp, Ident, IdentName,
    KeyValueProp, MethodProp, Number, SetterProp, SpreadElement, Str,
};

use crate::{
    conversions::{
        conv_atom, conv_bigint_value, conv_boxed_expr, conv_boxed_function, conv_f64, conv_ident,
        conv_propname, conv_span, conv_wtf8atom,
    }, macros::ast_node_variant, pyexpr::PyExpr, pyfunction::PyFunction, pyident::{PyIdent, PyIdentName, PyPrivateName}, pyspan::PySpan,
};

#[pyclass(subclass)]
pub struct PyPropName{

}

ast_node_variant!(PyPropName, PyIdentPropName, IdentName, {
    span: PySpan = conv_span,
    sym: String = conv_atom
});

ast_node_variant!(PyPropName, PyStrPropName, Str, {
    span: PySpan = conv_span,
    value: String = conv_wtf8atom
});

ast_node_variant!(PyPropName, PyNumPropName, Number, {
    span: PySpan = conv_span,
    value: f64 = conv_f64
});

ast_node_variant!(PyPropName, PyComputedPropName, ComputedPropName, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyPropName, PyBigIntPropName, SwcBigInt, {
    span: PySpan = conv_span,
    value: num_bigint::BigInt = conv_bigint_value
});

#[pyclass]
pub struct PyExprOrSpread{
    #[pyo3(get)]
    pub spread: Option<PySpan>,
    #[pyo3(get)]
    pub expr: Py<PyExpr>
}

#[pyclass(subclass)]
pub struct PyProp{

}

#[pyclass(extends=PyProp)]
pub struct PyShorthandProp{
    #[pyo3(get)]
    pub ident: PyIdent
}

impl PyShorthandProp {
    pub fn build(py: Python<'_>, ident: Ident) -> PyResult<Self> {
        Ok(PyShorthandProp { ident: conv_ident(py, ident)? })
    }
}

ast_node_variant!(PyProp, PyKeyValueProp, KeyValueProp, {
    key: Py<PyPropName> = conv_propname,
    value: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyProp, PyAssignProp, AssignProp, {
    span: PySpan = conv_span,
    key: PyIdent = conv_ident,
    value: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyProp, PyGetterProp, GetterProp, {
    span: PySpan = conv_span,
    key: Py<PyPropName> = conv_propname,
    function: Py<PyFunction> = conv_boxed_function
});

ast_node_variant!(PyProp, PySetterProp, SetterProp, {
    span: PySpan = conv_span,
    key: Py<PyPropName> = conv_propname,
    function: Py<PyFunction> = conv_boxed_function
});

ast_node_variant!(PyProp, PyMethodProp, MethodProp, {
    key: Py<PyPropName> = conv_propname,
    function: Py<PyFunction> = conv_boxed_function
});

#[pyclass(subclass)]
pub struct PyPropOrSpread{

}

#[pyclass(extends=PyPropOrSpread)]
pub struct PyPropOrSpreadProp{
    #[pyo3(get)]
    pub prop: Py<PyProp>
}

impl PyPropOrSpreadProp {
    pub fn build(py: Python<'_>, prop: Box<swc_core::ecma::ast::Prop>) -> PyResult<Self> {
        Ok(PyPropOrSpreadProp { prop: crate::conversions::conv_boxed_prop(py, prop)? })
    }
}

ast_node_variant!(PyPropOrSpread, PySpreadElement, SpreadElement, {
    dot3_token: PySpan = conv_span,
    expr: Py<PyExpr> = conv_boxed_expr
});

#[pyclass]
pub struct PyMemberProp{
    #[pyo3(get)]
    pub ident: Option<PyIdentName>,
    #[pyo3(get)]
    pub private_name: Option<PyPrivateName>,
    #[pyo3(get)]
    pub computed: Option<Py<PyComputedPropName>>
}

#[pyclass]
pub struct PySuperProp{
    #[pyo3(get)]
    pub ident: Option<PyIdentName>,
    #[pyo3(get)]
    pub computed: Option<Py<PyComputedPropName>>
}
