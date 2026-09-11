use pyo3::prelude::*;
use std::sync::Arc;
use swc_core::common::Span;
use swc_core::ecma::ast::{Accessibility, EmptyStmt, Key, MethodKind, PrivateName};

use crate::{
    conversions::{conv_bool, conv_ctxt, conv_option_accessibility, conv_private_name, conv_span},
    macros::{arc_variant_node, ast_node_variant},
    pyenums::{PyAccessibility, PyMethodKind},
    pyexpr::{ExprData, PyExpr, conv_option_arc_expr, lower_expr},
    pyfunction::{
        DecoratorData, FunctionBodyData, FunctionData, ParamData, PyDecorator, PyFunction,
        PyFunctionBody, conv_option_function_body_arc, lower_decorator, lower_function,
        lower_function_body, lower_param, wrap_decorators, wrap_function_data,
    },
    pyident::PyPrivateName,
    pypat::{AssignPatData, PatData, PyPat, conv_arc_pats, lower_assign_pat},
    pyprop::{PropNameData, PyPropName, conv_arc_propname, lower_propname},
    pyspan::PySpan,
    pystmts::{BlockStmtData, lower_block_stmt, wrap_block_stmt_data},
    pytypeinfo::{PyTsTypeAnn, TsTypeAnnData, conv_option_arc_tstypeann, lower_tstypeann},
};

pub enum KeyData {
    Private(PrivateName),
    Public(Arc<PropNameData>),
}

pub fn lower_key(k: Key) -> KeyData {
    match k {
        Key::Private(p) => KeyData::Private(p),
        Key::Public(p) => KeyData::Public(Arc::new(lower_propname(p))),
    }
}

pub fn wrap_key_data(py: Python<'_>, data: &KeyData) -> PyResult<Py<PyKey>> {
    match data {
        KeyData::Private(p) => Py::new(
            py,
            PyKey {
                private: Some(conv_private_name(py, p.clone())?),
                public: None,
            },
        ),
        KeyData::Public(p) => Py::new(
            py,
            PyKey {
                private: None,
                public: Some(conv_arc_propname(py, Arc::clone(p))?),
            },
        ),
    }
}

#[derive(Clone)]
pub enum TsParamPropParamData {
    Ident(swc_core::ecma::ast::BindingIdent),
    Assign(AssignPatData),
}

#[derive(Clone)]
pub struct TsParamPropData {
    pub span: Span,
    pub decorators: Vec<DecoratorData>,
    pub accessibility: Option<Accessibility>,
    pub is_override: bool,
    pub readonly: bool,
    pub param: TsParamPropParamData,
}

#[derive(Clone)]
pub enum ParamOrTsParamPropData {
    Param(ParamData),
    TsParamProp(TsParamPropData),
}

pub struct ConstructorData {
    pub span: Span,
    pub ctxt: swc_core::common::SyntaxContext,
    pub key: Arc<PropNameData>,
    pub params: Vec<ParamOrTsParamPropData>,
    pub body: Option<Arc<FunctionBodyData>>,
    pub accessibility: Option<Accessibility>,
    pub is_optional: bool,
}

pub struct ClassMethodData {
    pub span: Span,
    pub key: Arc<PropNameData>,
    pub function: Arc<FunctionData>,
    pub kind: MethodKind,
    pub is_static: bool,
    pub accessibility: Option<Accessibility>,
    pub is_abstract: bool,
    pub is_optional: bool,
    pub is_override: bool,
}

pub struct PrivateMethodData {
    pub span: Span,
    pub key: PrivateName,
    pub function: Arc<FunctionData>,
    pub kind: MethodKind,
    pub is_static: bool,
    pub accessibility: Option<Accessibility>,
    pub is_abstract: bool,
    pub is_optional: bool,
    pub is_override: bool,
}

pub struct ClassPropData {
    pub span: Span,
    pub key: Arc<PropNameData>,
    pub value: Option<Arc<ExprData>>,
    pub type_ann: Option<Arc<TsTypeAnnData>>,
    pub is_static: bool,
    pub decorators: Vec<DecoratorData>,
    pub accessibility: Option<Accessibility>,
    pub is_abstract: bool,
    pub is_optional: bool,
    pub is_override: bool,
    pub readonly: bool,
    pub declare: bool,
    pub definite: bool,
}

pub struct PrivatePropData {
    pub span: Span,
    pub ctxt: swc_core::common::SyntaxContext,
    pub key: PrivateName,
    pub value: Option<Arc<ExprData>>,
    pub type_ann: Option<Arc<TsTypeAnnData>>,
    pub is_static: bool,
    pub decorators: Vec<DecoratorData>,
    pub accessibility: Option<Accessibility>,
    pub is_optional: bool,
    pub is_override: bool,
    pub readonly: bool,
    pub definite: bool,
}

pub struct ClassIndexSignatureData {
    pub params: Vec<Arc<PatData>>,
    pub type_ann: Option<Arc<TsTypeAnnData>>,
    pub readonly: bool,
    pub is_static: bool,
    pub span: Span,
}

pub struct StaticBlockData {
    pub span: Span,
    pub body: Arc<BlockStmtData>,
}

pub struct AutoAccessorData {
    pub span: Span,
    pub key: KeyData,
    pub value: Option<Arc<ExprData>>,
    pub type_ann: Option<Arc<TsTypeAnnData>>,
    pub is_static: bool,
    pub decorators: Vec<DecoratorData>,
    pub accessibility: Option<Accessibility>,
    pub is_abstract: bool,
    pub is_override: bool,
    pub definite: bool,
}

pub enum ClassMemberData {
    Constructor(ConstructorData),
    Method(ClassMethodData),
    PrivateMethod(PrivateMethodData),
    ClassProp(ClassPropData),
    PrivateProp(PrivatePropData),
    TsIndexSignature(ClassIndexSignatureData),
    Empty(EmptyStmt),
    StaticBlock(StaticBlockData),
    AutoAccessor(AutoAccessorData),
}

pub fn lower_param_or_ts_param_prop(
    p: swc_core::ecma::ast::ParamOrTsParamProp,
) -> ParamOrTsParamPropData {
    match p {
        swc_core::ecma::ast::ParamOrTsParamProp::Param(p) => {
            ParamOrTsParamPropData::Param(lower_param(p))
        }
        swc_core::ecma::ast::ParamOrTsParamProp::TsParamProp(p) => {
            ParamOrTsParamPropData::TsParamProp(TsParamPropData {
                span: p.span,
                decorators: p.decorators.into_iter().map(lower_decorator).collect(),
                accessibility: p.accessibility,
                is_override: p.is_override,
                readonly: p.readonly,
                param: match p.param {
                    swc_core::ecma::ast::TsParamPropParam::Ident(i) => {
                        TsParamPropParamData::Ident(i)
                    }
                    swc_core::ecma::ast::TsParamPropParam::Assign(a) => {
                        TsParamPropParamData::Assign(lower_assign_pat(a))
                    }
                },
            })
        }
    }
}

pub fn lower_class_member(m: swc_core::ecma::ast::ClassMember) -> ClassMemberData {
    use swc_core::ecma::ast::ClassMember;
    match m {
        ClassMember::Constructor(c) => ClassMemberData::Constructor(ConstructorData {
            span: c.span,
            ctxt: c.ctxt,
            key: Arc::new(lower_propname(c.key)),
            params: c.params.into_iter().map(lower_param_or_ts_param_prop).collect(),
            body: c.body.map(|b| Arc::new(lower_function_body(b))),
            accessibility: c.accessibility,
            is_optional: c.is_optional,
        }),
        ClassMember::Method(m) => ClassMemberData::Method(ClassMethodData {
            span: m.span,
            key: Arc::new(lower_propname(m.key)),
            function: Arc::new(lower_function(*m.function)),
            kind: m.kind,
            is_static: m.is_static,
            accessibility: m.accessibility,
            is_abstract: m.is_abstract,
            is_optional: m.is_optional,
            is_override: m.is_override,
        }),
        ClassMember::PrivateMethod(m) => ClassMemberData::PrivateMethod(PrivateMethodData {
            span: m.span,
            key: m.key,
            function: Arc::new(lower_function(*m.function)),
            kind: m.kind,
            is_static: m.is_static,
            accessibility: m.accessibility,
            is_abstract: m.is_abstract,
            is_optional: m.is_optional,
            is_override: m.is_override,
        }),
        ClassMember::ClassProp(p) => ClassMemberData::ClassProp(ClassPropData {
            span: p.span,
            key: Arc::new(lower_propname(p.key)),
            value: p.value.map(|v| Arc::new(lower_expr(*v))),
            type_ann: p.type_ann.map(|t| Arc::new(lower_tstypeann(*t))),
            is_static: p.is_static,
            decorators: p.decorators.into_iter().map(lower_decorator).collect(),
            accessibility: p.accessibility,
            is_abstract: p.is_abstract,
            is_optional: p.is_optional,
            is_override: p.is_override,
            readonly: p.readonly,
            declare: p.declare,
            definite: p.definite,
        }),
        ClassMember::PrivateProp(p) => ClassMemberData::PrivateProp(PrivatePropData {
            span: p.span,
            ctxt: p.ctxt,
            key: p.key,
            value: p.value.map(|v| Arc::new(lower_expr(*v))),
            type_ann: p.type_ann.map(|t| Arc::new(lower_tstypeann(*t))),
            is_static: p.is_static,
            decorators: p.decorators.into_iter().map(lower_decorator).collect(),
            accessibility: p.accessibility,
            is_optional: p.is_optional,
            is_override: p.is_override,
            readonly: p.readonly,
            definite: p.definite,
        }),
        ClassMember::TsIndexSignature(t) => {
            ClassMemberData::TsIndexSignature(ClassIndexSignatureData {
                params: t
                    .params
                    .into_iter()
                    .map(|p| {
                        let pat = match p {
                            swc_core::ecma::ast::TsFnParam::Ident(p) => swc_core::ecma::ast::Pat::Ident(p),
                            swc_core::ecma::ast::TsFnParam::Array(p) => swc_core::ecma::ast::Pat::Array(p),
                            swc_core::ecma::ast::TsFnParam::Rest(p) => swc_core::ecma::ast::Pat::Rest(p),
                            swc_core::ecma::ast::TsFnParam::Object(p) => swc_core::ecma::ast::Pat::Object(p),
                        };
                        Arc::new(crate::pypat::lower_pat(pat))
                    })
                    .collect(),
                type_ann: t.type_ann.map(|t| Arc::new(lower_tstypeann(*t))),
                readonly: t.readonly,
                is_static: t.is_static,
                span: t.span,
            })
        }
        ClassMember::Empty(e) => ClassMemberData::Empty(e),
        ClassMember::StaticBlock(s) => ClassMemberData::StaticBlock(StaticBlockData {
            span: s.span,
            body: Arc::new(lower_block_stmt(s.body)),
        }),
        ClassMember::AutoAccessor(a) => ClassMemberData::AutoAccessor(AutoAccessorData {
            span: a.span,
            key: lower_key(a.key),
            value: a.value.map(|v| Arc::new(lower_expr(*v))),
            type_ann: a.type_ann.map(|t| Arc::new(lower_tstypeann(*t))),
            is_static: a.is_static,
            decorators: a.decorators.into_iter().map(lower_decorator).collect(),
            accessibility: a.accessibility,
            is_abstract: a.is_abstract,
            is_override: a.is_override,
            definite: a.definite,
        }),
    }
}

pub fn wrap_param_or_ts_param_prop_data(
    py: Python<'_>,
    data: ParamOrTsParamPropData,
) -> PyResult<Py<PyParamOrTsParamProp>> {
    let base = PyParamOrTsParamProp {};
    Ok(match data {
        ParamOrTsParamPropData::Param(p) => {
            Py::new(py, (PyParamOrTsParamPropParam::from_data(p), base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        ParamOrTsParamPropData::TsParamProp(p) => Py::new(
            py,
            (PyParamOrTsParamPropTsParamProp::from_data(p), base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
    })
}

pub fn wrap_class_member_data(py: Python<'_>, data: Arc<ClassMemberData>) -> PyResult<Py<PyClassMember>> {
    let base = PyClassMember {};
    Ok(match &*data {
        ClassMemberData::Constructor(_) => {
            Py::new(py, (PyConstructor::from_arc(Arc::clone(&data)), base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        ClassMemberData::Method(_) => Py::new(py, (PyClassMethod::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ClassMemberData::PrivateMethod(_) => {
            Py::new(py, (PyPrivateMethod::from_arc(Arc::clone(&data)), base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        ClassMemberData::ClassProp(_) => Py::new(py, (PyClassProp::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ClassMemberData::PrivateProp(_) => {
            Py::new(py, (PyPrivateProp::from_arc(Arc::clone(&data)), base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        ClassMemberData::TsIndexSignature(_) => Py::new(
            py,
            (PyClassIndexSignature::from_arc(Arc::clone(&data)), base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        ClassMemberData::Empty(e) => Py::new(py, (PyClassEmptyMember::build(py, e.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ClassMemberData::StaticBlock(_) => Py::new(py, (PyStaticBlock::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ClassMemberData::AutoAccessor(_) => {
            Py::new(py, (PyAutoAccessor::from_arc(Arc::clone(&data)), base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
    })
}

pub fn conv_arc_class_members(
    py: Python<'_>,
    data: Vec<Arc<ClassMemberData>>,
) -> PyResult<Vec<Py<PyClassMember>>> {
    data.into_iter().map(|d| wrap_class_member_data(py, d)).collect()
}

#[pyclass]
pub struct PyKey {
    #[pyo3(get)]
    pub private: Option<PyPrivateName>,
    #[pyo3(get)]
    pub public: Option<Py<PyPropName>>,
}

#[pyclass(subclass)]
pub struct PyClassMember {}

#[pyclass(extends=PyClassMember)]
pub struct PyConstructor {
    inner: Arc<ClassMemberData>,
}

impl PyConstructor {
    pub fn from_arc(inner: Arc<ClassMemberData>) -> Self {
        PyConstructor { inner }
    }

    fn data(&self) -> &ConstructorData {
        match &*self.inner {
            ClassMemberData::Constructor(d) => d,
            _ => unreachable!("ClassMemberData/PyConstructor mismatch"),
        }
    }
}

#[pymethods]
impl PyConstructor {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.data().span)
    }

    #[getter]
    fn ctxt(&self, py: Python<'_>) -> PyResult<u32> {
        conv_ctxt(py, self.data().ctxt)
    }

    #[getter]
    fn key(&self, py: Python<'_>) -> PyResult<Py<PyPropName>> {
        conv_arc_propname(py, Arc::clone(&self.data().key))
    }

    #[getter]
    fn accessibility(&self, py: Python<'_>) -> PyResult<Option<PyAccessibility>> {
        conv_option_accessibility(py, self.data().accessibility)
    }

    #[getter]
    fn is_optional(&self) -> bool {
        self.data().is_optional
    }

    #[getter]
    fn params(&self, py: Python<'_>) -> PyResult<Vec<Py<PyParamOrTsParamProp>>> {
        self.data()
            .params
            .iter()
            .cloned()
            .map(|p| wrap_param_or_ts_param_prop_data(py, p))
            .collect()
    }

    #[getter]
    fn body(&self, py: Python<'_>) -> PyResult<Option<Py<PyFunctionBody>>> {
        conv_option_function_body_arc(py, self.data().body.clone())
    }
}

arc_variant_node!(PyClassMember, PyClassMethod, ClassMemberData, ClassMemberData::Method, ClassMethodData, {
    span: PySpan = conv_span,
    key: Py<PyPropName> = conv_arc_propname,
    function: Py<PyFunction> = wrap_function_data,
    kind: PyMethodKind = crate::conversions::conv_method_kind,
    is_static: bool = conv_bool,
    accessibility: Option<PyAccessibility> = conv_option_accessibility,
    is_abstract: bool = conv_bool,
    is_optional: bool = conv_bool,
    is_override: bool = conv_bool,
});

arc_variant_node!(PyClassMember, PyPrivateMethod, ClassMemberData, ClassMemberData::PrivateMethod, PrivateMethodData, {
    span: PySpan = conv_span,
    key: PyPrivateName = conv_private_name,
    function: Py<PyFunction> = wrap_function_data,
    kind: PyMethodKind = crate::conversions::conv_method_kind,
    is_static: bool = conv_bool,
    accessibility: Option<PyAccessibility> = conv_option_accessibility,
    is_abstract: bool = conv_bool,
    is_optional: bool = conv_bool,
    is_override: bool = conv_bool,
});

#[pyclass(extends=PyClassMember)]
pub struct PyClassProp {
    inner: Arc<ClassMemberData>,
}

impl PyClassProp {
    pub fn from_arc(inner: Arc<ClassMemberData>) -> Self {
        PyClassProp { inner }
    }

    fn data(&self) -> &ClassPropData {
        match &*self.inner {
            ClassMemberData::ClassProp(d) => d,
            _ => unreachable!("ClassMemberData/PyClassProp mismatch"),
        }
    }
}

#[pymethods]
impl PyClassProp {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.data().span)
    }

    #[getter]
    fn key(&self, py: Python<'_>) -> PyResult<Py<PyPropName>> {
        conv_arc_propname(py, Arc::clone(&self.data().key))
    }

    #[getter]
    fn value(&self, py: Python<'_>) -> PyResult<Option<Py<PyExpr>>> {
        conv_option_arc_expr(py, self.data().value.clone())
    }

    #[getter]
    fn type_ann(&self, py: Python<'_>) -> PyResult<Option<Py<PyTsTypeAnn>>> {
        conv_option_arc_tstypeann(py, self.data().type_ann.clone())
    }

    #[getter]
    fn is_static(&self) -> bool {
        self.data().is_static
    }

    #[getter]
    fn accessibility(&self, py: Python<'_>) -> PyResult<Option<PyAccessibility>> {
        conv_option_accessibility(py, self.data().accessibility)
    }

    #[getter]
    fn is_abstract(&self) -> bool {
        self.data().is_abstract
    }

    #[getter]
    fn is_optional(&self) -> bool {
        self.data().is_optional
    }

    #[getter]
    fn is_override(&self) -> bool {
        self.data().is_override
    }

    #[getter]
    fn readonly(&self) -> bool {
        self.data().readonly
    }

    #[getter]
    fn declare(&self) -> bool {
        self.data().declare
    }

    #[getter]
    fn definite(&self) -> bool {
        self.data().definite
    }

    #[getter]
    fn decorators(&self, py: Python<'_>) -> PyResult<Vec<Py<PyDecorator>>> {
        wrap_decorators(py, self.data().decorators.clone())
    }
}

#[pyclass(extends=PyClassMember)]
pub struct PyPrivateProp {
    inner: Arc<ClassMemberData>,
}

impl PyPrivateProp {
    pub fn from_arc(inner: Arc<ClassMemberData>) -> Self {
        PyPrivateProp { inner }
    }

    fn data(&self) -> &PrivatePropData {
        match &*self.inner {
            ClassMemberData::PrivateProp(d) => d,
            _ => unreachable!("ClassMemberData/PyPrivateProp mismatch"),
        }
    }
}

#[pymethods]
impl PyPrivateProp {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.data().span)
    }

    #[getter]
    fn ctxt(&self, py: Python<'_>) -> PyResult<u32> {
        conv_ctxt(py, self.data().ctxt)
    }

    #[getter]
    fn key(&self, py: Python<'_>) -> PyResult<PyPrivateName> {
        conv_private_name(py, self.data().key.clone())
    }

    #[getter]
    fn value(&self, py: Python<'_>) -> PyResult<Option<Py<PyExpr>>> {
        conv_option_arc_expr(py, self.data().value.clone())
    }

    #[getter]
    fn type_ann(&self, py: Python<'_>) -> PyResult<Option<Py<PyTsTypeAnn>>> {
        conv_option_arc_tstypeann(py, self.data().type_ann.clone())
    }

    #[getter]
    fn is_static(&self) -> bool {
        self.data().is_static
    }

    #[getter]
    fn accessibility(&self, py: Python<'_>) -> PyResult<Option<PyAccessibility>> {
        conv_option_accessibility(py, self.data().accessibility)
    }

    #[getter]
    fn is_optional(&self) -> bool {
        self.data().is_optional
    }

    #[getter]
    fn is_override(&self) -> bool {
        self.data().is_override
    }

    #[getter]
    fn readonly(&self) -> bool {
        self.data().readonly
    }

    #[getter]
    fn definite(&self) -> bool {
        self.data().definite
    }

    #[getter]
    fn decorators(&self, py: Python<'_>) -> PyResult<Vec<Py<PyDecorator>>> {
        wrap_decorators(py, self.data().decorators.clone())
    }
}

arc_variant_node!(PyClassMember, PyClassIndexSignature, ClassMemberData, ClassMemberData::TsIndexSignature, ClassIndexSignatureData, {
    params: Vec<Py<PyPat>> = conv_arc_pats,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_arc_tstypeann,
    readonly: bool = conv_bool,
    is_static: bool = conv_bool,
    span: PySpan = conv_span,
});

ast_node_variant!(PyClassMember, PyClassEmptyMember, EmptyStmt, {
    span: PySpan = conv_span
});

#[pyclass(extends=PyClassMember)]
pub struct PyStaticBlock {
    inner: Arc<ClassMemberData>,
}

impl PyStaticBlock {
    pub fn from_arc(inner: Arc<ClassMemberData>) -> Self {
        PyStaticBlock { inner }
    }

    fn data(&self) -> &StaticBlockData {
        match &*self.inner {
            ClassMemberData::StaticBlock(d) => d,
            _ => unreachable!("ClassMemberData/PyStaticBlock mismatch"),
        }
    }
}

#[pymethods]
impl PyStaticBlock {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.data().span)
    }

    #[getter]
    fn body(&self, py: Python<'_>) -> PyResult<Py<crate::pystmts::PyBlockStmt>> {
        wrap_block_stmt_data(py, Arc::clone(&self.data().body))
    }
}

#[pyclass(extends=PyClassMember)]
pub struct PyAutoAccessor {
    inner: Arc<ClassMemberData>,
}

impl PyAutoAccessor {
    pub fn from_arc(inner: Arc<ClassMemberData>) -> Self {
        PyAutoAccessor { inner }
    }

    fn data(&self) -> &AutoAccessorData {
        match &*self.inner {
            ClassMemberData::AutoAccessor(d) => d,
            _ => unreachable!("ClassMemberData/PyAutoAccessor mismatch"),
        }
    }
}

#[pymethods]
impl PyAutoAccessor {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.data().span)
    }

    #[getter]
    fn value(&self, py: Python<'_>) -> PyResult<Option<Py<PyExpr>>> {
        conv_option_arc_expr(py, self.data().value.clone())
    }

    #[getter]
    fn type_ann(&self, py: Python<'_>) -> PyResult<Option<Py<PyTsTypeAnn>>> {
        conv_option_arc_tstypeann(py, self.data().type_ann.clone())
    }

    #[getter]
    fn is_static(&self) -> bool {
        self.data().is_static
    }

    #[getter]
    fn accessibility(&self, py: Python<'_>) -> PyResult<Option<PyAccessibility>> {
        conv_option_accessibility(py, self.data().accessibility)
    }

    #[getter]
    fn is_abstract(&self) -> bool {
        self.data().is_abstract
    }

    #[getter]
    fn is_override(&self) -> bool {
        self.data().is_override
    }

    #[getter]
    fn definite(&self) -> bool {
        self.data().definite
    }

    #[getter]
    fn key(&self, py: Python<'_>) -> PyResult<Py<PyKey>> {
        wrap_key_data(py, &self.data().key)
    }

    #[getter]
    fn decorators(&self, py: Python<'_>) -> PyResult<Vec<Py<PyDecorator>>> {
        wrap_decorators(py, self.data().decorators.clone())
    }
}

#[pyclass(subclass)]
pub struct PyParamOrTsParamProp {}

#[pyclass(extends=PyParamOrTsParamProp)]
pub struct PyParamOrTsParamPropParam {
    data: ParamData,
}

impl PyParamOrTsParamPropParam {
    pub fn from_data(data: ParamData) -> Self {
        PyParamOrTsParamPropParam { data }
    }
}

#[pymethods]
impl PyParamOrTsParamPropParam {
    #[getter]
    fn param(&self, py: Python<'_>) -> PyResult<Py<PyParam>> {
        Py::new(py, PyParam::from_data(self.data.clone()))
    }
}

#[pyclass(extends=PyParamOrTsParamProp)]
pub struct PyParamOrTsParamPropTsParamProp {
    data: TsParamPropData,
}

impl PyParamOrTsParamPropTsParamProp {
    pub fn from_data(data: TsParamPropData) -> Self {
        PyParamOrTsParamPropTsParamProp { data }
    }
}

#[pymethods]
impl PyParamOrTsParamPropTsParamProp {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.data.span)
    }

    #[getter]
    fn decorators(&self, py: Python<'_>) -> PyResult<Vec<Py<PyDecorator>>> {
        wrap_decorators(py, self.data.decorators.clone())
    }

    #[getter]
    fn accessibility(&self, py: Python<'_>) -> PyResult<Option<PyAccessibility>> {
        conv_option_accessibility(py, self.data.accessibility)
    }

    #[getter]
    fn is_override(&self) -> bool {
        self.data.is_override
    }

    #[getter]
    fn readonly(&self) -> bool {
        self.data.readonly
    }

    #[getter]
    fn param_ident(&self, py: Python<'_>) -> PyResult<Option<Py<crate::pyident::PyBindingIdent>>> {
        match &self.data.param {
            TsParamPropParamData::Ident(i) => Py::new(
                py,
                crate::pyident::PyBindingIdent::new(i.id.clone(), {
                    match &i.type_ann {
                        Some(a) => Some(crate::pytypeinfo::wrap_tstypeann(
                            py,
                            Arc::new(lower_tstypeann((**a).clone())),
                        )?),
                        None => None,
                    }
                }),
            )
            .map(Some),
            TsParamPropParamData::Assign(_) => Ok(None),
        }
    }

    #[getter]
    fn param_assign(&self, py: Python<'_>) -> PyResult<Option<Py<crate::pypat::PyAssignPat>>> {
        match &self.data.param {
            TsParamPropParamData::Assign(a) => {
                let base = PyPat {};
                let sub = crate::pypat::PyAssignPat::from_arc(Arc::new(PatData::Assign(a.clone())));
                Py::new(py, (sub, base)).map(Some)
            }
            TsParamPropParamData::Ident(_) => Ok(None),
        }
    }
}

use crate::pyfunction::PyParam;

pub struct ClassData {
    pub span: Span,
    pub ctxt: swc_core::common::SyntaxContext,
    pub decorators: Vec<DecoratorData>,
    pub body: Vec<Arc<ClassMemberData>>,
    pub super_class: Option<Arc<ExprData>>,
    pub is_abstract: bool,
    pub type_params: Option<Arc<crate::pytypeinfo::TsTypeParamDeclData>>,
    pub super_type_params: Option<Arc<crate::pytypeinfo::TsTypeParamInstantiationData>>,
    pub implements: Vec<swc_core::ecma::ast::TsExprWithTypeArgs>,
}

pub fn lower_class(c: swc_core::ecma::ast::Class) -> ClassData {
    ClassData {
        span: c.span,
        ctxt: c.ctxt,
        decorators: c.decorators.into_iter().map(lower_decorator).collect(),
        body: c.body.into_iter().map(|m| Arc::new(lower_class_member(m))).collect(),
        super_class: c.super_class.map(|e| Arc::new(lower_expr(*e))),
        is_abstract: c.is_abstract,
        type_params: c
            .type_params
            .map(|tp| Arc::new(crate::pytypeinfo::lower_ts_type_param_decl(*tp))),
        super_type_params: c
            .super_type_params
            .map(|tp| Arc::new(crate::pytypeinfo::lower_ts_type_param_instantiation(*tp))),
        implements: c.implements,
    }
}

#[pyclass]
pub struct PyClass {
    inner: Arc<ClassData>,
}

impl PyClass {
    pub fn from_arc(inner: Arc<ClassData>) -> Self {
        PyClass { inner }
    }
}

#[pymethods]
impl PyClass {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.inner.span)
    }

    #[getter]
    fn ctxt(&self, py: Python<'_>) -> PyResult<u32> {
        conv_ctxt(py, self.inner.ctxt)
    }

    #[getter]
    fn decorators(&self, py: Python<'_>) -> PyResult<Vec<Py<PyDecorator>>> {
        wrap_decorators(py, self.inner.decorators.clone())
    }

    #[getter]
    fn body(&self, py: Python<'_>) -> PyResult<Vec<Py<PyClassMember>>> {
        conv_arc_class_members(py, self.inner.body.clone())
    }

    #[getter]
    fn super_class(&self, py: Python<'_>) -> PyResult<Option<Py<PyExpr>>> {
        conv_option_arc_expr(py, self.inner.super_class.clone())
    }

    #[getter]
    fn is_abstract(&self) -> bool {
        self.inner.is_abstract
    }

    #[getter]
    fn type_params(&self, py: Python<'_>) -> PyResult<Option<Py<crate::pytypeinfo::PyTsTypeParamDecl>>> {
        crate::pytypeinfo::conv_option_arc_ts_type_param_decl(py, self.inner.type_params.clone())
    }

    #[getter]
    fn super_type_params(
        &self,
        py: Python<'_>,
    ) -> PyResult<Option<Py<crate::pytypeinfo::PyTsTypeParamInstantiation>>> {
        crate::pytypeinfo::conv_option_arc_ts_type_param_instantiation(
            py,
            self.inner.super_type_params.clone(),
        )
    }

    #[getter]
    fn implements(&self, py: Python<'_>) -> PyResult<Vec<Py<crate::pytypeinfo::PyTsExprWithTypeArgs>>> {
        crate::conversions::conv_ts_expr_with_type_args_vec(py, self.inner.implements.clone())
    }
}

pub fn wrap_class_data(py: Python<'_>, data: Arc<ClassData>) -> PyResult<Py<PyClass>> {
    Py::new(py, PyClass::from_arc(data))
}

pub fn class_to_py(py: Python<'_>, class: swc_core::ecma::ast::Class) -> PyResult<Py<PyClass>> {
    wrap_class_data(py, Arc::new(lower_class(class)))
}
