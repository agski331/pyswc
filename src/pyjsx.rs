use pyo3::prelude::*;
use std::sync::Arc;
use swc_core::common::Span;
use swc_core::ecma::ast::{
    JSXClosingElement, JSXElement, JSXElementChild, JSXEmptyExpr, JSXExprContainer, JSXFragment,
    JSXMemberExpr, JSXNamespacedName, JSXOpeningElement, JSXOpeningFragment, JSXSpreadChild,
    JSXText,
};

use crate::{
    conversions::{conv_bool, conv_boxed_expr, conv_identname, conv_span, conv_typeparams},
    macros::{ast_node_variant, lazy_leaf_node},
    pyexpr::PyExpr,
    pyident::{PyIdent, PyIdentName},
    pyprop::{PyPropOrSpread, PySpreadElement},
    pyspan::PySpan,
    pytypeinfo::PyTsTypeParamInstantiation,
};

#[pyclass]
pub struct PyJSXObject {
    #[pyo3(get)]
    pub jsx_member_expr: Option<Py<PyJSXMemberExpr>>,
    #[pyo3(get)]
    pub ident: Option<PyIdent>,
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
pub struct PyJSXExpr {
    #[pyo3(get)]
    pub empty_span: Option<PySpan>,
    #[pyo3(get)]
    pub expr: Option<Py<PyExpr>>,
}

#[pyclass]
pub struct PyJSXExprContainer {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub expr: Py<PyJSXExpr>,
}

impl PyJSXExprContainer {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXExprContainer) -> PyResult<Self> {
        Ok(PyJSXExprContainer {
            span: conv_span(py, node.span)?,
            expr: crate::conversions::conv_jsx_expr(py, node.expr)?,
        })
    }
}

#[pyclass]
pub struct PyJSXSpreadChild {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub expr: Py<PyExpr>,
}

impl PyJSXSpreadChild {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXSpreadChild) -> PyResult<Self> {
        Ok(PyJSXSpreadChild {
            span: conv_span(py, node.span)?,
            expr: conv_boxed_expr(py, node.expr)?,
        })
    }
}

#[pyclass]
pub struct PyJSXElementName {
    #[pyo3(get)]
    pub ident: Option<PyIdent>,
    #[pyo3(get)]
    pub jsx_member_expr: Option<Py<PyJSXMemberExprData>>,
    #[pyo3(get)]
    pub jsx_namespaced_name: Option<PyJSXNamespacedNameData>,
}

#[pyclass]
pub struct PyJSXMemberExprData {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub obj: Py<PyJSXObject>,
    #[pyo3(get)]
    pub prop: PyIdentName,
}

lazy_leaf_node!(PyJSXNamespacedNameData, JSXNamespacedName, {
    span: PySpan = conv_span,
    ns: PyIdentName = conv_identname,
    name: PyIdentName = conv_identname,
});

#[pyclass(subclass)]
pub struct PyJSXAttrOrSpread {}

#[pyclass(extends=PyJSXAttrOrSpread)]
pub struct PyJSXAttrOrSpreadAttr {
    #[pyo3(get)]
    pub attr: Py<PyJSXAttr>,
}

impl PyJSXAttrOrSpreadAttr {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXAttr) -> PyResult<Self> {
        Ok(PyJSXAttrOrSpreadAttr {
            attr: Py::new(py, PyJSXAttr::build(py, node)?)?,
        })
    }
}

#[pyclass(extends=PyJSXAttrOrSpread)]
pub struct PyJSXAttrOrSpreadSpread {
    #[pyo3(get)]
    pub spread: Py<PySpreadElement>,
}

impl PyJSXAttrOrSpreadSpread {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::SpreadElement) -> PyResult<Self> {
        let base = PyPropOrSpread {};
        let data = Arc::new(crate::pyprop::PropOrSpreadData::Spread(
            crate::pyprop::lower_spread_element(node),
        ));
        let sub = PySpreadElement::from_arc(data);
        Ok(PyJSXAttrOrSpreadSpread {
            spread: Py::new(py, (sub, base))?,
        })
    }
}

pub fn conv_jsx_attr_or_spread(
    py: Python<'_>,
    node: swc_core::ecma::ast::JSXAttrOrSpread,
) -> PyResult<Py<PyJSXAttrOrSpread>> {
    let base = PyJSXAttrOrSpread {};
    Ok(match node {
        swc_core::ecma::ast::JSXAttrOrSpread::JSXAttr(a) => {
            Py::new(py, (PyJSXAttrOrSpreadAttr::build(py, a)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        swc_core::ecma::ast::JSXAttrOrSpread::SpreadElement(s) => {
            Py::new(py, (PyJSXAttrOrSpreadSpread::build(py, s)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
    })
}

pub fn conv_jsx_attr_or_spreads(
    py: Python<'_>,
    nodes: Vec<swc_core::ecma::ast::JSXAttrOrSpread>,
) -> PyResult<Vec<Py<PyJSXAttrOrSpread>>> {
    nodes
        .into_iter()
        .map(|n| conv_jsx_attr_or_spread(py, n))
        .collect()
}

#[derive(Clone)]
#[pyclass]
pub struct PyJSXAttrName {
    #[pyo3(get)]
    pub ident: Option<PyIdentName>,
    #[pyo3(get)]
    pub jsx_namespaced_name: Option<PyJSXNamespacedNameData>,
}

#[pyclass(subclass)]
pub struct PyJSXAttrValue {}

#[pyclass(extends=PyJSXAttrValue)]
pub struct PyJSXAttrValueStr {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub value: String,
}

impl PyJSXAttrValueStr {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::Str) -> PyResult<Self> {
        Ok(PyJSXAttrValueStr {
            span: conv_span(py, node.span)?,
            value: crate::conversions::conv_str(py, node)?,
        })
    }
}

#[pyclass(extends=PyJSXAttrValue)]
pub struct PyJSXAttrValueExprContainer {
    #[pyo3(get)]
    pub container: Py<PyJSXExprContainer>,
}

impl PyJSXAttrValueExprContainer {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXExprContainer) -> PyResult<Self> {
        Ok(PyJSXAttrValueExprContainer {
            container: Py::new(py, PyJSXExprContainer::build(py, node)?)?,
        })
    }
}

#[pyclass(extends=PyJSXAttrValue)]
pub struct PyJSXAttrValueElement {
    #[pyo3(get)]
    pub element: Py<PyJSXElement>,
}

impl PyJSXAttrValueElement {
    pub fn build(py: Python<'_>, node: Box<swc_core::ecma::ast::JSXElement>) -> PyResult<Self> {
        Ok(PyJSXAttrValueElement {
            element: crate::conversions::conv_jsx_element(py, *node)?,
        })
    }
}

#[pyclass(extends=PyJSXAttrValue)]
pub struct PyJSXAttrValueFragment {
    #[pyo3(get)]
    pub fragment: Py<PyJSXFragment>,
}

impl PyJSXAttrValueFragment {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXFragment) -> PyResult<Self> {
        Ok(PyJSXAttrValueFragment {
            fragment: crate::conversions::conv_jsx_fragment(py, node)?,
        })
    }
}

pub fn conv_jsx_attr_value(
    py: Python<'_>,
    node: swc_core::ecma::ast::JSXAttrValue,
) -> PyResult<Py<PyJSXAttrValue>> {
    let base = PyJSXAttrValue {};
    Ok(match node {
        swc_core::ecma::ast::JSXAttrValue::Str(s) => {
            Py::new(py, (PyJSXAttrValueStr::build(py, s)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        swc_core::ecma::ast::JSXAttrValue::JSXExprContainer(c) => {
            Py::new(py, (PyJSXAttrValueExprContainer::build(py, c)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        swc_core::ecma::ast::JSXAttrValue::JSXElement(e) => {
            Py::new(py, (PyJSXAttrValueElement::build(py, e)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        swc_core::ecma::ast::JSXAttrValue::JSXFragment(f) => {
            Py::new(py, (PyJSXAttrValueFragment::build(py, f)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
    })
}

pub fn conv_option_jsx_attr_value(
    py: Python<'_>,
    node: Option<swc_core::ecma::ast::JSXAttrValue>,
) -> PyResult<Option<Py<PyJSXAttrValue>>> {
    match node {
        None => Ok(None),
        Some(v) => Ok(Some(conv_jsx_attr_value(py, v)?)),
    }
}

#[pyclass]
pub struct PyJSXAttr {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub name: PyJSXAttrName,
    #[pyo3(get)]
    pub value: Option<Py<PyJSXAttrValue>>,
}

impl PyJSXAttr {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXAttr) -> PyResult<Self> {
        Ok(PyJSXAttr {
            span: conv_span(py, node.span)?,
            name: crate::conversions::conv_jsx_attr_name(py, node.name)?,
            value: conv_option_jsx_attr_value(py, node.value)?,
        })
    }
}

#[pyclass]
pub struct PyJSXOpeningElement {
    #[pyo3(get)]
    pub name: Py<PyJSXElementName>,
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub attrs: Vec<Py<PyJSXAttrOrSpread>>,
    #[pyo3(get)]
    pub self_closing: bool,
    #[pyo3(get)]
    pub type_args: Option<Py<PyTsTypeParamInstantiation>>,
}

impl PyJSXOpeningElement {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXOpeningElement) -> PyResult<Self> {
        Ok(PyJSXOpeningElement {
            name: crate::conversions::conv_jsx_element_name(py, node.name)?,
            span: conv_span(py, node.span)?,
            attrs: conv_jsx_attr_or_spreads(py, node.attrs)?,
            self_closing: node.self_closing,
            type_args: conv_typeparams(py, node.type_args)?,
        })
    }
}

#[pyclass]
pub struct PyJSXClosingElement {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub name: Py<PyJSXElementName>,
}

impl PyJSXClosingElement {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXClosingElement) -> PyResult<Self> {
        Ok(PyJSXClosingElement {
            span: conv_span(py, node.span)?,
            name: crate::conversions::conv_jsx_element_name(py, node.name)?,
        })
    }
}

#[pyclass(subclass)]
pub struct PyJSXElementChild {}

#[pyclass(extends=PyJSXElementChild)]
pub struct PyJSXElementChildText {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub value: String,
    #[pyo3(get)]
    pub raw: String,
}

impl PyJSXElementChildText {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXText) -> PyResult<Self> {
        Ok(PyJSXElementChildText {
            span: conv_span(py, node.span)?,
            value: node.value.to_atom_lossy().to_string(),
            raw: node.raw.to_string(),
        })
    }
}

#[pyclass(extends=PyJSXElementChild)]
pub struct PyJSXElementChildExprContainer {
    #[pyo3(get)]
    pub container: Py<PyJSXExprContainer>,
}

impl PyJSXElementChildExprContainer {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXExprContainer) -> PyResult<Self> {
        Ok(PyJSXElementChildExprContainer {
            container: Py::new(py, PyJSXExprContainer::build(py, node)?)?,
        })
    }
}

#[pyclass(extends=PyJSXElementChild)]
pub struct PyJSXElementChildSpreadChild {
    #[pyo3(get)]
    pub spread_child: Py<PyJSXSpreadChild>,
}

impl PyJSXElementChildSpreadChild {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXSpreadChild) -> PyResult<Self> {
        Ok(PyJSXElementChildSpreadChild {
            spread_child: Py::new(py, PyJSXSpreadChild::build(py, node)?)?,
        })
    }
}

#[pyclass(extends=PyJSXElementChild)]
pub struct PyJSXElementChildElement {
    #[pyo3(get)]
    pub element: Py<PyJSXElement>,
}


#[pyclass(extends=PyJSXElementChild)]
pub struct PyJSXElementChildFragment {
    #[pyo3(get)]
    pub fragment: Py<PyJSXFragment>,
}


pub struct JSXElementData {
    pub span: Span,
    pub opening: JSXOpeningElement,
    pub children: Vec<Arc<JSXElementChildData>>,
    pub closing: Option<JSXClosingElement>,
}

pub struct JSXFragmentData {
    pub span: Span,
    pub opening: JSXOpeningFragment,
    pub children: Vec<Arc<JSXElementChildData>>,
    pub closing: swc_core::ecma::ast::JSXClosingFragment,
}

pub enum JSXElementChildData {
    Text(JSXText),
    ExprContainer(JSXExprContainer),
    SpreadChild(JSXSpreadChild),
    Element(Arc<JSXElementData>),
    Fragment(Arc<JSXFragmentData>),
}

pub fn lower_jsx_element(e: JSXElement) -> JSXElementData {
    JSXElementData {
        span: e.span,
        opening: e.opening,
        children: e
            .children
            .into_iter()
            .map(|c| Arc::new(lower_jsx_element_child(c)))
            .collect(),
        closing: e.closing,
    }
}

pub fn lower_jsx_fragment(f: JSXFragment) -> JSXFragmentData {
    JSXFragmentData {
        span: f.span,
        opening: f.opening,
        children: f
            .children
            .into_iter()
            .map(|c| Arc::new(lower_jsx_element_child(c)))
            .collect(),
        closing: f.closing,
    }
}

pub fn lower_jsx_element_child(c: JSXElementChild) -> JSXElementChildData {
    match c {
        JSXElementChild::JSXText(t) => JSXElementChildData::Text(t),
        JSXElementChild::JSXExprContainer(e) => JSXElementChildData::ExprContainer(e),
        JSXElementChild::JSXSpreadChild(s) => JSXElementChildData::SpreadChild(s),
        JSXElementChild::JSXElement(e) => {
            JSXElementChildData::Element(Arc::new(lower_jsx_element(*e)))
        }
        JSXElementChild::JSXFragment(f) => {
            JSXElementChildData::Fragment(Arc::new(lower_jsx_fragment(f)))
        }
    }
}

pub fn wrap_jsx_element_data(py: Python<'_>, data: Arc<JSXElementData>) -> PyResult<Py<PyJSXElement>> {
    Py::new(py, (PyJSXElement::from_arc(data), PyExpr {}))
}

pub fn wrap_jsx_fragment_data(
    py: Python<'_>,
    data: Arc<JSXFragmentData>,
) -> PyResult<Py<PyJSXFragment>> {
    Py::new(py, (PyJSXFragment::from_arc(data), PyExpr {}))
}

pub fn wrap_jsx_element_child_data(
    py: Python<'_>,
    data: Arc<JSXElementChildData>,
) -> PyResult<Py<PyJSXElementChild>> {
    let base = PyJSXElementChild {};
    Ok(match &*data {
        JSXElementChildData::Text(t) => Py::new(py, (PyJSXElementChildText::build(py, t.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        JSXElementChildData::ExprContainer(c) => Py::new(
            py,
            (PyJSXElementChildExprContainer::build(py, c.clone())?, base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        JSXElementChildData::SpreadChild(s) => Py::new(
            py,
            (PyJSXElementChildSpreadChild::build(py, s.clone())?, base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        JSXElementChildData::Element(e) => {
            let element = wrap_jsx_element_data(py, Arc::clone(e))?;
            Py::new(py, (PyJSXElementChildElement { element }, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        JSXElementChildData::Fragment(f) => {
            let fragment = wrap_jsx_fragment_data(py, Arc::clone(f))?;
            Py::new(py, (PyJSXElementChildFragment { fragment }, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
    })
}


#[pyclass(extends=PyExpr)]
pub struct PyJSXElement {
    inner: Arc<JSXElementData>,
}

impl PyJSXElement {
    pub fn from_arc(inner: Arc<JSXElementData>) -> Self {
        PyJSXElement { inner }
    }
}

#[pymethods]
impl PyJSXElement {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.inner.span)
    }

    #[getter]
    fn opening(&self, py: Python<'_>) -> PyResult<Py<PyJSXOpeningElement>> {
        crate::conversions::conv_jsx_opening_element(py, self.inner.opening.clone())
    }

    #[getter]
    fn children(&self, py: Python<'_>) -> PyResult<Vec<Py<PyJSXElementChild>>> {
        self.inner
            .children
            .iter()
            .map(|c| wrap_jsx_element_child_data(py, Arc::clone(c)))
            .collect()
    }

    #[getter]
    fn closing(&self, py: Python<'_>) -> PyResult<Option<Py<PyJSXClosingElement>>> {
        crate::conversions::conv_option_jsx_closing_element(py, self.inner.closing.clone())
    }
}

#[derive(Clone)]
#[pyclass]
pub struct PyJSXOpeningFragment {
    #[pyo3(get)]
    pub span: PySpan,
}

impl PyJSXOpeningFragment {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXOpeningFragment) -> PyResult<Self> {
        Ok(PyJSXOpeningFragment {
            span: conv_span(py, node.span)?,
        })
    }
}

#[derive(Clone)]
#[pyclass]
pub struct PyJSXClosingFragment {
    #[pyo3(get)]
    pub span: PySpan,
}

impl PyJSXClosingFragment {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::JSXClosingFragment) -> PyResult<Self> {
        Ok(PyJSXClosingFragment {
            span: conv_span(py, node.span)?,
        })
    }
}

#[pyclass(extends=PyExpr)]
pub struct PyJSXFragment {
    inner: Arc<JSXFragmentData>,
}

impl PyJSXFragment {
    pub fn from_arc(inner: Arc<JSXFragmentData>) -> Self {
        PyJSXFragment { inner }
    }
}

#[pymethods]
impl PyJSXFragment {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.inner.span)
    }

    #[getter]
    fn opening(&self, py: Python<'_>) -> PyResult<PyJSXOpeningFragment> {
        PyJSXOpeningFragment::build(py, self.inner.opening.clone())
    }

    #[getter]
    fn children(&self, py: Python<'_>) -> PyResult<Vec<Py<PyJSXElementChild>>> {
        self.inner
            .children
            .iter()
            .map(|c| wrap_jsx_element_child_data(py, Arc::clone(c)))
            .collect()
    }

    #[getter]
    fn closing(&self, py: Python<'_>) -> PyResult<PyJSXClosingFragment> {
        PyJSXClosingFragment::build(py, self.inner.closing.clone())
    }
}
