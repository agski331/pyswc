use pyo3::prelude::*;
use swc_core::ecma::ast::{JSXElement, JSXEmptyExpr, JSXFragment, JSXMemberExpr, JSXNamespacedName};

use crate::{
    conversions::{conv_boxed_expr, conv_bool, conv_identname, conv_span, conv_typeparams},
    macros::ast_node_variant,
    pyexpr::PyExpr,
    pyident::{PyIdent, PyIdentName},
    pyprop::{PyPropOrSpread, PySpreadElement},
    pyspan::PySpan,
    pytypeinfo::PyTsTypeParamInstantiation,
};

#[pyclass]
pub struct PyJSXObject{
    #[pyo3(get)]
    pub jsx_member_expr: Option<Py<PyJSXMemberExpr>>,
    #[pyo3(get)]
    pub ident: Option<PyIdent>
}

ast_node_variant!(PyExpr, PyJSXMemberExpr, JSXMemberExpr, {
    span: PySpan = conv_span,
    obj: Py<PyJSXObject> = crate::conversions::conv_jsx_object,
    prop: PyIdentName = conv_identname
});

ast_node_variant!(PyExpr, PyJSXNamespacedName, JSXNamespacedName, {
    span: PySpan = conv_span,
    ns: PyIdentName = conv_identname,
    name: PyIdentName = conv_identname
});

ast_node_variant!(PyExpr, PyJSXEmptyExpr, JSXEmptyExpr, {
    span: PySpan = conv_span
});

#[pyclass]
pub struct PyJSXExpr{
    #[pyo3(get)]
    pub empty_span: Option<PySpan>,
    #[pyo3(get)]
    pub expr: Option<Py<PyExpr>>
}

#[pyclass]
pub struct PyJSXExprContainer{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub expr: Py<PyJSXExpr>
}

impl PyJSXExprContainer {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXExprContainer) -> PyResult<Self> {
        Ok(PyJSXExprContainer {
            span: conv_span(py, node.span)?,
            expr: crate::conversions::conv_jsx_expr(py, node.expr)?
        })
    }
}

#[pyclass]
pub struct PyJSXSpreadChild{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub expr: Py<PyExpr>
}

impl PyJSXSpreadChild {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXSpreadChild) -> PyResult<Self> {
        Ok(PyJSXSpreadChild {
            span: conv_span(py, node.span)?,
            expr: conv_boxed_expr(py, node.expr)?
        })
    }
}

#[pyclass]
pub struct PyJSXElementName{
    #[pyo3(get)]
    pub ident: Option<PyIdent>,
    #[pyo3(get)]
    pub jsx_member_expr: Option<Py<PyJSXMemberExprData>>,
    #[pyo3(get)]
    pub jsx_namespaced_name: Option<PyJSXNamespacedNameData>
}

#[pyclass]
pub struct PyJSXMemberExprData{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub obj: Py<PyJSXObject>,
    #[pyo3(get)]
    pub prop: PyIdentName
}

#[derive(Clone)]
#[pyclass]
pub struct PyJSXNamespacedNameData{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub ns: PyIdentName,
    #[pyo3(get)]
    pub name: PyIdentName
}

#[pyclass(subclass)]
pub struct PyJSXAttrOrSpread{

}

#[pyclass(extends=PyJSXAttrOrSpread)]
pub struct PyJSXAttrOrSpreadAttr{
    #[pyo3(get)]
    pub attr: Py<PyJSXAttr>
}

impl PyJSXAttrOrSpreadAttr {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXAttr) -> PyResult<Self> {
        Ok(PyJSXAttrOrSpreadAttr { attr: Py::new(py, PyJSXAttr::build(py, node)?)? })
    }
}

#[pyclass(extends=PyJSXAttrOrSpread)]
pub struct PyJSXAttrOrSpreadSpread{
    #[pyo3(get)]
    pub spread: Py<PySpreadElement>
}

impl PyJSXAttrOrSpreadSpread {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::SpreadElement) -> PyResult<Self> {
        let base = PyPropOrSpread { };
        let sub = PySpreadElement::build(py, node)?;
        Ok(PyJSXAttrOrSpreadSpread { spread: Py::new(py, (sub, base))? })
    }
}

pub fn conv_jsx_attr_or_spread(py: Python<'_>, node: swc_core::ecma::ast::JSXAttrOrSpread) -> PyResult<Py<PyJSXAttrOrSpread>>{
    let base = PyJSXAttrOrSpread { };
    Ok(match node {
        swc_core::ecma::ast::JSXAttrOrSpread::JSXAttr(a) => Py::new(py, (PyJSXAttrOrSpreadAttr::build(py, a)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::JSXAttrOrSpread::SpreadElement(s) => Py::new(py, (PyJSXAttrOrSpreadSpread::build(py, s)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_jsx_attr_or_spreads(py: Python<'_>, nodes: Vec<swc_core::ecma::ast::JSXAttrOrSpread>) -> PyResult<Vec<Py<PyJSXAttrOrSpread>>>{
    nodes.into_iter().map(|n| conv_jsx_attr_or_spread(py, n)).collect()
}

#[derive(Clone)]
#[pyclass]
pub struct PyJSXAttrName{
    #[pyo3(get)]
    pub ident: Option<PyIdentName>,
    #[pyo3(get)]
    pub jsx_namespaced_name: Option<PyJSXNamespacedNameData>
}

#[pyclass(subclass)]
pub struct PyJSXAttrValue{

}

#[pyclass(extends=PyJSXAttrValue)]
pub struct PyJSXAttrValueStr{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub value: String
}

impl PyJSXAttrValueStr {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::Str) -> PyResult<Self> {
        Ok(PyJSXAttrValueStr { span: conv_span(py, node.span)?, value: crate::conversions::conv_str(py, node)? })
    }
}

#[pyclass(extends=PyJSXAttrValue)]
pub struct PyJSXAttrValueExprContainer{
    #[pyo3(get)]
    pub container: Py<PyJSXExprContainer>
}

impl PyJSXAttrValueExprContainer {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXExprContainer) -> PyResult<Self> {
        Ok(PyJSXAttrValueExprContainer { container: Py::new(py, PyJSXExprContainer::build(py, node)?)? })
    }
}

#[pyclass(extends=PyJSXAttrValue)]
pub struct PyJSXAttrValueElement{
    #[pyo3(get)]
    pub element: Py<PyJSXElement>
}

impl PyJSXAttrValueElement {
    pub fn build(py: Python<'_>, node: Box<swc_core::ecma::ast::JSXElement>) -> PyResult<Self> {
        Ok(PyJSXAttrValueElement { element: crate::conversions::conv_jsx_element(py, *node)? })
    }
}

#[pyclass(extends=PyJSXAttrValue)]
pub struct PyJSXAttrValueFragment{
    #[pyo3(get)]
    pub fragment: Py<PyJSXFragment>
}

impl PyJSXAttrValueFragment {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXFragment) -> PyResult<Self> {
        Ok(PyJSXAttrValueFragment { fragment: crate::conversions::conv_jsx_fragment(py, node)? })
    }
}

pub fn conv_jsx_attr_value(py: Python<'_>, node: swc_core::ecma::ast::JSXAttrValue) -> PyResult<Py<PyJSXAttrValue>>{
    let base = PyJSXAttrValue { };
    Ok(match node {
        swc_core::ecma::ast::JSXAttrValue::Str(s) => Py::new(py, (PyJSXAttrValueStr::build(py, s)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::JSXAttrValue::JSXExprContainer(c) => Py::new(py, (PyJSXAttrValueExprContainer::build(py, c)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::JSXAttrValue::JSXElement(e) => Py::new(py, (PyJSXAttrValueElement::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::JSXAttrValue::JSXFragment(f) => Py::new(py, (PyJSXAttrValueFragment::build(py, f)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_option_jsx_attr_value(py: Python<'_>, node: Option<swc_core::ecma::ast::JSXAttrValue>) -> PyResult<Option<Py<PyJSXAttrValue>>>{
    match node {
        None => Ok(None),
        Some(v) => Ok(Some(conv_jsx_attr_value(py, v)?))
    }
}

#[pyclass]
pub struct PyJSXAttr{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub name: PyJSXAttrName,
    #[pyo3(get)]
    pub value: Option<Py<PyJSXAttrValue>>
}

impl PyJSXAttr {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXAttr) -> PyResult<Self> {
        Ok(PyJSXAttr {
            span: conv_span(py, node.span)?,
            name: crate::conversions::conv_jsx_attr_name(py, node.name)?,
            value: conv_option_jsx_attr_value(py, node.value)?
        })
    }
}

#[pyclass]
pub struct PyJSXOpeningElement{
    #[pyo3(get)]
    pub name: Py<PyJSXElementName>,
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub attrs: Vec<Py<PyJSXAttrOrSpread>>,
    #[pyo3(get)]
    pub self_closing: bool,
    #[pyo3(get)]
    pub type_args: Option<Py<PyTsTypeParamInstantiation>>
}

impl PyJSXOpeningElement {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXOpeningElement) -> PyResult<Self> {
        Ok(PyJSXOpeningElement {
            name: crate::conversions::conv_jsx_element_name(py, node.name)?,
            span: conv_span(py, node.span)?,
            attrs: conv_jsx_attr_or_spreads(py, node.attrs)?,
            self_closing: node.self_closing,
            type_args: conv_typeparams(py, node.type_args)?
        })
    }
}

#[pyclass]
pub struct PyJSXClosingElement{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub name: Py<PyJSXElementName>
}

impl PyJSXClosingElement {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXClosingElement) -> PyResult<Self> {
        Ok(PyJSXClosingElement {
            span: conv_span(py, node.span)?,
            name: crate::conversions::conv_jsx_element_name(py, node.name)?
        })
    }
}

#[pyclass(subclass)]
pub struct PyJSXElementChild{

}

#[pyclass(extends=PyJSXElementChild)]
pub struct PyJSXElementChildText{
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub value: String,
    #[pyo3(get)]
    pub raw: String
}

impl PyJSXElementChildText {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXText) -> PyResult<Self> {
        Ok(PyJSXElementChildText {
            span: conv_span(py, node.span)?,
            value: node.value.to_atom_lossy().to_string(),
            raw: node.raw.to_string()
        })
    }
}

#[pyclass(extends=PyJSXElementChild)]
pub struct PyJSXElementChildExprContainer{
    #[pyo3(get)]
    pub container: Py<PyJSXExprContainer>
}

impl PyJSXElementChildExprContainer {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXExprContainer) -> PyResult<Self> {
        Ok(PyJSXElementChildExprContainer { container: Py::new(py, PyJSXExprContainer::build(py, node)?)? })
    }
}

#[pyclass(extends=PyJSXElementChild)]
pub struct PyJSXElementChildSpreadChild{
    #[pyo3(get)]
    pub spread_child: Py<PyJSXSpreadChild>
}

impl PyJSXElementChildSpreadChild {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXSpreadChild) -> PyResult<Self> {
        Ok(PyJSXElementChildSpreadChild { spread_child: Py::new(py, PyJSXSpreadChild::build(py, node)?)? })
    }
}

#[pyclass(extends=PyJSXElementChild)]
pub struct PyJSXElementChildElement{
    #[pyo3(get)]
    pub element: Py<PyJSXElement>
}

impl PyJSXElementChildElement {
    pub fn build(py: Python<'_>, node: Box<swc_core::ecma::ast::JSXElement>) -> PyResult<Self> {
        Ok(PyJSXElementChildElement { element: crate::conversions::conv_jsx_element(py, *node)? })
    }
}

#[pyclass(extends=PyJSXElementChild)]
pub struct PyJSXElementChildFragment{
    #[pyo3(get)]
    pub fragment: Py<PyJSXFragment>
}

impl PyJSXElementChildFragment {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXFragment) -> PyResult<Self> {
        Ok(PyJSXElementChildFragment { fragment: crate::conversions::conv_jsx_fragment(py, node)? })
    }
}

pub fn conv_jsx_element_child(py: Python<'_>, node: swc_core::ecma::ast::JSXElementChild) -> PyResult<Py<PyJSXElementChild>>{
    let base = PyJSXElementChild { };
    Ok(match node {
        swc_core::ecma::ast::JSXElementChild::JSXText(t) => Py::new(py, (PyJSXElementChildText::build(py, t)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::JSXElementChild::JSXExprContainer(c) => Py::new(py, (PyJSXElementChildExprContainer::build(py, c)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::JSXElementChild::JSXSpreadChild(s) => Py::new(py, (PyJSXElementChildSpreadChild::build(py, s)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::JSXElementChild::JSXElement(e) => Py::new(py, (PyJSXElementChildElement::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        swc_core::ecma::ast::JSXElementChild::JSXFragment(f) => Py::new(py, (PyJSXElementChildFragment::build(py, f)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_jsx_element_children(py: Python<'_>, nodes: Vec<swc_core::ecma::ast::JSXElementChild>) -> PyResult<Vec<Py<PyJSXElementChild>>>{
    nodes.into_iter().map(|n| conv_jsx_element_child(py, n)).collect()
}

ast_node_variant!(PyExpr, PyJSXElement, JSXElement, {
    span: PySpan = conv_span,
    opening: Py<PyJSXOpeningElement> = crate::conversions::conv_jsx_opening_element,
    children: Vec<Py<PyJSXElementChild>> = conv_jsx_element_children,
    closing: Option<Py<PyJSXClosingElement>> = crate::conversions::conv_option_jsx_closing_element
});

#[derive(Clone)]
#[pyclass]
pub struct PyJSXOpeningFragment{
    #[pyo3(get)]
    pub span: PySpan
}

impl PyJSXOpeningFragment {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXOpeningFragment) -> PyResult<Self> {
        Ok(PyJSXOpeningFragment { span: conv_span(py, node.span)? })
    }
}

#[derive(Clone)]
#[pyclass]
pub struct PyJSXClosingFragment{
    #[pyo3(get)]
    pub span: PySpan
}

impl PyJSXClosingFragment {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXClosingFragment) -> PyResult<Self> {
        Ok(PyJSXClosingFragment { span: conv_span(py, node.span)? })
    }
}

ast_node_variant!(PyExpr, PyJSXFragment, JSXFragment, {
    span: PySpan = conv_span,
    opening: PyJSXOpeningFragment = PyJSXOpeningFragment::build,
    children: Vec<Py<PyJSXElementChild>> = conv_jsx_element_children,
    closing: PyJSXClosingFragment = PyJSXClosingFragment::build
});
