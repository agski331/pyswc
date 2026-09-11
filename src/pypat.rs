use pyo3::prelude::*;
use std::sync::Arc;
use swc_core::common::Span;
use swc_core::ecma::ast::{BindingIdent, Invalid, ObjectPatProp, Pat, PropName};

use crate::{
    conversions::{conv_bindingident, conv_bool, conv_option_tstypeann, conv_propname, conv_span},
    macros::arc_variant_node,
    pyexpr::{ExprData, PyExpr, conv_arc_expr, conv_option_arc_expr, lower_expr},
    pyident::PyBindingIdent,
    pyprop::PyPropName,
    pyspan::PySpan,
    pytypeinfo::PyTsTypeAnn,
};

#[pyclass(subclass)]
pub struct PyPat {}

pub enum PatData {
    Ident(BindingIdent),
    Array(ArrayPatData),
    Rest(RestPatData),
    Object(ObjectPatData),
    Assign(AssignPatData),
    Invalid(Invalid),
    Expr(Arc<ExprData>),
}

pub struct ArrayPatData {
    pub span: Span,
    pub elems: Vec<Option<Arc<PatData>>>,
    pub optional: bool,
    pub type_ann: Option<Box<swc_core::ecma::ast::TsTypeAnn>>,
}

pub struct RestPatData {
    pub span: Span,
    pub dot3_token: Span,
    pub arg: Arc<PatData>,
    pub type_ann: Option<Box<swc_core::ecma::ast::TsTypeAnn>>,
}

pub struct ObjectPatData {
    pub span: Span,
    pub props: Vec<Arc<ObjectPatPropData>>,
    pub optional: bool,
    pub type_ann: Option<Box<swc_core::ecma::ast::TsTypeAnn>>,
}

#[derive(Clone)]
pub struct AssignPatData {
    pub span: Span,
    pub left: Arc<PatData>,
    pub right: Arc<ExprData>,
}

pub enum ObjectPatPropData {
    KeyValue(KeyValuePatPropData),
    Assign(AssignPatPropData),
    Rest(RestPatData),
}

pub struct KeyValuePatPropData {
    pub key: PropName,
    pub value: Arc<PatData>,
}

pub struct AssignPatPropData {
    pub span: Span,
    pub key: BindingIdent,
    pub value: Option<Arc<ExprData>>,
}


pub fn lower_pat(pat: Pat) -> PatData {
    match pat {
        Pat::Ident(p) => PatData::Ident(p),
        Pat::Array(p) => PatData::Array(ArrayPatData {
            span: p.span,
            elems: p
                .elems
                .into_iter()
                .map(|opt| opt.map(|el| Arc::new(lower_pat(el))))
                .collect(),
            optional: p.optional,
            type_ann: p.type_ann,
        }),
        Pat::Rest(p) => PatData::Rest(lower_rest_pat(p)),
        Pat::Object(p) => PatData::Object(ObjectPatData {
            span: p.span,
            props: p
                .props
                .into_iter()
                .map(|prop| Arc::new(lower_object_pat_prop(prop)))
                .collect(),
            optional: p.optional,
            type_ann: p.type_ann,
        }),
        Pat::Assign(p) => PatData::Assign(lower_assign_pat(p)),
        Pat::Invalid(p) => PatData::Invalid(p),
        Pat::Expr(e) => PatData::Expr(Arc::new(lower_expr(*e))),
    }
}

pub fn lower_assign_pat(p: swc_core::ecma::ast::AssignPat) -> AssignPatData {
    AssignPatData {
        span: p.span,
        left: Arc::new(lower_pat(*p.left)),
        right: Arc::new(lower_expr(*p.right)),
    }
}

fn lower_rest_pat(p: swc_core::ecma::ast::RestPat) -> RestPatData {
    RestPatData {
        span: p.span,
        dot3_token: p.dot3_token,
        arg: Arc::new(lower_pat(*p.arg)),
        type_ann: p.type_ann,
    }
}

pub fn lower_object_pat_prop(prop: ObjectPatProp) -> ObjectPatPropData {
    match prop {
        ObjectPatProp::KeyValue(p) => ObjectPatPropData::KeyValue(KeyValuePatPropData {
            key: p.key,
            value: Arc::new(lower_pat(*p.value)),
        }),
        ObjectPatProp::Assign(p) => ObjectPatPropData::Assign(AssignPatPropData {
            span: p.span,
            key: p.key,
            value: p.value.map(|v| Arc::new(lower_expr(*v))),
        }),
        ObjectPatProp::Rest(p) => ObjectPatPropData::Rest(lower_rest_pat(p)),
    }
}

pub fn wrap_pat_data(py: Python<'_>, data: Arc<PatData>) -> PyResult<Py<PyPat>> {
    let base = PyPat {};
    Ok(match &*data {
        PatData::Ident(p) => Py::new(py, (PyBindingIdentPat::build(py, p.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PatData::Array(_) => Py::new(py, (PyArrayPat::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PatData::Rest(_) => Py::new(py, (PyRestPat::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PatData::Object(_) => Py::new(py, (PyObjectPat::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PatData::Assign(_) => Py::new(py, (PyAssignPat::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PatData::Invalid(p) => Py::new(py, (PyInvalidPat::build(py, p.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PatData::Expr(e) => Py::new(py, (PyExprPat::from_arc(Arc::clone(e)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
    })
}

pub fn wrap_object_pat_prop_data(
    py: Python<'_>,
    data: Arc<ObjectPatPropData>,
) -> PyResult<Py<PyObjectPatProp>> {
    let base = PyObjectPatProp {};
    Ok(match &*data {
        ObjectPatPropData::KeyValue(_) => {
            Py::new(py, (PyKeyValuePatProp::from_arc(Arc::clone(&data)), base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        ObjectPatPropData::Assign(_) => {
            Py::new(py, (PyAssignPatProp::from_arc(Arc::clone(&data)), base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        ObjectPatPropData::Rest(_) => Py::new(
            py,
            (PyObjectPatRestProp::from_arc(Arc::clone(&data)), base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
    })
}

pub fn conv_arc_pat(py: Python<'_>, data: Arc<PatData>) -> PyResult<Py<PyPat>> {
    wrap_pat_data(py, data)
}

pub fn conv_arc_pats(py: Python<'_>, data: Vec<Arc<PatData>>) -> PyResult<Vec<Py<PyPat>>> {
    data.into_iter().map(|d| wrap_pat_data(py, d)).collect()
}

pub fn conv_arc_pat_elems(
    py: Python<'_>,
    data: Vec<Option<Arc<PatData>>>,
) -> PyResult<Vec<Option<Py<PyPat>>>> {
    data.into_iter()
        .map(|opt| opt.map(|d| wrap_pat_data(py, d)).transpose())
        .collect()
}

pub fn conv_arc_object_pat_props(
    py: Python<'_>,
    data: Vec<Arc<ObjectPatPropData>>,
) -> PyResult<Vec<Py<PyObjectPatProp>>> {
    data.into_iter()
        .map(|d| wrap_object_pat_prop_data(py, d))
        .collect()
}


#[pyclass(extends=PyPat)]
pub struct PyBindingIdentPat {
    #[pyo3(get)]
    pub ident: Py<PyBindingIdent>,
}

impl PyBindingIdentPat {
    pub fn build(py: Python<'_>, node: BindingIdent) -> PyResult<Self> {
        Ok(PyBindingIdentPat {
            ident: conv_bindingident(py, node)?,
        })
    }
}

arc_variant_node!(PyPat, PyArrayPat, PatData, PatData::Array, ArrayPatData, {
    span: PySpan = conv_span,
    elems: Vec<Option<Py<PyPat>>> = conv_arc_pat_elems,
    optional: bool = conv_bool,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann,
});

arc_variant_node!(PyPat, PyRestPat, PatData, PatData::Rest, RestPatData, {
    span: PySpan = conv_span,
    dot3_token: PySpan = conv_span,
    arg: Py<PyPat> = conv_arc_pat,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann,
});

arc_variant_node!(PyPat, PyObjectPat, PatData, PatData::Object, ObjectPatData, {
    span: PySpan = conv_span,
    props: Vec<Py<PyObjectPatProp>> = conv_arc_object_pat_props,
    optional: bool = conv_bool,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann,
});

arc_variant_node!(PyPat, PyAssignPat, PatData, PatData::Assign, AssignPatData, {
    span: PySpan = conv_span,
    left: Py<PyPat> = conv_arc_pat,
    right: Py<PyExpr> = conv_arc_expr,
});

#[pyclass(extends=PyPat)]
pub struct PyInvalidPat {
    #[pyo3(get)]
    pub span: PySpan,
}

impl PyInvalidPat {
    pub fn build(py: Python<'_>, node: Invalid) -> PyResult<Self> {
        Ok(PyInvalidPat {
            span: conv_span(py, node.span)?,
        })
    }
}

#[pyclass(extends=PyPat)]
pub struct PyExprPat {
    inner: Arc<ExprData>,
}

impl PyExprPat {
    pub fn from_arc(inner: Arc<ExprData>) -> Self {
        PyExprPat { inner }
    }
}

#[pymethods]
impl PyExprPat {
    #[getter]
    fn expr(&self, py: Python<'_>) -> PyResult<Py<PyExpr>> {
        conv_arc_expr(py, Arc::clone(&self.inner))
    }
}

pub fn pat_to_py(py: Python<'_>, pat: Pat) -> PyResult<Py<PyPat>> {
    wrap_pat_data(py, Arc::new(lower_pat(pat)))
}

#[pyclass(subclass)]
pub struct PyObjectPatProp {}

arc_variant_node!(PyObjectPatProp, PyKeyValuePatProp, ObjectPatPropData, ObjectPatPropData::KeyValue, KeyValuePatPropData, {
    key: Py<PyPropName> = conv_propname,
    value: Py<PyPat> = conv_arc_pat,
});

arc_variant_node!(PyObjectPatProp, PyAssignPatProp, ObjectPatPropData, ObjectPatPropData::Assign, AssignPatPropData, {
    span: PySpan = conv_span,
    key: Py<PyBindingIdent> = conv_bindingident,
    value: Option<Py<PyExpr>> = conv_option_arc_expr,
});

arc_variant_node!(PyObjectPatProp, PyObjectPatRestProp, ObjectPatPropData, ObjectPatPropData::Rest, RestPatData, {
    span: PySpan = conv_span,
    dot3_token: PySpan = conv_span,
    arg: Py<PyPat> = conv_arc_pat,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann,
});

pub fn object_pat_prop_to_py(py: Python<'_>, prop: ObjectPatProp) -> PyResult<Py<PyObjectPatProp>> {
    wrap_object_pat_prop_data(py, Arc::new(lower_object_pat_prop(prop)))
}
