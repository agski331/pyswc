use pyo3::prelude::*;
use std::sync::Arc;
use swc_core::common::Span;
use swc_core::ecma::ast::{
    BigInt as SwcBigInt, ComputedPropName, Ident, IdentName, Number, Prop, PropName, PropOrSpread,
    SpreadElement, Str,
};

use crate::{
    conversions::{conv_atom, conv_bigint_value, conv_f64, conv_ident, conv_span, conv_wtf8atom},
    macros::{arc_variant_node, ast_node_variant},
    pyexpr::{ExprData, PyExpr, conv_arc_expr, lower_expr},
    pyfunction::{FunctionData, PyFunction, lower_function, wrap_function_data},
    pyident::{PyIdent, PyIdentName, PyPrivateName},
    pyspan::PySpan,
};


pub struct ComputedPropNameData {
    pub span: Span,
    pub expr: Arc<ExprData>,
}

pub enum PropNameData {
    Ident(IdentName),
    Str(Str),
    Num(Number),
    Computed(ComputedPropNameData),
    BigInt(SwcBigInt),
}

pub struct KeyValuePropData {
    pub key: Arc<PropNameData>,
    pub value: Arc<ExprData>,
}

pub struct AssignPropData {
    pub span: Span,
    pub key: Ident,
    pub value: Arc<ExprData>,
}

pub struct GetterPropData {
    pub span: Span,
    pub key: Arc<PropNameData>,
    pub function: Arc<FunctionData>,
}

pub struct SetterPropData {
    pub span: Span,
    pub key: Arc<PropNameData>,
    pub function: Arc<FunctionData>,
}

pub struct MethodPropData {
    pub key: Arc<PropNameData>,
    pub function: Arc<FunctionData>,
}

pub enum PropData {
    Shorthand(Ident),
    KeyValue(KeyValuePropData),
    Assign(AssignPropData),
    Getter(GetterPropData),
    Setter(SetterPropData),
    Method(MethodPropData),
}

pub struct SpreadElementData {
    pub dot3_token: Span,
    pub expr: Arc<ExprData>,
}

pub enum PropOrSpreadData {
    Spread(SpreadElementData),
    Prop(Arc<PropData>),
}

#[derive(Clone)]
pub struct ExprOrSpreadData {
    pub spread: Option<Span>,
    pub expr: Arc<ExprData>,
}

pub fn lower_propname(p: PropName) -> PropNameData {
    match p {
        PropName::Ident(i) => PropNameData::Ident(i),
        PropName::Str(s) => PropNameData::Str(s),
        PropName::Num(n) => PropNameData::Num(n),
        PropName::Computed(c) => PropNameData::Computed(ComputedPropNameData {
            span: c.span,
            expr: Arc::new(lower_expr(*c.expr)),
        }),
        PropName::BigInt(b) => PropNameData::BigInt(b),
    }
}

pub fn lower_computed_propname(c: ComputedPropName) -> ComputedPropNameData {
    ComputedPropNameData {
        span: c.span,
        expr: Arc::new(lower_expr(*c.expr)),
    }
}

pub fn lower_prop(p: Prop) -> PropData {
    match p {
        Prop::Shorthand(i) => PropData::Shorthand(i),
        Prop::KeyValue(kv) => PropData::KeyValue(KeyValuePropData {
            key: Arc::new(lower_propname(kv.key)),
            value: Arc::new(lower_expr(*kv.value)),
        }),
        Prop::Assign(a) => PropData::Assign(AssignPropData {
            span: a.span,
            key: a.key,
            value: Arc::new(lower_expr(*a.value)),
        }),
        Prop::Getter(g) => PropData::Getter(GetterPropData {
            span: g.span,
            key: Arc::new(lower_propname(g.key)),
            function: Arc::new(lower_function(*g.function)),
        }),
        Prop::Setter(s) => PropData::Setter(SetterPropData {
            span: s.span,
            key: Arc::new(lower_propname(s.key)),
            function: Arc::new(lower_function(*s.function)),
        }),
        Prop::Method(m) => PropData::Method(MethodPropData {
            key: Arc::new(lower_propname(m.key)),
            function: Arc::new(lower_function(*m.function)),
        }),
    }
}

pub fn lower_spread_element(s: SpreadElement) -> SpreadElementData {
    SpreadElementData {
        dot3_token: s.dot3_token,
        expr: Arc::new(lower_expr(*s.expr)),
    }
}

pub fn lower_prop_or_spread(p: PropOrSpread) -> PropOrSpreadData {
    match p {
        PropOrSpread::Spread(s) => PropOrSpreadData::Spread(lower_spread_element(s)),
        PropOrSpread::Prop(pr) => PropOrSpreadData::Prop(Arc::new(lower_prop(*pr))),
    }
}

pub fn lower_expr_or_spread(e: swc_core::ecma::ast::ExprOrSpread) -> ExprOrSpreadData {
    ExprOrSpreadData {
        spread: e.spread,
        expr: Arc::new(lower_expr(*e.expr)),
    }
}

pub fn wrap_propname_data(py: Python<'_>, data: Arc<PropNameData>) -> PyResult<Py<PyPropName>> {
    let base = PyPropName {};
    Ok(match &*data {
        PropNameData::Ident(n) => Py::new(py, (PyIdentPropName::build(py, n.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PropNameData::Str(n) => Py::new(py, (PyStrPropName::build(py, n.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PropNameData::Num(n) => Py::new(py, (PyNumPropName::build(py, n.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PropNameData::Computed(_) => {
            Py::new(py, (PyComputedPropName::from_arc(Arc::clone(&data)), base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        PropNameData::BigInt(n) => Py::new(py, (PyBigIntPropName::build(py, n.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
    })
}

pub fn wrap_computed_propname_data(
    py: Python<'_>,
    data: ComputedPropNameData,
) -> PyResult<Py<PyComputedPropName>> {
    Py::new(
        py,
        (
            PyComputedPropName::from_arc(Arc::new(PropNameData::Computed(data))),
            PyPropName {},
        ),
    )
}

pub fn wrap_prop_data(py: Python<'_>, data: Arc<PropData>) -> PyResult<Py<PyProp>> {
    let base = PyProp {};
    Ok(match &*data {
        PropData::Shorthand(i) => Py::new(py, (PyShorthandProp::build(py, i.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PropData::KeyValue(_) => Py::new(py, (PyKeyValueProp::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PropData::Assign(_) => Py::new(py, (PyAssignProp::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PropData::Getter(_) => Py::new(py, (PyGetterProp::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PropData::Setter(_) => Py::new(py, (PySetterProp::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        PropData::Method(_) => Py::new(py, (PyMethodProp::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
    })
}

pub fn wrap_prop_or_spread_data(
    py: Python<'_>,
    data: Arc<PropOrSpreadData>,
) -> PyResult<Py<PyPropOrSpread>> {
    let base = PyPropOrSpread {};
    Ok(match &*data {
        PropOrSpreadData::Spread(_) => {
            Py::new(py, (PySpreadElement::from_arc(Arc::clone(&data)), base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        PropOrSpreadData::Prop(p) => {
            let prop = wrap_prop_data(py, Arc::clone(p))?;
            Py::new(py, (PyPropOrSpreadProp { prop }, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
    })
}

pub fn conv_arc_propname(py: Python<'_>, data: Arc<PropNameData>) -> PyResult<Py<PyPropName>> {
    wrap_propname_data(py, data)
}

pub fn conv_arc_prop_or_spreads(
    py: Python<'_>,
    data: Vec<Arc<PropOrSpreadData>>,
) -> PyResult<Vec<Py<PyPropOrSpread>>> {
    data.into_iter().map(|d| wrap_prop_or_spread_data(py, d)).collect()
}

pub fn wrap_expr_or_spread_data(py: Python<'_>, data: ExprOrSpreadData) -> PyResult<Py<PyExprOrSpread>> {
    Py::new(py, PyExprOrSpread::from_data(data))
}

pub fn conv_arc_expr_or_spreads(
    py: Python<'_>,
    data: Vec<ExprOrSpreadData>,
) -> PyResult<Vec<Py<PyExprOrSpread>>> {
    data.into_iter().map(|d| wrap_expr_or_spread_data(py, d)).collect()
}

pub fn conv_option_arc_expr_or_spreads(
    py: Python<'_>,
    data: Option<Vec<ExprOrSpreadData>>,
) -> PyResult<Option<Vec<Py<PyExprOrSpread>>>> {
    data.map(|ds| conv_arc_expr_or_spreads(py, ds)).transpose()
}


#[pyclass(subclass)]
pub struct PyPropName {}

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

ast_node_variant!(PyPropName, PyBigIntPropName, SwcBigInt, {
    span: PySpan = conv_span,
    value: num_bigint::BigInt = conv_bigint_value
});

arc_variant_node!(PyPropName, PyComputedPropName, PropNameData, PropNameData::Computed, ComputedPropNameData, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_arc_expr,
});

#[pyclass]
pub struct PyExprOrSpread {
    data: ExprOrSpreadData,
}

impl PyExprOrSpread {
    pub fn from_data(data: ExprOrSpreadData) -> Self {
        PyExprOrSpread { data }
    }
}

#[pymethods]
impl PyExprOrSpread {
    #[getter]
    fn spread(&self, py: Python<'_>) -> PyResult<Option<PySpan>> {
        self.data.spread.map(|s| conv_span(py, s)).transpose()
    }

    #[getter]
    fn expr(&self, py: Python<'_>) -> PyResult<Py<PyExpr>> {
        conv_arc_expr(py, Arc::clone(&self.data.expr))
    }
}

#[pyclass(subclass)]
pub struct PyProp {}

impl PyShorthandProp {
    pub fn build(py: Python<'_>, ident: Ident) -> PyResult<Self> {
        Ok(PyShorthandProp {
            ident: conv_ident(py, ident)?,
        })
    }
}

#[pyclass(extends=PyProp)]
pub struct PyShorthandProp {
    #[pyo3(get)]
    pub ident: PyIdent,
}

arc_variant_node!(PyProp, PyKeyValueProp, PropData, PropData::KeyValue, KeyValuePropData, {
    key: Py<PyPropName> = conv_arc_propname,
    value: Py<PyExpr> = conv_arc_expr,
});

arc_variant_node!(PyProp, PyAssignProp, PropData, PropData::Assign, AssignPropData, {
    span: PySpan = conv_span,
    key: PyIdent = conv_ident,
    value: Py<PyExpr> = conv_arc_expr,
});

arc_variant_node!(PyProp, PyGetterProp, PropData, PropData::Getter, GetterPropData, {
    span: PySpan = conv_span,
    key: Py<PyPropName> = conv_arc_propname,
    function: Py<PyFunction> = wrap_function_data,
});

arc_variant_node!(PyProp, PySetterProp, PropData, PropData::Setter, SetterPropData, {
    span: PySpan = conv_span,
    key: Py<PyPropName> = conv_arc_propname,
    function: Py<PyFunction> = wrap_function_data,
});

arc_variant_node!(PyProp, PyMethodProp, PropData, PropData::Method, MethodPropData, {
    key: Py<PyPropName> = conv_arc_propname,
    function: Py<PyFunction> = wrap_function_data,
});

#[pyclass(subclass)]
pub struct PyPropOrSpread {}

#[pyclass(extends=PyPropOrSpread)]
pub struct PyPropOrSpreadProp {
    #[pyo3(get)]
    pub prop: Py<PyProp>,
}

arc_variant_node!(PyPropOrSpread, PySpreadElement, PropOrSpreadData, PropOrSpreadData::Spread, SpreadElementData, {
    dot3_token: PySpan = conv_span,
    expr: Py<PyExpr> = conv_arc_expr,
});

#[pyclass]
pub struct PyMemberProp {
    #[pyo3(get)]
    pub ident: Option<PyIdentName>,
    #[pyo3(get)]
    pub private_name: Option<PyPrivateName>,
    #[pyo3(get)]
    pub computed: Option<Py<PyComputedPropName>>,
}

#[pyclass]
pub struct PySuperProp {
    #[pyo3(get)]
    pub ident: Option<PyIdentName>,
    #[pyo3(get)]
    pub computed: Option<Py<PyComputedPropName>>,
}
