use pyo3::prelude::*;
use swc_core::ecma::ast::BigInt as SwcBigInt;
use swc_core::ecma::ast::*;

use crate::{
    conversions::{
        conv_bigint_value, conv_bool, conv_boxed_expr, conv_boxed_tstype, conv_boxed_tstypeann,
        conv_f64, conv_fn_param, conv_fn_params, conv_ident, conv_option_boxed_tstype,
        conv_option_entity_name, conv_option_import_call_options, conv_option_true_plus_minus,
        conv_option_tstypeann, conv_option_type_param_decl, conv_span, conv_str, conv_ts_lit,
        conv_ts_this_type_or_ident, conv_ts_type_elements, conv_ts_type_operator_op,
        conv_ts_type_query_expr, conv_tsentityname, conv_tskeywordtypekind, conv_tstypes,
        conv_tuple_elements, conv_type_param, conv_typeparams, conv_wtf8atom,
    },
    macros::ast_node_variant,
    pyexpr::PyExpr,
    pyident::{PyIdent, PyIdentName},
    pypat::PyPat,
    pyspan::PySpan,
};

#[pyclass]
pub struct PyTsTypeAnn {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub type_ann: Py<PyTsType>,
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

#[pyclass]
pub struct PyTsTypeParamInstantiation {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub params: Vec<Py<PyTsType>>,
}

#[pyclass]
pub struct PyTsTypeParamDecl {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub params: Vec<Py<PyTsTypeParam>>,
}

#[pyclass]
pub struct PyTsTypeParam {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub name: PyIdent,
    #[pyo3(get)]
    pub is_in: bool,
    #[pyo3(get)]
    pub is_out: bool,
    #[pyo3(get)]
    pub is_const: bool,
    #[pyo3(get)]
    pub constraint: Option<Py<PyTsType>>,
    #[pyo3(get)]
    pub default: Option<Py<PyTsType>>,
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

#[pyclass]
pub struct PyTplElement {
    #[pyo3(get)]
    pub span: PySpan,
    #[pyo3(get)]
    pub tail: bool,
    #[pyo3(get)]
    pub cooked: Option<String>,
    #[pyo3(get)]
    pub raw: String,
}

ast_node_variant!(PyTsType, PyTsKeywordType, TsKeywordType, {
    span: PySpan = conv_span,
    kind: u32 = conv_tskeywordtypekind
});

ast_node_variant!(PyTsType, PyTsThisType, TsThisType, {
    span: PySpan = conv_span
});

ast_node_variant!(PyTsType, PyTsTypeRef, TsTypeRef, {
    span: PySpan = conv_span,
    type_name: Py<PyTsEntityName> = conv_tsentityname,
    type_params: Option<Py<PyTsTypeParamInstantiation>> = conv_typeparams
});

ast_node_variant!(PyTsType, PyTsFnType, TsFnType, {
    span: PySpan = conv_span,
    params: Vec<Py<PyPat>> = conv_fn_params,
    type_params: Option<Py<PyTsTypeParamDecl>> = conv_option_type_param_decl,
    type_ann: Py<PyTsTypeAnn> = conv_boxed_tstypeann
});

ast_node_variant!(PyTsType, PyTsConstructorType, TsConstructorType, {
    span: PySpan = conv_span,
    params: Vec<Py<PyPat>> = conv_fn_params,
    type_params: Option<Py<PyTsTypeParamDecl>> = conv_option_type_param_decl,
    type_ann: Py<PyTsTypeAnn> = conv_boxed_tstypeann,
    is_abstract: bool = conv_bool
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

ast_node_variant!(PyTsType, PyTsArrayType, TsArrayType, {
    span: PySpan = conv_span,
    elem_type: Py<PyTsType> = conv_boxed_tstype
});

ast_node_variant!(PyTsType, PyTsTupleType, TsTupleType, {
    span: PySpan = conv_span,
    elem_types: Vec<Py<PyTsTupleElement>> = conv_tuple_elements
});

ast_node_variant!(PyTsType, PyTsOptionalType, TsOptionalType, {
    span: PySpan = conv_span,
    type_ann: Py<PyTsType> = conv_boxed_tstype
});

ast_node_variant!(PyTsType, PyTsRestType, TsRestType, {
    span: PySpan = conv_span,
    type_ann: Py<PyTsType> = conv_boxed_tstype
});

ast_node_variant!(PyTsType, PyTsUnionType, TsUnionType, {
    span: PySpan = conv_span,
    types: Vec<Py<PyTsType>> = conv_tstypes
});

ast_node_variant!(PyTsType, PyTsIntersectionType, TsIntersectionType, {
    span: PySpan = conv_span,
    types: Vec<Py<PyTsType>> = conv_tstypes
});

ast_node_variant!(PyTsType, PyTsConditionalType, TsConditionalType, {
    span: PySpan = conv_span,
    check_type: Py<PyTsType> = conv_boxed_tstype,
    extends_type: Py<PyTsType> = conv_boxed_tstype,
    true_type: Py<PyTsType> = conv_boxed_tstype,
    false_type: Py<PyTsType> = conv_boxed_tstype
});

ast_node_variant!(PyTsType, PyTsInferType, TsInferType, {
    span: PySpan = conv_span,
    type_param: Py<PyTsTypeParam> = conv_type_param
});

ast_node_variant!(PyTsType, PyTsParenthesizedType, TsParenthesizedType, {
    span: PySpan = conv_span,
    type_ann: Py<PyTsType> = conv_boxed_tstype
});

ast_node_variant!(PyTsType, PyTsTypeOperator, TsTypeOperator, {
    span: PySpan = conv_span,
    op: u32 = conv_ts_type_operator_op,
    type_ann: Py<PyTsType> = conv_boxed_tstype
});

ast_node_variant!(PyTsType, PyTsIndexedAccessType, TsIndexedAccessType, {
    span: PySpan = conv_span,
    readonly: bool = conv_bool,
    obj_type: Py<PyTsType> = conv_boxed_tstype,
    index_type: Py<PyTsType> = conv_boxed_tstype
});

ast_node_variant!(PyTsType, PyTsMappedType, TsMappedType, {
    span: PySpan = conv_span,
    readonly: Option<u32> = conv_option_true_plus_minus,
    type_param: Py<PyTsTypeParam> = conv_type_param,
    name_type: Option<Py<PyTsType>> = conv_option_boxed_tstype,
    optional: Option<u32> = conv_option_true_plus_minus,
    type_ann: Option<Py<PyTsType>> = conv_option_boxed_tstype
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

ast_node_variant!(PyTsType, PyTsTypePredicate, TsTypePredicate, {
    span: PySpan = conv_span,
    asserts: bool = conv_bool,
    param_name: Py<PyTsThisTypeOrIdent> = conv_ts_this_type_or_ident,
    type_ann: Option<Py<PyTsTypeAnn>> = conv_option_tstypeann
});

pub fn tstype_to_py(py: Python<'_>, ts_type: TsType) -> PyResult<Py<PyTsType>> {
    crate::conversions::conv_tstype(py, ts_type)
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
