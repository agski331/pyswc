use pyo3::prelude::*;
use std::sync::Arc;
use swc_core::common::Span;
use swc_core::ecma::ast::BigInt as SwcBigInt;
use swc_core::ecma::ast::*;

use crate::{
    conversions::{
        conv_atom, conv_bigint_value, conv_bool, conv_boxed_expr, conv_f64, conv_fn_param,
        conv_fn_params, conv_ident, conv_option_entity_name, conv_option_import_call_options,
        conv_option_true_plus_minus, conv_option_tstypeann, conv_option_type_param_decl,
        conv_option_wtf8atom, conv_span, conv_str, conv_ts_lit,
        conv_ts_this_type_or_ident, conv_ts_type_elements, conv_ts_type_operator_op,
        conv_ts_type_query_expr, conv_tsentityname, conv_tskeywordtypekind,
        conv_tstypes, conv_tuple_elements, conv_typeparams, conv_wtf8atom,
    },
    macros::{arc_variant_node, ast_node_variant, lazy_leaf_node},
    pyenums::{PyTruePlusMinus, PyTsKeywordTypeKind, PyTsTypeOperatorOp},
    pyexpr::PyExpr,
    pyident::{PyIdent, PyIdentName},
    pypat::PyPat,
    pyspan::PySpan,
};

pub struct TsTypeAnnData {
    pub span: swc_core::common::Span,
    pub type_ann: std::sync::Arc<TsTypeData>,
}

pub enum TsTypeData {
    KeywordType(TsKeywordType),
    ThisType(TsThisType),
    FnType(TsFnOrConstructorTypeData),
    ConstructorType(TsFnOrConstructorTypeData),
    TypeRef(TsTypeRef),
    TypeQuery(TsTypeQuery),
    TypeLit(TsTypeLit),
    ArrayType(TsArrayTypeData),
    TupleType(TsTupleType),
    OptionalType(TsOptionalTypeData),
    RestType(TsRestTypeData),
    UnionType(TsUnionOrIntersectionTypeData),
    IntersectionType(TsUnionOrIntersectionTypeData),
    ConditionalType(TsConditionalTypeData),
    InferType(TsInferType),
    ParenthesizedType(TsParenthesizedTypeData),
    TypeOperator(TsTypeOperatorData),
    IndexedAccessType(TsIndexedAccessTypeData),
    MappedType(TsMappedTypeData),
    LitType(TsLitType),
    TypePredicate(TsTypePredicateData),
    ImportType(TsImportType),
}

pub struct TsFnOrConstructorTypeData {
    pub span: swc_core::common::Span,
    pub params: Vec<TsFnParam>,
    pub type_params: Option<Arc<TsTypeParamDeclData>>,
    pub type_ann: std::sync::Arc<TsTypeAnnData>,
    pub is_abstract: bool,
}

pub struct TsArrayTypeData {
    pub span: swc_core::common::Span,
    pub elem_type: std::sync::Arc<TsTypeData>,
}

pub struct TsOptionalTypeData {
    pub span: swc_core::common::Span,
    pub type_ann: std::sync::Arc<TsTypeData>,
}

pub struct TsRestTypeData {
    pub span: swc_core::common::Span,
    pub type_ann: std::sync::Arc<TsTypeData>,
}

pub struct TsUnionOrIntersectionTypeData {
    pub span: swc_core::common::Span,
    pub types: Vec<std::sync::Arc<TsTypeData>>,
}

pub struct TsConditionalTypeData {
    pub span: swc_core::common::Span,
    pub check_type: std::sync::Arc<TsTypeData>,
    pub extends_type: std::sync::Arc<TsTypeData>,
    pub true_type: std::sync::Arc<TsTypeData>,
    pub false_type: std::sync::Arc<TsTypeData>,
}

pub struct TsParenthesizedTypeData {
    pub span: swc_core::common::Span,
    pub type_ann: std::sync::Arc<TsTypeData>,
}

pub struct TsTypeOperatorData {
    pub span: swc_core::common::Span,
    pub op: TsTypeOperatorOp,
    pub type_ann: std::sync::Arc<TsTypeData>,
}

pub struct TsIndexedAccessTypeData {
    pub span: swc_core::common::Span,
    pub readonly: bool,
    pub obj_type: std::sync::Arc<TsTypeData>,
    pub index_type: std::sync::Arc<TsTypeData>,
}

pub struct TsMappedTypeData {
    pub span: swc_core::common::Span,
    pub readonly: Option<TruePlusMinus>,
    pub type_param: TsTypeParamData,
    pub name_type: Option<std::sync::Arc<TsTypeData>>,
    pub optional: Option<TruePlusMinus>,
    pub type_ann: Option<std::sync::Arc<TsTypeData>>,
}

pub struct TsTypePredicateData {
    pub span: swc_core::common::Span,
    pub asserts: bool,
    pub param_name: TsThisTypeOrIdent,
    pub type_ann: Option<std::sync::Arc<TsTypeAnnData>>,
}

pub fn lower_tstypeann(ann: TsTypeAnn) -> TsTypeAnnData {
    TsTypeAnnData {
        span: ann.span,
        type_ann: std::sync::Arc::new(lower_tstype(*ann.type_ann)),
    }
}

pub fn lower_tstype(ts_type: TsType) -> TsTypeData {
    use std::sync::Arc;
    match ts_type {
        TsType::TsKeywordType(t) => TsTypeData::KeywordType(t),
        TsType::TsThisType(t) => TsTypeData::ThisType(t),
        TsType::TsFnOrConstructorType(TsFnOrConstructorType::TsFnType(t)) => {
            TsTypeData::FnType(TsFnOrConstructorTypeData {
                span: t.span,
                params: t.params,
                type_params: t.type_params.map(|tp| Arc::new(lower_ts_type_param_decl(*tp))),
                type_ann: Arc::new(lower_tstypeann(*t.type_ann)),
                is_abstract: false,
            })
        }
        TsType::TsFnOrConstructorType(TsFnOrConstructorType::TsConstructorType(t)) => {
            TsTypeData::ConstructorType(TsFnOrConstructorTypeData {
                span: t.span,
                params: t.params,
                type_params: t.type_params.map(|tp| Arc::new(lower_ts_type_param_decl(*tp))),
                type_ann: Arc::new(lower_tstypeann(*t.type_ann)),
                is_abstract: t.is_abstract,
            })
        }
        TsType::TsTypeRef(t) => TsTypeData::TypeRef(t),
        TsType::TsTypeQuery(t) => TsTypeData::TypeQuery(t),
        TsType::TsTypeLit(t) => TsTypeData::TypeLit(t),
        TsType::TsArrayType(t) => TsTypeData::ArrayType(TsArrayTypeData {
            span: t.span,
            elem_type: Arc::new(lower_tstype(*t.elem_type)),
        }),
        TsType::TsTupleType(t) => TsTypeData::TupleType(t),
        TsType::TsOptionalType(t) => TsTypeData::OptionalType(TsOptionalTypeData {
            span: t.span,
            type_ann: Arc::new(lower_tstype(*t.type_ann)),
        }),
        TsType::TsRestType(t) => TsTypeData::RestType(TsRestTypeData {
            span: t.span,
            type_ann: Arc::new(lower_tstype(*t.type_ann)),
        }),
        TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(t)) => {
            TsTypeData::UnionType(TsUnionOrIntersectionTypeData {
                span: t.span,
                types: t.types.into_iter().map(|x| Arc::new(lower_tstype(*x))).collect(),
            })
        }
        TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsIntersectionType(t)) => {
            TsTypeData::IntersectionType(TsUnionOrIntersectionTypeData {
                span: t.span,
                types: t.types.into_iter().map(|x| Arc::new(lower_tstype(*x))).collect(),
            })
        }
        TsType::TsConditionalType(t) => TsTypeData::ConditionalType(TsConditionalTypeData {
            span: t.span,
            check_type: Arc::new(lower_tstype(*t.check_type)),
            extends_type: Arc::new(lower_tstype(*t.extends_type)),
            true_type: Arc::new(lower_tstype(*t.true_type)),
            false_type: Arc::new(lower_tstype(*t.false_type)),
        }),
        TsType::TsInferType(t) => TsTypeData::InferType(t),
        TsType::TsParenthesizedType(t) => {
            TsTypeData::ParenthesizedType(TsParenthesizedTypeData {
                span: t.span,
                type_ann: Arc::new(lower_tstype(*t.type_ann)),
            })
        }
        TsType::TsTypeOperator(t) => TsTypeData::TypeOperator(TsTypeOperatorData {
            span: t.span,
            op: t.op,
            type_ann: Arc::new(lower_tstype(*t.type_ann)),
        }),
        TsType::TsIndexedAccessType(t) => {
            TsTypeData::IndexedAccessType(TsIndexedAccessTypeData {
                span: t.span,
                readonly: t.readonly,
                obj_type: Arc::new(lower_tstype(*t.obj_type)),
                index_type: Arc::new(lower_tstype(*t.index_type)),
            })
        }
        TsType::TsMappedType(t) => TsTypeData::MappedType(TsMappedTypeData {
            span: t.span,
            readonly: t.readonly,
            type_param: lower_ts_type_param(t.type_param),
            name_type: t.name_type.map(|x| Arc::new(lower_tstype(*x))),
            optional: t.optional,
            type_ann: t.type_ann.map(|x| Arc::new(lower_tstype(*x))),
        }),
        TsType::TsLitType(t) => TsTypeData::LitType(t),
        TsType::TsTypePredicate(t) => TsTypeData::TypePredicate(TsTypePredicateData {
            span: t.span,
            asserts: t.asserts,
            param_name: t.param_name,
            type_ann: t.type_ann.map(|a| Arc::new(lower_tstypeann(*a))),
        }),
        TsType::TsImportType(t) => TsTypeData::ImportType(t),
    }
}

pub fn wrap_tstypeann(py: Python<'_>, data: std::sync::Arc<TsTypeAnnData>) -> PyResult<Py<PyTsTypeAnn>> {
    Py::new(py, PyTsTypeAnn { inner: data })
}

pub fn wrap_tstype_data(py: Python<'_>, data: std::sync::Arc<TsTypeData>) -> PyResult<Py<PyTsType>> {
    let base = PyTsType {};
    Ok(match &*data {
        TsTypeData::KeywordType(t) => Py::new(py, (PyTsKeywordType::build(py, t.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::ThisType(t) => Py::new(py, (PyTsThisType::build(py, t.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::FnType(_) => Py::new(py, (PyTsFnType::from_arc(std::sync::Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::ConstructorType(_) => Py::new(
            py,
            (PyTsConstructorType::from_arc(std::sync::Arc::clone(&data)), base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        TsTypeData::TypeRef(t) => Py::new(py, (PyTsTypeRef::build(py, t.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::TypeQuery(t) => Py::new(py, (PyTsTypeQuery::build(py, t.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::TypeLit(t) => Py::new(py, (PyTsTypeLit::build(py, t.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::ArrayType(_) => Py::new(py, (PyTsArrayType::from_arc(std::sync::Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::TupleType(t) => Py::new(py, (PyTsTupleType::build(py, t.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::OptionalType(_) => Py::new(
            py,
            (PyTsOptionalType::from_arc(std::sync::Arc::clone(&data)), base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        TsTypeData::RestType(_) => Py::new(py, (PyTsRestType::from_arc(std::sync::Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::UnionType(_) => Py::new(py, (PyTsUnionType::from_arc(std::sync::Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::IntersectionType(_) => Py::new(
            py,
            (PyTsIntersectionType::from_arc(std::sync::Arc::clone(&data)), base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        TsTypeData::ConditionalType(_) => Py::new(
            py,
            (PyTsConditionalType::from_arc(std::sync::Arc::clone(&data)), base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        TsTypeData::InferType(t) => Py::new(py, (PyTsInferType::build(py, t.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::ParenthesizedType(_) => Py::new(
            py,
            (PyTsParenthesizedType::from_arc(std::sync::Arc::clone(&data)), base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        TsTypeData::TypeOperator(_) => Py::new(
            py,
            (PyTsTypeOperator::from_arc(std::sync::Arc::clone(&data)), base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        TsTypeData::IndexedAccessType(_) => Py::new(
            py,
            (PyTsIndexedAccessType::from_arc(std::sync::Arc::clone(&data)), base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        TsTypeData::MappedType(_) => Py::new(py, (PyTsMappedType::from_arc(std::sync::Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::LitType(t) => Py::new(py, (PyTsLitType::build(py, t.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsTypeData::TypePredicate(_) => Py::new(
            py,
            (PyTsTypePredicate::from_arc(std::sync::Arc::clone(&data)), base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        TsTypeData::ImportType(t) => Py::new(py, (PyTsImportType::build(py, t.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
    })
}

pub fn conv_arc_tstype(py: Python<'_>, data: std::sync::Arc<TsTypeData>) -> PyResult<Py<PyTsType>> {
    wrap_tstype_data(py, data)
}

pub fn conv_option_arc_tstype(
    py: Python<'_>,
    data: Option<std::sync::Arc<TsTypeData>>,
) -> PyResult<Option<Py<PyTsType>>> {
    data.map(|d| wrap_tstype_data(py, d)).transpose()
}

pub fn conv_arc_tstypes(
    py: Python<'_>,
    data: Vec<std::sync::Arc<TsTypeData>>,
) -> PyResult<Vec<Py<PyTsType>>> {
    data.into_iter().map(|d| wrap_tstype_data(py, d)).collect()
}

pub fn conv_arc_tstypeann(
    py: Python<'_>,
    data: std::sync::Arc<TsTypeAnnData>,
) -> PyResult<Py<PyTsTypeAnn>> {
    wrap_tstypeann(py, data)
}

pub fn conv_option_arc_tstypeann(
    py: Python<'_>,
    data: Option<std::sync::Arc<TsTypeAnnData>>,
) -> PyResult<Option<Py<PyTsTypeAnn>>> {
    data.map(|d| wrap_tstypeann(py, d)).transpose()
}

#[pyclass]
pub struct PyTsTypeAnn {
    inner: std::sync::Arc<TsTypeAnnData>,
}

#[pymethods]
impl PyTsTypeAnn {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.inner.span)
    }

    #[getter]
    fn type_ann(&self, py: Python<'_>) -> PyResult<Py<PyTsType>> {
        conv_arc_tstype(py, std::sync::Arc::clone(&self.inner.type_ann))
    }
}

#[pyclass(subclass)]
pub struct PyTsType {}

#[pyclass]
pub struct PyTsQualifiedName {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub left: Py<PyTsEntityName>,
    #[pyo3(get)]
    pub right: PyIdentName,
}

#[pyclass]
pub struct PyTsEntityName {
    #[pyo3(get)]
    pub ident: Option<PyIdent>,
    #[pyo3(get)]
    pub qualified_name: Option<Py<PyTsQualifiedName>>,
}

#[derive(Clone)]
pub struct TsTypeParamData {
    pub span: Span,
    pub name: Ident,
    pub is_in: bool,
    pub is_out: bool,
    pub is_const: bool,
    pub constraint: Option<Arc<TsTypeData>>,
    pub default: Option<Arc<TsTypeData>>,
}

pub struct TsTypeParamDeclData {
    pub span: Span,
    pub params: Vec<TsTypeParamData>,
}

pub struct TsTypeParamInstantiationData {
    pub span: Span,
    pub params: Vec<Arc<TsTypeData>>,
}

pub fn lower_ts_type_param(p: TsTypeParam) -> TsTypeParamData {
    TsTypeParamData {
        span: p.span,
        name: p.name,
        is_in: p.is_in,
        is_out: p.is_out,
        is_const: p.is_const,
        constraint: p.constraint.map(|c| Arc::new(lower_tstype(*c))),
        default: p.default.map(|d| Arc::new(lower_tstype(*d))),
    }
}

pub fn lower_ts_type_param_decl(d: TsTypeParamDecl) -> TsTypeParamDeclData {
    TsTypeParamDeclData {
        span: d.span,
        params: d.params.into_iter().map(lower_ts_type_param).collect(),
    }
}

pub fn lower_ts_type_param_instantiation(
    i: TsTypeParamInstantiation,
) -> TsTypeParamInstantiationData {
    TsTypeParamInstantiationData {
        span: i.span,
        params: i
            .params
            .into_iter()
            .map(|t| Arc::new(lower_tstype(*t)))
            .collect(),
    }
}

#[pyclass]
pub struct PyTsTypeParamInstantiation {
    inner: Arc<TsTypeParamInstantiationData>,
}

impl PyTsTypeParamInstantiation {
    pub fn from_arc(inner: Arc<TsTypeParamInstantiationData>) -> Self {
        PyTsTypeParamInstantiation { inner }
    }
}

#[pymethods]
impl PyTsTypeParamInstantiation {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.inner.span)
    }

    #[getter]
    fn params(&self, py: Python<'_>) -> PyResult<Vec<Py<PyTsType>>> {
        conv_arc_tstypes(py, self.inner.params.clone())
    }
}

#[pyclass]
pub struct PyTsTypeParamDecl {
    inner: Arc<TsTypeParamDeclData>,
}

impl PyTsTypeParamDecl {
    pub fn from_arc(inner: Arc<TsTypeParamDeclData>) -> Self {
        PyTsTypeParamDecl { inner }
    }
}

#[pymethods]
impl PyTsTypeParamDecl {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.inner.span)
    }

    #[getter]
    fn params(&self, py: Python<'_>) -> PyResult<Vec<Py<PyTsTypeParam>>> {
        wrap_ts_type_params(py, self.inner.params.clone())
    }
}

#[pyclass]
pub struct PyTsTypeParam {
    data: TsTypeParamData,
}

impl PyTsTypeParam {
    pub fn from_data(data: TsTypeParamData) -> Self {
        PyTsTypeParam { data }
    }
}

#[pymethods]
impl PyTsTypeParam {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.data.span)
    }

    #[getter]
    fn name(&self, py: Python<'_>) -> PyResult<PyIdent> {
        conv_ident(py, self.data.name.clone())
    }

    #[getter]
    fn is_in(&self) -> bool {
        self.data.is_in
    }

    #[getter]
    fn is_out(&self) -> bool {
        self.data.is_out
    }

    #[getter]
    fn is_const(&self) -> bool {
        self.data.is_const
    }

    #[getter]
    fn constraint(&self, py: Python<'_>) -> PyResult<Option<Py<PyTsType>>> {
        conv_option_arc_tstype(py, self.data.constraint.clone())
    }

    #[getter]
    fn default(&self, py: Python<'_>) -> PyResult<Option<Py<PyTsType>>> {
        conv_option_arc_tstype(py, self.data.default.clone())
    }
}

pub fn wrap_ts_type_param(py: Python<'_>, data: TsTypeParamData) -> PyResult<Py<PyTsTypeParam>> {
    Py::new(py, PyTsTypeParam::from_data(data))
}

pub fn wrap_ts_type_params(
    py: Python<'_>,
    data: Vec<TsTypeParamData>,
) -> PyResult<Vec<Py<PyTsTypeParam>>> {
    data.into_iter().map(|d| wrap_ts_type_param(py, d)).collect()
}

pub fn wrap_ts_type_param_decl_data(
    py: Python<'_>,
    data: Arc<TsTypeParamDeclData>,
) -> PyResult<Py<PyTsTypeParamDecl>> {
    Py::new(py, PyTsTypeParamDecl::from_arc(data))
}

pub fn wrap_ts_type_param_instantiation_data(
    py: Python<'_>,
    data: Arc<TsTypeParamInstantiationData>,
) -> PyResult<Py<PyTsTypeParamInstantiation>> {
    Py::new(py, PyTsTypeParamInstantiation::from_arc(data))
}

pub fn conv_option_arc_ts_type_param_decl(
    py: Python<'_>,
    data: Option<Arc<TsTypeParamDeclData>>,
) -> PyResult<Option<Py<PyTsTypeParamDecl>>> {
    data.map(|d| wrap_ts_type_param_decl_data(py, d)).transpose()
}

pub fn conv_arc_ts_type_param_instantiation(
    py: Python<'_>,
    data: Arc<TsTypeParamInstantiationData>,
) -> PyResult<Py<PyTsTypeParamInstantiation>> {
    wrap_ts_type_param_instantiation_data(py, data)
}

pub fn conv_option_arc_ts_type_param_instantiation(
    py: Python<'_>,
    data: Option<Arc<TsTypeParamInstantiationData>>,
) -> PyResult<Option<Py<PyTsTypeParamInstantiation>>> {
    data.map(|d| wrap_ts_type_param_instantiation_data(py, d))
        .transpose()
}

pub fn conv_ts_type_param(py: Python<'_>, param: TsTypeParam) -> PyResult<Py<PyTsTypeParam>> {
    wrap_ts_type_param(py, lower_ts_type_param(param))
}

#[pyclass]
pub struct PyTsImportCallOptions {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub with: Py<PyExpr>,
}

#[pyclass]
pub struct PyTsTupleElement {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub label: Option<Py<PyPat>>,
    #[pyo3(get)]
    pub ty: Py<PyTsType>,
}

lazy_leaf_node!(PyTplElement, TplElement, {
    span: PySpan = conv_span,
    tail: bool = conv_bool,
    cooked: Option<String> = conv_option_wtf8atom,
    raw: String = conv_atom,
});

ast_node_variant!(PyTsType, PyTsKeywordType, TsKeywordType, {
    span: PySpan = conv_span,
    kind: PyTsKeywordTypeKind = conv_tskeywordtypekind
});

ast_node_variant!(PyTsType, PyTsThisType, TsThisType, {
    span: PySpan = conv_span
});

ast_node_variant!(PyTsType, PyTsTypeRef, TsTypeRef, {
    span: PySpan = conv_span,
    type_name: Py<PyTsEntityName> = conv_tsentityname,
    type_params: Option<Py<PyTsTypeParamInstantiation>> = conv_typeparams
});

arc_variant_node!(PyTsType, PyTsFnType, TsTypeData, TsTypeData::FnType, TsFnOrConstructorTypeData, {
    span: PySpan = conv_span,
    params: Vec<Py<PyPat>> = conv_fn_params,
    type_params: Option<Py<PyTsTypeParamDecl>> = conv_option_arc_ts_type_param_decl,
    type_ann: Py<PyTsTypeAnn> = conv_arc_tstypeann,
});

arc_variant_node!(PyTsType, PyTsConstructorType, TsTypeData, TsTypeData::ConstructorType, TsFnOrConstructorTypeData, {
    span: PySpan = conv_span,
    params: Vec<Py<PyPat>> = conv_fn_params,
    type_params: Option<Py<PyTsTypeParamDecl>> = conv_option_arc_ts_type_param_decl,
    type_ann: Py<PyTsTypeAnn> = conv_arc_tstypeann,
    is_abstract: bool = conv_bool,
});

#[pyclass(subclass)]
pub struct PyTsTypeQueryExpr {}

#[pyclass(extends=PyTsTypeQueryExpr)]
pub struct PyTsTypeQueryExprEntityName {
    #[pyo3(get)]
    pub entity_name: Py<PyTsEntityName>,
}

impl PyTsTypeQueryExprEntityName {
    pub fn build(py: Python<'_>, name: TsEntityName) -> PyResult<Self> {
        Ok(PyTsTypeQueryExprEntityName {
            entity_name: conv_tsentityname(py, name)?,
        })
    }
}

#[pyclass(extends=PyTsTypeQueryExpr)]
pub struct PyTsTypeQueryExprImport {
    #[pyo3(get)]
    pub import: Py<PyTsImportType>,
}

impl PyTsTypeQueryExprImport {
    pub fn build(py: Python<'_>, node: TsImportType) -> PyResult<Self> {
        Ok(PyTsTypeQueryExprImport {
            import: crate::conversions::conv_ts_import_type(py, node)?,
        })
    }
}

ast_node_variant!(PyTsType, PyTsTypeQuery, TsTypeQuery, {
    span: PySpan = conv_span,
    expr_name: Py<PyTsTypeQueryExpr> = conv_ts_type_query_expr,
    type_args: Option<Py<PyTsTypeParamInstantiation>> = conv_typeparams
});

ast_node_variant!(PyTsType, PyTsImportType, TsImportType, {
    span: PySpan = conv_span,
    arg: String = conv_str,
    qualifier: Option<Py<PyTsEntityName>> = conv_option_entity_name,
    type_args: Option<Py<PyTsTypeParamInstantiation>> = conv_typeparams,
    attributes: Option<Py<PyTsImportCallOptions>> = conv_option_import_call_options
});

#[pyclass(subclass)]
pub struct PyTsTypeElement {}

ast_node_variant!(PyTsTypeElement, PyTsCallSignatureDecl, TsCallSignatureDecl, {
    span: PySpan = conv_span,
    params: Vec<Py<PyPat>> = conv_fn_params,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann,
    type_params: Option<Py<PyTsTypeParamDecl>> = conv_option_type_param_decl
});

ast_node_variant!(PyTsTypeElement, PyTsConstructSignatureDecl, TsConstructSignatureDecl, {
    span: PySpan = conv_span,
    params: Vec<Py<PyPat>> = conv_fn_params,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann,
    type_params: Option<Py<PyTsTypeParamDecl>> = conv_option_type_param_decl
});

ast_node_variant!(PyTsTypeElement, PyTsPropertySignature, TsPropertySignature, {
    span: PySpan = conv_span,
    readonly: bool = conv_bool,
    key: Py<PyExpr> = conv_boxed_expr,
    computed: bool = conv_bool,
    optional: bool = conv_bool,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann
});

ast_node_variant!(PyTsTypeElement, PyTsGetterSignature, TsGetterSignature, {
    span: PySpan = conv_span,
    key: Py<PyExpr> = conv_boxed_expr,
    computed: bool = conv_bool,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann
});

ast_node_variant!(PyTsTypeElement, PyTsSetterSignature, TsSetterSignature, {
    span: PySpan = conv_span,
    key: Py<PyExpr> = conv_boxed_expr,
    computed: bool = conv_bool,
    param: Py<PyPat> = conv_fn_param
});

ast_node_variant!(PyTsTypeElement, PyTsMethodSignature, TsMethodSignature, {
    span: PySpan = conv_span,
    key: Py<PyExpr> = conv_boxed_expr,
    computed: bool = conv_bool,
    optional: bool = conv_bool,
    params: Vec<Py<PyPat>> = conv_fn_params,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann,
    type_params: Option<Py<PyTsTypeParamDecl>> = conv_option_type_param_decl
});

ast_node_variant!(PyTsTypeElement, PyTsIndexSignature, TsIndexSignature, {
    params: Vec<Py<PyPat>> = conv_fn_params,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann,
    readonly: bool = conv_bool,
    is_static: bool = conv_bool,
    span: PySpan = conv_span
});

ast_node_variant!(PyTsType, PyTsTypeLit, TsTypeLit, {
    span: PySpan = conv_span,
    members: Vec<Py<PyTsTypeElement>> = conv_ts_type_elements
});

arc_variant_node!(PyTsType, PyTsArrayType, TsTypeData, TsTypeData::ArrayType, TsArrayTypeData, {
    span: PySpan = conv_span,
    elem_type: Py<PyTsType> = conv_arc_tstype,
});

ast_node_variant!(PyTsType, PyTsTupleType, TsTupleType, {
    span: PySpan = conv_span,
    elem_types: Vec<Py<PyTsTupleElement>> = conv_tuple_elements
});

arc_variant_node!(PyTsType, PyTsOptionalType, TsTypeData, TsTypeData::OptionalType, TsOptionalTypeData, {
    span: PySpan = conv_span,
    type_ann: Py<PyTsType> = conv_arc_tstype,
});

arc_variant_node!(PyTsType, PyTsRestType, TsTypeData, TsTypeData::RestType, TsRestTypeData, {
    span: PySpan = conv_span,
    type_ann: Py<PyTsType> = conv_arc_tstype,
});

arc_variant_node!(PyTsType, PyTsUnionType, TsTypeData, TsTypeData::UnionType, TsUnionOrIntersectionTypeData, {
    span: PySpan = conv_span,
    types: Vec<Py<PyTsType>> = conv_arc_tstypes,
});

arc_variant_node!(PyTsType, PyTsIntersectionType, TsTypeData, TsTypeData::IntersectionType, TsUnionOrIntersectionTypeData, {
    span: PySpan = conv_span,
    types: Vec<Py<PyTsType>> = conv_arc_tstypes,
});

arc_variant_node!(PyTsType, PyTsConditionalType, TsTypeData, TsTypeData::ConditionalType, TsConditionalTypeData, {
    span: PySpan = conv_span,
    check_type: Py<PyTsType> = conv_arc_tstype,
    extends_type: Py<PyTsType> = conv_arc_tstype,
    true_type: Py<PyTsType> = conv_arc_tstype,
    false_type: Py<PyTsType> = conv_arc_tstype,
});

ast_node_variant!(PyTsType, PyTsInferType, TsInferType, {
    span: PySpan = conv_span,
    type_param: Py<PyTsTypeParam> = conv_ts_type_param
});

arc_variant_node!(PyTsType, PyTsParenthesizedType, TsTypeData, TsTypeData::ParenthesizedType, TsParenthesizedTypeData, {
    span: PySpan = conv_span,
    type_ann: Py<PyTsType> = conv_arc_tstype,
});

arc_variant_node!(PyTsType, PyTsTypeOperator, TsTypeData, TsTypeData::TypeOperator, TsTypeOperatorData, {
    span: PySpan = conv_span,
    op: PyTsTypeOperatorOp = conv_ts_type_operator_op,
    type_ann: Py<PyTsType> = conv_arc_tstype,
});

arc_variant_node!(PyTsType, PyTsIndexedAccessType, TsTypeData, TsTypeData::IndexedAccessType, TsIndexedAccessTypeData, {
    span: PySpan = conv_span,
    readonly: bool = conv_bool,
    obj_type: Py<PyTsType> = conv_arc_tstype,
    index_type: Py<PyTsType> = conv_arc_tstype,
});

arc_variant_node!(PyTsType, PyTsMappedType, TsTypeData, TsTypeData::MappedType, TsMappedTypeData, {
    span: PySpan = conv_span,
    readonly: Option<PyTruePlusMinus> = conv_option_true_plus_minus,
    type_param: Py<PyTsTypeParam> = wrap_ts_type_param,
    name_type: Option<Py<PyTsType>> = conv_option_arc_tstype,
    optional: Option<PyTruePlusMinus> = conv_option_true_plus_minus,
    type_ann: Option<Py<PyTsType>> = conv_option_arc_tstype,
});

#[pyclass(subclass)]
pub struct PyTsLit {}

ast_node_variant!(PyTsLit, PyTsLitNumber, Number, {
    span: PySpan = conv_span,
    value: f64 = conv_f64
});

ast_node_variant!(PyTsLit, PyTsLitStr, Str, {
    span: PySpan = conv_span,
    value: String = conv_wtf8atom
});

ast_node_variant!(PyTsLit, PyTsLitBool, Bool, {
    span: PySpan = conv_span,
    value: bool = conv_bool
});

ast_node_variant!(PyTsLit, PyTsLitBigInt, SwcBigInt, {
    span: PySpan = conv_span,
    value: num_bigint::BigInt = conv_bigint_value
});

ast_node_variant!(PyTsLit, PyTsLitTpl, TsTplLitType, {
    span: PySpan = conv_span,
    types: Vec<Py<PyTsType>> = conv_tstypes,
    quasis: Vec<Py<PyTplElement>> = crate::conversions::conv_tpl_elements
});

ast_node_variant!(PyTsType, PyTsLitType, TsLitType, {
    span: PySpan = conv_span,
    lit: Py<PyTsLit> = conv_ts_lit
});

#[pyclass(subclass)]
pub struct PyTsThisTypeOrIdent {}

#[pyclass(extends=PyTsThisTypeOrIdent)]
pub struct PyTsThisTypeOrIdentThis {
    #[pyo3(get)]
    pub span: PySpan,
}

impl PyTsThisTypeOrIdentThis {
    pub fn build(py: Python<'_>, node: TsThisType) -> PyResult<Self> {
        Ok(PyTsThisTypeOrIdentThis {
            span: conv_span(py, node.span)?,
        })
    }
}

#[pyclass(extends=PyTsThisTypeOrIdent)]
pub struct PyTsThisTypeOrIdentIdent {
    #[pyo3(get)]
    pub ident: PyIdent,
}

impl PyTsThisTypeOrIdentIdent {
    pub fn build(py: Python<'_>, node: Ident) -> PyResult<Self> {
        Ok(PyTsThisTypeOrIdentIdent {
            ident: conv_ident(py, node)?,
        })
    }
}

arc_variant_node!(PyTsType, PyTsTypePredicate, TsTypeData, TsTypeData::TypePredicate, TsTypePredicateData, {
    span: PySpan = conv_span,
    asserts: bool = conv_bool,
    param_name: Py<PyTsThisTypeOrIdent> = conv_ts_this_type_or_ident,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_arc_tstypeann,
});

pub fn tstype_to_py(py: Python<'_>, ts_type: TsType) -> PyResult<Py<PyTsType>> {
    wrap_tstype_data(py, std::sync::Arc::new(lower_tstype(ts_type)))
}

#[pyclass]
pub struct PyTsExprWithTypeArgs {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub expr: Py<crate::pyexpr::PyExpr>,
    #[pyo3(get)]
    pub type_args: Option<Py<PyTsTypeParamInstantiation>>,
}
