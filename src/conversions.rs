use crate::pyexpr::expr_to_py;
use crate::pyfunction::{PyCallee, PyFunction};
use crate::pyident::PyIdent;
use crate::pyident::PyIdentName;
use crate::pyident::{PyBindingIdent, PyPrivateName};
use crate::pyspan::PySpan;
use crate::pytypeinfo::PyTsType;
use crate::pytypeinfo::PyTsTypeAnn;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use swc_core::ecma::visit::NodeRef::MemberProp;

use crate::pyenums::*;
use crate::pyclass::PyClass;
use crate::pydecl::*;
use crate::pyexpr::*;
use crate::pyjsx::*;
use crate::pymodule::*;
use crate::pypat::PyPat;
use crate::pyprop::{PyMemberProp, PyPropName, PySuperProp};
use crate::pystmts::{PyForHead, PyVarDeclOrExpr};
use crate::pytypeinfo::{
    PyTplElement, PyTsCallSignatureDecl, PyTsConstructSignatureDecl, PyTsEntityName,
    PyTsExprWithTypeArgs, PyTsGetterSignature, PyTsImportCallOptions, PyTsImportType,
    PyTsIndexSignature, PyTsLit, PyTsLitBigInt, PyTsLitBool, PyTsLitNumber, PyTsLitStr,
    PyTsLitTpl, PyTsMethodSignature, PyTsPropertySignature, PyTsQualifiedName,
    PyTsSetterSignature, PyTsThisTypeOrIdent, PyTsThisTypeOrIdentIdent, PyTsThisTypeOrIdentThis,
    PyTsTupleElement, PyTsTypeElement, PyTsTypeParam, PyTsTypeParamDecl,
    PyTsTypeParamInstantiation, PyTsTypeQueryExpr, PyTsTypeQueryExprEntityName,
    PyTsTypeQueryExprImport,
};
use swc_core::common::{Span, SyntaxContext};
use swc_core::ecma::ast::*;
use swc_core::ecma::atoms::{Atom, Wtf8Atom};

pub fn conv_member_prop(
    py: Python<'_>,
    expr: swc_core::ecma::ast::MemberProp,
) -> PyResult<Py<PyMemberProp>> {
    if let swc_core::ecma::ast::MemberProp::Ident(id) = expr {
        let idname = conv_identname(py, id)?;
        return Py::new(
            py,
            PyMemberProp {
                ident: Some(idname),
                private_name: None,
                computed: None,
            },
        );
    } else if let swc_core::ecma::ast::MemberProp::PrivateName(p) = expr {
        let privname = conv_private_name(py, p)?;
        return Py::new(
            py,
            PyMemberProp {
                ident: None,
                private_name: Some(privname),
                computed: None,
            },
        );
    } else {
        if let swc_core::ecma::ast::MemberProp::Computed(c) = expr {
            let prop_name = crate::pyprop::wrap_computed_propname_data(
                py,
                crate::pyprop::lower_computed_propname(c),
            )?;
            return Py::new(
                py,
                PyMemberProp {
                    ident: None,
                    private_name: None,
                    computed: Some(prop_name),
                },
            );
        }
        return Err(PyValueError::new_err("Weird MemberProp conversion"));
    }
}

pub fn conv_unary_op(_py: Python<'_>, op: UnaryOp) -> PyResult<PyUnaryOp> {
    Ok(op.into())
}

pub fn conv_private_name(_py: Python<'_>, name: PrivateName) -> PyResult<PyPrivateName> {
    Ok(PyPrivateName::from_owned(name))
}

pub fn conv_assign_op(_py: Python<'_>, op: AssignOp) -> PyResult<PyAssignOp> {
    Ok(op.into())
}

pub fn conv_update_op(_py: Python<'_>, op: UpdateOp) -> PyResult<PyUpdateOp> {
    Ok(op.into())
}

pub fn conv_binary_op(_py: Python<'_>, op: BinaryOp) -> PyResult<PyBinaryOp> {
    Ok(op.into())
}

pub fn conv_tsqualfiiedname(py: Python<'_>, name: TsQualifiedName) -> PyResult<PyTsQualifiedName> {
    return Ok(PyTsQualifiedName {
        span: conv_span(py, name.span)?,
        left: conv_tsentityname(py, name.left)?,
        right: conv_identname(py, name.right)?,
    });
}

pub fn conv_identname(_py: Python<'_>, name: IdentName) -> PyResult<PyIdentName> {
    return Ok(PyIdentName::from_owned(name));
}

pub fn conv_ident(_py: Python<'_>, ident: Ident) -> PyResult<PyIdent> {
    return Ok(PyIdent::from_owned(ident));
}

pub fn conv_option_ident(py: Python<'_>, ident: Option<Ident>) -> PyResult<Option<PyIdent>> {
    if ident.is_none() {
        return Ok(None);
    }

    let id = ident.unwrap();
    let pyid = conv_ident(py, id)?;
    return Ok(Some(pyid));
}

pub fn conv_tsentityname(py: Python<'_>, name: TsEntityName) -> PyResult<Py<PyTsEntityName>> {
    if let TsEntityName::TsQualifiedName(name) = name {
        return Py::new(
            py,
            PyTsEntityName {
                ident: None,
                qualified_name: Some(Py::new(py, conv_tsqualfiiedname(py, *name)?)?),
            },
        );
    } else {
        if !name.is_ident() {
            return Err(PyValueError::new_err("Cannot convert TsEntityName"));
        }
        return Py::new(
            py,
            PyTsEntityName {
                ident: Some(conv_ident(py, name.expect_ident())?),
                qualified_name: None,
            },
        );
    }
}

pub fn conv_tstypeann(py: Python<'_>, ann: TsTypeAnn) -> PyResult<Py<PyTsTypeAnn>> {
    crate::pytypeinfo::wrap_tstypeann(
        py,
        std::sync::Arc::new(crate::pytypeinfo::lower_tstypeann(ann)),
    )
}

pub fn conv_tstype(py: Python<'_>, ts_type: TsType) -> PyResult<Py<PyTsType>> {
    crate::pytypeinfo::tstype_to_py(py, ts_type)
}

pub fn conv_tstypes(py: Python<'_>, ts_types: Vec<Box<TsType>>) -> PyResult<Vec<Py<PyTsType>>> {
    ts_types
        .into_iter()
        .map(|eos| conv_tstype(py, *eos))
        .collect()
}

pub fn conv_typeparams(
    py: Python<'_>,
    type_params: Option<Box<TsTypeParamInstantiation>>,
) -> PyResult<Option<Py<PyTsTypeParamInstantiation>>> {
    type_params
        .map(|tp| {
            crate::pytypeinfo::wrap_ts_type_param_instantiation_data(
                py,
                std::sync::Arc::new(crate::pytypeinfo::lower_ts_type_param_instantiation(*tp)),
            )
        })
        .transpose()
}

pub fn conv_tskeywordtypekind(_py: Python<'_>, kind: TsKeywordTypeKind) -> PyResult<PyTsKeywordTypeKind> {
    Ok(kind.into())
}

pub fn conv_span(_py: Python<'_>, span: Span) -> PyResult<PySpan> {
    Ok(span.into())
}

pub fn conv_elems_noopt(py: Python<'_>, elems: Vec<ExprOrSpread>) -> PyResult<Vec<Py<PyExpr>>> {
    elems
        .into_iter()
        .map(|eos| expr_to_py(py, *eos.expr))
        .collect()
}

pub fn conv_ctxt(_py: Python<'_>, ctxt: SyntaxContext) -> PyResult<u32> {
    Ok(ctxt.as_u32())
}

pub fn conv_option_pat(py: Python<'_>, pat: Option<Pat>) -> PyResult<Option<Py<PyPat>>> {
    pat.map(|p| conv_pat(py, p)).transpose()
}

pub fn conv_pat(py: Python<'_>, pat: Pat) -> PyResult<Py<PyPat>> {
    crate::pypat::pat_to_py(py, pat)
}

pub fn conv_bindingident(py: Python<'_>, ident: BindingIdent) -> PyResult<Py<PyBindingIdent>> {
    let type_ann = conv_option_tstypeann(py, ident.type_ann)?;
    Py::new(py, PyBindingIdent::new(ident.id, type_ann))
}

pub fn conv_option_tstypeann(
    py: Python<'_>,
    ann: Option<Box<TsTypeAnn>>,
) -> PyResult<Option<Py<PyTsTypeAnn>>> {
    match ann {
        None => Ok(None),
        Some(a) => Ok(Some(conv_tstypeann(py, *a)?)),
    }
}

pub fn conv_bool(_py: Python<'_>, value: bool) -> PyResult<bool> {
    Ok(value)
}

pub fn conv_f64(_py: Python<'_>, value: f64) -> PyResult<f64> {
    Ok(value)
}

pub fn conv_atom(_py: Python<'_>, atom: Atom) -> PyResult<String> {
    Ok(atom.to_string())
}

pub fn conv_wtf8atom(_py: Python<'_>, atom: Wtf8Atom) -> PyResult<String> {
    Ok(atom.to_atom_lossy().to_string())
}

pub fn conv_option_wtf8atom(_py: Python<'_>, atom: Option<Wtf8Atom>) -> PyResult<Option<String>> {
    Ok(atom.map(|a| a.to_atom_lossy().to_string()))
}

pub fn conv_bigint_value(_py: Python<'_>, value: Box<BigIntValue>) -> PyResult<num_bigint::BigInt> {
    Ok((*value).clone())
}

pub fn conv_propname(py: Python<'_>, name: PropName) -> PyResult<Py<PyPropName>> {
    crate::pyprop::wrap_propname_data(py, std::sync::Arc::new(crate::pyprop::lower_propname(name)))
}

pub fn conv_boxed_expr(py: Python<'_>, expr: Box<Expr>) -> PyResult<Py<PyExpr>> {
    crate::pyexpr::expr_to_py(py, *expr)
}

pub fn conv_expr(py: Python<'_>, expr: Expr) -> PyResult<Py<PyExpr>> {
    crate::pyexpr::expr_to_py(py, expr)
}

pub fn conv_callee(py: Python<'_>, callee: Callee) -> PyResult<Py<PyCallee>> {
    if callee.is_super_() {
        return Py::new(
            py,
            PyCallee {
                span: Some(conv_span(py, callee.expect_super_().span)?),
                is_super: true,
                phase: None,
                expr: None,
            },
        );
    } else {
        if callee.is_expr() {
            return Py::new(
                py,
                PyCallee {
                    span: None,
                    is_super: false,
                    phase: None,
                    expr: Some(conv_boxed_expr(py, callee.expect_expr())?),
                },
            );
        } else {
            let import = callee.expect_import();
            return Py::new(
                py,
                PyCallee {
                    span: Some(conv_span(py, import.span)?),
                    is_super: false,
                    phase: Some(import.phase.into()),
                    expr: None,
                },
            );
        }
    }
}

pub fn conv_boxed_tstype(py: Python<'_>, ts_type: Box<TsType>) -> PyResult<Py<PyTsType>> {
    conv_tstype(py, *ts_type)
}

pub fn conv_str(_py: Python<'_>, s: Str) -> PyResult<String> {
    Ok(s.value.to_atom_lossy().to_string())
}

pub fn conv_boxed_object_lit(py: Python<'_>, lit: Box<ObjectLit>) -> PyResult<Py<PyExpr>> {
    conv_expr(py, Expr::Object(*lit))
}

pub fn conv_option_entity_name(
    py: Python<'_>,
    name: Option<TsEntityName>,
) -> PyResult<Option<Py<PyTsEntityName>>> {
    match name {
        None => Ok(None),
        Some(n) => Ok(Some(conv_tsentityname(py, n)?)),
    }
}

pub fn conv_option_import_call_options(
    py: Python<'_>,
    opts: Option<TsImportCallOptions>,
) -> PyResult<Option<Py<PyTsImportCallOptions>>> {
    match opts {
        None => Ok(None),
        Some(o) => Ok(Some(Py::new(
            py,
            PyTsImportCallOptions {
                span: conv_span(py, o.span)?,
                with: conv_boxed_object_lit(py, o.with)?,
            },
        )?)),
    }
}

pub fn conv_ts_import_type(py: Python<'_>, node: TsImportType) -> PyResult<Py<PyTsImportType>> {
    Py::new(py, (PyTsImportType::build(py, node)?, PyTsType {}))
}

pub fn conv_ts_type_query_expr(
    py: Python<'_>,
    expr: TsTypeQueryExpr,
) -> PyResult<Py<PyTsTypeQueryExpr>> {
    let base = PyTsTypeQueryExpr {};
    Ok(match expr {
        TsTypeQueryExpr::TsEntityName(n) => {
            Py::new(py, (PyTsTypeQueryExprEntityName::build(py, n)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        TsTypeQueryExpr::Import(i) => Py::new(py, (PyTsTypeQueryExprImport::build(py, i)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
    })
}

pub fn conv_fn_param(py: Python<'_>, param: TsFnParam) -> PyResult<Py<PyPat>> {
    let pat = match param {
        TsFnParam::Ident(p) => Pat::Ident(p),
        TsFnParam::Array(p) => Pat::Array(p),
        TsFnParam::Rest(p) => Pat::Rest(p),
        TsFnParam::Object(p) => Pat::Object(p),
    };
    conv_pat(py, pat)
}

pub fn conv_fn_params(py: Python<'_>, params: Vec<TsFnParam>) -> PyResult<Vec<Py<PyPat>>> {
    params.into_iter().map(|p| conv_fn_param(py, p)).collect()
}

pub fn conv_type_param_decl(
    py: Python<'_>,
    decl: TsTypeParamDecl,
) -> PyResult<Py<PyTsTypeParamDecl>> {
    crate::pytypeinfo::wrap_ts_type_param_decl_data(
        py,
        std::sync::Arc::new(crate::pytypeinfo::lower_ts_type_param_decl(decl)),
    )
}

pub fn conv_option_type_param_decl(
    py: Python<'_>,
    decl: Option<Box<TsTypeParamDecl>>,
) -> PyResult<Option<Py<PyTsTypeParamDecl>>> {
    decl.map(|d| conv_type_param_decl(py, *d)).transpose()
}

pub fn conv_ts_type_element(py: Python<'_>, elem: TsTypeElement) -> PyResult<Py<PyTsTypeElement>> {
    let base = PyTsTypeElement {};
    Ok(match elem {
        TsTypeElement::TsCallSignatureDecl(e) => {
            Py::new(py, (PyTsCallSignatureDecl::build(py, e)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        TsTypeElement::TsConstructSignatureDecl(e) => {
            Py::new(py, (PyTsConstructSignatureDecl::build(py, e)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        TsTypeElement::TsPropertySignature(e) => {
            Py::new(py, (PyTsPropertySignature::build(py, e)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        TsTypeElement::TsGetterSignature(e) => {
            Py::new(py, (PyTsGetterSignature::build(py, e)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        TsTypeElement::TsSetterSignature(e) => {
            Py::new(py, (PyTsSetterSignature::build(py, e)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        TsTypeElement::TsMethodSignature(e) => {
            Py::new(py, (PyTsMethodSignature::build(py, e)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        TsTypeElement::TsIndexSignature(e) => {
            Py::new(py, (PyTsIndexSignature::build(py, e)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
    })
}

pub fn conv_ts_type_elements(
    py: Python<'_>,
    elems: Vec<TsTypeElement>,
) -> PyResult<Vec<Py<PyTsTypeElement>>> {
    elems
        .into_iter()
        .map(|e| conv_ts_type_element(py, e))
        .collect()
}

pub fn conv_tuple_element(py: Python<'_>, elem: TsTupleElement) -> PyResult<Py<PyTsTupleElement>> {
    Py::new(
        py,
        PyTsTupleElement {
            span: conv_span(py, elem.span)?,
            label: conv_option_pat(py, elem.label)?,
            ty: conv_boxed_tstype(py, elem.ty)?,
        },
    )
}

pub fn conv_tuple_elements(
    py: Python<'_>,
    elems: Vec<TsTupleElement>,
) -> PyResult<Vec<Py<PyTsTupleElement>>> {
    elems
        .into_iter()
        .map(|e| conv_tuple_element(py, e))
        .collect()
}

pub fn conv_option_true_plus_minus(
    _py: Python<'_>,
    v: Option<TruePlusMinus>,
) -> PyResult<Option<PyTruePlusMinus>> {
    Ok(v.map(|t| t.into()))
}

pub fn conv_ts_type_operator_op(_py: Python<'_>, op: TsTypeOperatorOp) -> PyResult<PyTsTypeOperatorOp> {
    Ok(op.into())
}

pub fn conv_ts_lit(py: Python<'_>, lit: TsLit) -> PyResult<Py<PyTsLit>> {
    let base = PyTsLit {};
    Ok(match lit {
        TsLit::Number(n) => Py::new(py, (PyTsLitNumber::build(py, n)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsLit::Str(s) => Py::new(py, (PyTsLitStr::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsLit::Bool(b) => Py::new(py, (PyTsLitBool::build(py, b)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsLit::BigInt(b) => Py::new(py, (PyTsLitBigInt::build(py, b)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsLit::Tpl(t) => Py::new(py, (PyTsLitTpl::build(py, t)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
    })
}

pub fn conv_tpl_element(py: Python<'_>, el: TplElement) -> PyResult<Py<PyTplElement>> {
    Py::new(py, PyTplElement::from_owned(el))
}

pub fn conv_tpl_elements(py: Python<'_>, els: Vec<TplElement>) -> PyResult<Vec<Py<PyTplElement>>> {
    els.into_iter().map(|e| conv_tpl_element(py, e)).collect()
}

pub fn conv_ts_this_type_or_ident(
    py: Python<'_>,
    node: TsThisTypeOrIdent,
) -> PyResult<Py<PyTsThisTypeOrIdent>> {
    let base = PyTsThisTypeOrIdent {};
    Ok(match node {
        TsThisTypeOrIdent::TsThisType(t) => {
            Py::new(py, (PyTsThisTypeOrIdentThis::build(py, t)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        TsThisTypeOrIdent::Ident(i) => {
            Py::new(py, (PyTsThisTypeOrIdentIdent::build(py, i)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
    })
}

pub fn conv_function(py: Python<'_>, func: Function) -> PyResult<Py<PyFunction>> {
    crate::pyfunction::function_to_py(py, func)
}

pub fn conv_boxed_function(py: Python<'_>, func: Box<Function>) -> PyResult<Py<PyFunction>> {
    conv_function(py, *func)
}

pub fn conv_super(py: Python<'_>, node: Super) -> PyResult<PySuper> {
    Ok(PySuper {
        span: conv_span(py, node.span)?,
    })
}

pub fn conv_super_prop(py: Python<'_>, prop: SuperProp) -> PyResult<Py<PySuperProp>> {
    match prop {
        SuperProp::Ident(id) => {
            let idname = conv_identname(py, id)?;
            Py::new(
                py,
                PySuperProp {
                    ident: Some(idname),
                    computed: None,
                },
            )
        }
        SuperProp::Computed(c) => {
            let prop_name = crate::pyprop::wrap_computed_propname_data(
                py,
                crate::pyprop::lower_computed_propname(c),
            )?;
            Py::new(
                py,
                PySuperProp {
                    ident: None,
                    computed: Some(prop_name),
                },
            )
        }
    }
}

pub fn conv_opt_chain_base(py: Python<'_>, base: OptChainBase) -> PyResult<Py<PyOptChainBase>> {
    let b = PyOptChainBase {};
    Ok(match base {
        OptChainBase::Member(m) => Py::new(py, (PyOptChainBaseMember::build(py, m)?, b))?
            .into_bound(py)
            .into_super()
            .unbind(),
        OptChainBase::Call(c) => Py::new(py, (PyOptCall::build(py, c)?, b))?
            .into_bound(py)
            .into_super()
            .unbind(),
    })
}

pub fn conv_boxed_opt_chain_base(
    py: Python<'_>,
    base: Box<OptChainBase>,
) -> PyResult<Py<PyOptChainBase>> {
    conv_opt_chain_base(py, *base)
}

pub fn conv_option_accessibility(
    _py: Python<'_>,
    acc: Option<Accessibility>,
) -> PyResult<Option<PyAccessibility>> {
    Ok(acc.map(|a| a.into()))
}

pub fn conv_method_kind(_py: Python<'_>, kind: MethodKind) -> PyResult<PyMethodKind> {
    Ok(kind.into())
}

pub fn conv_ts_expr_with_type_args(
    py: Python<'_>,
    node: TsExprWithTypeArgs,
) -> PyResult<Py<PyTsExprWithTypeArgs>> {
    Py::new(
        py,
        PyTsExprWithTypeArgs {
            span: conv_span(py, node.span)?,
            expr: conv_boxed_expr(py, node.expr)?,
            type_args: conv_typeparams(py, node.type_args)?,
        },
    )
}

pub fn conv_ts_expr_with_type_args_vec(
    py: Python<'_>,
    nodes: Vec<TsExprWithTypeArgs>,
) -> PyResult<Vec<Py<PyTsExprWithTypeArgs>>> {
    nodes
        .into_iter()
        .map(|n| conv_ts_expr_with_type_args(py, n))
        .collect()
}

pub fn conv_class(py: Python<'_>, class: Class) -> PyResult<Py<PyClass>> {
    crate::pyclass::class_to_py(py, class)
}

pub fn conv_boxed_class(py: Python<'_>, class: Box<Class>) -> PyResult<Py<PyClass>> {
    conv_class(py, *class)
}

pub fn conv_var_decl_kind(_py: Python<'_>, kind: VarDeclKind) -> PyResult<PyVarDeclKind> {
    Ok(kind.into())
}

pub fn conv_ts_module_name(py: Python<'_>, name: TsModuleName) -> PyResult<PyTsModuleName> {
    match name {
        TsModuleName::Ident(i) => Ok(PyTsModuleName {
            ident: Some(conv_ident(py, i)?),
            str_span: None,
            str_value: None,
        }),
        TsModuleName::Str(s) => Ok(PyTsModuleName {
            ident: None,
            str_span: Some(conv_span(py, s.span)?),
            str_value: Some(conv_str(py, s)?),
        }),
    }
}

pub fn conv_ts_namespace_body(
    py: Python<'_>,
    body: TsNamespaceBody,
) -> PyResult<Py<PyTsNamespaceBody>> {
    let base = PyTsNamespaceBody {};
    Ok(match body {
        TsNamespaceBody::TsModuleBlock(b) => Py::new(py, (PyTsModuleBlock::build(py, b)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        TsNamespaceBody::TsNamespaceDecl(d) => {
            Py::new(py, (PyTsNamespaceDecl::build(py, d)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
    })
}

pub fn conv_option_ts_namespace_body(
    py: Python<'_>,
    body: Option<TsNamespaceBody>,
) -> PyResult<Option<Py<PyTsNamespaceBody>>> {
    match body {
        None => Ok(None),
        Some(b) => Ok(Some(conv_ts_namespace_body(py, b)?)),
    }
}

pub fn conv_boxed_ts_namespace_body(
    py: Python<'_>,
    body: Box<TsNamespaceBody>,
) -> PyResult<Py<PyTsNamespaceBody>> {
    conv_ts_namespace_body(py, *body)
}

pub fn conv_module_items(
    py: Python<'_>,
    items: Vec<ModuleItem>,
) -> PyResult<Vec<Py<PyModuleItem>>> {
    items.into_iter().map(|i| conv_module_item(py, i)).collect()
}

pub fn conv_import_phase(_py: Python<'_>, phase: ImportPhase) -> PyResult<PyImportPhase> {
    Ok(phase.into())
}

pub fn conv_boxed_str(py: Python<'_>, s: Box<Str>) -> PyResult<String> {
    conv_str(py, *s)
}

pub fn conv_option_boxed_str(py: Python<'_>, s: Option<Box<Str>>) -> PyResult<Option<String>> {
    match s {
        None => Ok(None),
        Some(s) => Ok(Some(conv_boxed_str(py, s)?)),
    }
}

pub fn conv_option_boxed_object_lit(
    py: Python<'_>,
    lit: Option<Box<ObjectLit>>,
) -> PyResult<Option<Py<PyExpr>>> {
    match lit {
        None => Ok(None),
        Some(l) => Ok(Some(conv_boxed_object_lit(py, l)?)),
    }
}

pub fn conv_module_export_name(
    py: Python<'_>,
    name: ModuleExportName,
) -> PyResult<PyModuleExportName> {
    match name {
        ModuleExportName::Ident(i) => Ok(PyModuleExportName {
            ident: Some(conv_ident(py, i)?),
            str_span: None,
            str_value: None,
        }),
        ModuleExportName::Str(s) => Ok(PyModuleExportName {
            ident: None,
            str_span: Some(conv_span(py, s.span)?),
            str_value: Some(conv_str(py, s)?),
        }),
    }
}

pub fn conv_option_module_export_name(
    py: Python<'_>,
    name: Option<ModuleExportName>,
) -> PyResult<Option<PyModuleExportName>> {
    match name {
        None => Ok(None),
        Some(n) => Ok(Some(conv_module_export_name(py, n)?)),
    }
}

pub fn conv_ts_module_ref(py: Python<'_>, node: TsModuleRef) -> PyResult<Py<PyTsModuleRef>> {
    match node {
        TsModuleRef::TsEntityName(n) => Py::new(
            py,
            PyTsModuleRef {
                entity_name: Some(conv_tsentityname(py, n)?),
                external_module_ref: None,
            },
        ),
        TsModuleRef::TsExternalModuleRef(r) => {
            let external = Py::new(py, PyTsExternalModuleRef::from_owned(r))?;
            Py::new(
                py,
                PyTsModuleRef {
                    entity_name: None,
                    external_module_ref: Some(external),
                },
            )
        }
    }
}

pub fn module_to_py(py: Python<'_>, module: Module) -> PyResult<Py<PySourceModule>> {
    conv_module(py, module)
}

pub fn conv_jsx_object(py: Python<'_>, obj: JSXObject) -> PyResult<Py<PyJSXObject>> {
    match obj {
        JSXObject::JSXMemberExpr(m) => Py::new(
            py,
            PyJSXObject {
                jsx_member_expr: Some(conv_jsx_member_expr(py, *m)?),
                ident: None,
            },
        ),
        JSXObject::Ident(i) => Py::new(
            py,
            PyJSXObject {
                jsx_member_expr: None,
                ident: Some(conv_ident(py, i)?),
            },
        ),
    }
}

pub fn conv_jsx_member_expr(py: Python<'_>, node: JSXMemberExpr) -> PyResult<Py<PyJSXMemberExpr>> {
    let sub = PyJSXMemberExpr::build(py, node)?;
    Py::new(py, (sub, PyExpr {}))
}

pub fn conv_jsx_member_expr_data(
    py: Python<'_>,
    node: JSXMemberExpr,
) -> PyResult<Py<PyJSXMemberExprData>> {
    Py::new(
        py,
        PyJSXMemberExprData {
            span: conv_span(py, node.span)?,
            obj: conv_jsx_object(py, node.obj)?,
            prop: conv_identname(py, node.prop)?,
        },
    )
}

pub fn conv_jsx_expr(py: Python<'_>, expr: JSXExpr) -> PyResult<Py<PyJSXExpr>> {
    match expr {
        JSXExpr::JSXEmptyExpr(e) => Py::new(
            py,
            PyJSXExpr {
                empty_span: Some(conv_span(py, e.span)?),
                expr: None,
            },
        ),
        JSXExpr::Expr(e) => Py::new(
            py,
            PyJSXExpr {
                empty_span: None,
                expr: Some(conv_boxed_expr(py, e)?),
            },
        ),
    }
}

pub fn conv_jsx_element_name(
    py: Python<'_>,
    name: JSXElementName,
) -> PyResult<Py<PyJSXElementName>> {
    match name {
        JSXElementName::Ident(i) => Py::new(
            py,
            PyJSXElementName {
                ident: Some(conv_ident(py, i)?),
                jsx_member_expr: None,
                jsx_namespaced_name: None,
            },
        ),
        JSXElementName::JSXMemberExpr(m) => {
            let data = conv_jsx_member_expr_data(py, m)?;
            Py::new(
                py,
                PyJSXElementName {
                    ident: None,
                    jsx_member_expr: Some(data),
                    jsx_namespaced_name: None,
                },
            )
        }
        JSXElementName::JSXNamespacedName(n) => Py::new(
            py,
            PyJSXElementName {
                ident: None,
                jsx_member_expr: None,
                jsx_namespaced_name: Some(conv_jsx_namespaced_name_data(py, n)?),
            },
        ),
    }
}

pub fn conv_jsx_namespaced_name_data(
    _py: Python<'_>,
    node: JSXNamespacedName,
) -> PyResult<PyJSXNamespacedNameData> {
    Ok(PyJSXNamespacedNameData::from_owned(node))
}

pub fn conv_jsx_attr_name(py: Python<'_>, name: JSXAttrName) -> PyResult<PyJSXAttrName> {
    match name {
        JSXAttrName::Ident(i) => Ok(PyJSXAttrName {
            ident: Some(conv_identname(py, i)?),
            jsx_namespaced_name: None,
        }),
        JSXAttrName::JSXNamespacedName(n) => Ok(PyJSXAttrName {
            ident: None,
            jsx_namespaced_name: Some(conv_jsx_namespaced_name_data(py, n)?),
        }),
    }
}

pub fn conv_jsx_element(py: Python<'_>, node: JSXElement) -> PyResult<Py<PyJSXElement>> {
    crate::pyjsx::wrap_jsx_element_data(py, std::sync::Arc::new(crate::pyjsx::lower_jsx_element(node)))
}

pub fn conv_option_jsx_closing_element(
    py: Python<'_>,
    node: Option<JSXClosingElement>,
) -> PyResult<Option<Py<PyJSXClosingElement>>> {
    match node {
        None => Ok(None),
        Some(e) => Ok(Some(Py::new(py, PyJSXClosingElement::build(py, e)?)?)),
    }
}

pub fn conv_jsx_opening_element(
    py: Python<'_>,
    node: JSXOpeningElement,
) -> PyResult<Py<PyJSXOpeningElement>> {
    Py::new(py, PyJSXOpeningElement::build(py, node)?)
}

pub fn conv_jsx_fragment(py: Python<'_>, node: JSXFragment) -> PyResult<Py<PyJSXFragment>> {
    crate::pyjsx::wrap_jsx_fragment_data(
        py,
        std::sync::Arc::new(crate::pyjsx::lower_jsx_fragment(node)),
    )
}

pub fn conv_meta_prop_kind(_py: Python<'_>, kind: MetaPropKind) -> PyResult<PyMetaPropKind> {
    Ok(kind.into())
}


pub fn conv_var_decl_or_expr(py: Python<'_>, node: VarDeclOrExpr) -> PyResult<Py<PyVarDeclOrExpr>> {
    match node {
        VarDeclOrExpr::VarDecl(v) => {
            let data = std::sync::Arc::new(crate::pydecl::DeclData::Var(crate::pydecl::lower_var_decl(*v)));
            let vd = crate::pydecl::PyVarDecl::from_arc(data);
            let vd_py = Py::new(py, (vd, crate::pydecl::PyDecl {}))?;
            Py::new(
                py,
                PyVarDeclOrExpr {
                    var_decl: Some(vd_py),
                    expr: None,
                },
            )
        }
        VarDeclOrExpr::Expr(e) => Py::new(
            py,
            PyVarDeclOrExpr {
                var_decl: None,
                expr: Some(conv_boxed_expr(py, e)?),
            },
        ),
    }
}

pub fn conv_option_var_decl_or_expr(
    py: Python<'_>,
    node: Option<VarDeclOrExpr>,
) -> PyResult<Option<Py<PyVarDeclOrExpr>>> {
    match node {
        None => Ok(None),
        Some(n) => Ok(Some(conv_var_decl_or_expr(py, n)?)),
    }
}

pub fn conv_for_head(py: Python<'_>, node: ForHead) -> PyResult<Py<PyForHead>> {
    match node {
        ForHead::VarDecl(v) => {
            let data = std::sync::Arc::new(crate::pydecl::DeclData::Var(crate::pydecl::lower_var_decl(*v)));
            let vd = crate::pydecl::PyVarDecl::from_arc(data);
            let vd_py = Py::new(py, (vd, crate::pydecl::PyDecl {}))?;
            Py::new(
                py,
                PyForHead {
                    var_decl: Some(vd_py),
                    using_decl: None,
                    pat: None,
                },
            )
        }
        ForHead::UsingDecl(u) => {
            let data = std::sync::Arc::new(crate::pydecl::DeclData::Using(crate::pydecl::lower_using_decl(*u)));
            let ud = crate::pydecl::PyUsingDecl::from_arc(data);
            let ud_py = Py::new(py, (ud, crate::pydecl::PyDecl {}))?;
            Py::new(
                py,
                PyForHead {
                    var_decl: None,
                    using_decl: Some(ud_py),
                    pat: None,
                },
            )
        }
        ForHead::Pat(p) => Py::new(
            py,
            PyForHead {
                var_decl: None,
                using_decl: None,
                pat: Some(conv_pat(py, *p)?),
            },
        ),
    }
}
