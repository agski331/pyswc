use pyo3::prelude::*;

use std::sync::Arc;

use swc_core::common::Span;
use swc_core::ecma::ast::{
    AssignTarget, AssignTargetPat, BindingIdent, Callee, Expr, Invalid, Lit, MemberExpr,
    MemberProp, MetaPropExpr, OptCall, OptChainExpr, ParenExpr, Pat, PrivateName,
    SimpleAssignTarget, SuperPropExpr, ThisExpr, Tpl, TsAsExpr, TsInstantiation, TsNonNullExpr,
    TsSatisfiesExpr, TsType, TsTypeAssertion,
};

use crate::conversions::{
    conv_assign_op, conv_binary_op, conv_bindingident, conv_bool, conv_boxed_opt_chain_base,
    conv_boxed_tstype, conv_callee, conv_ctxt, conv_ident,
    conv_member_prop, conv_meta_prop_kind, conv_option_ident,
    conv_pat, conv_span, conv_super, conv_super_prop, conv_tpl_elements, conv_typeparams,
    conv_unary_op, conv_update_op,
};
use crate::macros::{ast_node_variant, arc_variant_node};
use crate::pyclass::PyClass;
use crate::pyenums::{PyAssignOp, PyBinaryOp, PyMetaPropKind, PyUnaryOp, PyUpdateOp};
use crate::pyfunction::{PyCallee, PyFunction, PyFunctionBody};
use crate::pypat::{PatData, lower_pat};
use crate::pytypeinfo::lower_tstypeann;
use crate::pyident::PyIdent;
use crate::pyident::{PyBindingIdent, PyPrivateName};
use crate::pyjsx::{
    PyJSXElement, PyJSXEmptyExpr, PyJSXFragment, PyJSXMemberExpr, PyJSXNamespacedName,
};
use crate::pylit::PyLit;
use crate::pypat::PyPat;
use crate::pyprop::{PyMemberProp, PyPropOrSpread, PySuperProp};
use crate::pyspan::PySpan;
use crate::pytypeinfo::{
    PyTplElement, PyTsType, PyTsTypeParamDecl, PyTsTypeParamInstantiation, TsTypeParamDeclData,
    TsTypeParamInstantiationData, conv_arc_ts_type_param_instantiation,
    conv_option_arc_ts_type_param_decl, conv_option_arc_ts_type_param_instantiation,
    lower_ts_type_param_decl, lower_ts_type_param_instantiation,
};

#[pyclass(subclass)]
pub struct PyExpr {}

pub enum ExprData {
    This(ThisExpr),
    Array(ArrayLitData),
    Object(Arc<ObjectLitData>),
    Fn(Arc<FnExprData>),
    Unary(UnaryExprData),
    Update(UpdateExprData),
    Bin(BinExprData),
    Assign(AssignExprData),
    Member(MemberExprData),
    SuperProp(SuperPropExpr),
    Cond(CondExprData),
    Call(CallExprData),
    New(NewExprData),
    Seq(SeqExprData),
    Ident(swc_core::ecma::ast::Ident),
    Lit(Lit),
    Tpl(Arc<TplData>),
    TaggedTpl(TaggedTplData),
    Arrow(Arc<ArrowExprData>),
    Class(Arc<ClassExprData>),
    Yield(YieldExprData),
    MetaProp(MetaPropExpr),
    Await(AwaitExprData),
    Paren(ParenExprData),
    JSXMember(swc_core::ecma::ast::JSXMemberExpr),
    JSXNamespacedName(swc_core::ecma::ast::JSXNamespacedName),
    JSXEmpty(swc_core::ecma::ast::JSXEmptyExpr),
    JSXElement(Arc<crate::pyjsx::JSXElementData>),
    JSXFragment(Arc<crate::pyjsx::JSXFragmentData>),
    TsTypeAssertion(TsTypeAssertionData),
    TsConstAssertion(TsConstAssertionData),
    TsNonNull(TsNonNullExprData),
    TsAs(TsAsExprData),
    TsInstantiation(TsInstantiationData),
    TsSatisfies(TsSatisfiesExprData),
    PrivateName(PrivateName),
    OptChain(OptChainExpr),
    Invalid(Invalid),
}

pub struct ArrayLitData {
    pub span: Span,
    pub elems: Vec<Option<Arc<ExprData>>>,
}

pub struct UnaryExprData {
    pub span: Span,
    pub op: swc_core::ecma::ast::UnaryOp,
    pub arg: Arc<ExprData>,
}

pub struct UpdateExprData {
    pub span: Span,
    pub op: swc_core::ecma::ast::UpdateOp,
    pub prefix: bool,
    pub arg: Arc<ExprData>,
}

pub struct BinExprData {
    pub span: Span,
    pub op: swc_core::ecma::ast::BinaryOp,
    pub left: Arc<ExprData>,
    pub right: Arc<ExprData>,
}

pub struct AssignExprData {
    pub span: Span,
    pub op: swc_core::ecma::ast::AssignOp,
    pub left: AssignTarget,
    pub right: Arc<ExprData>,
}

pub struct MemberExprData {
    pub span: Span,
    pub obj: Arc<ExprData>,
    pub prop: MemberProp,
}

pub struct CondExprData {
    pub span: Span,
    pub test: Arc<ExprData>,
    pub cons: Arc<ExprData>,
    pub alt: Arc<ExprData>,
}

pub struct CallExprData {
    pub span: Span,
    pub ctxt: swc_core::common::SyntaxContext,
    pub callee: Callee,
    pub args: Vec<Arc<ExprData>>,
    pub type_args: Option<Arc<TsTypeParamInstantiationData>>,
}

pub struct NewExprData {
    pub span: Span,
    pub ctxt: swc_core::common::SyntaxContext,
    pub callee: Arc<ExprData>,
    pub args: Option<Vec<crate::pyprop::ExprOrSpreadData>>,
    pub type_args: Option<Arc<TsTypeParamInstantiationData>>,
}

pub struct SeqExprData {
    pub span: Span,
    pub exprs: Vec<Arc<ExprData>>,
}

pub struct TplData {
    pub span: Span,
    pub exprs: Vec<Arc<ExprData>>,
    pub quasis: Vec<swc_core::ecma::ast::TplElement>,
}

pub struct TaggedTplData {
    pub span: Span,
    pub ctxt: swc_core::common::SyntaxContext,
    pub tag: Arc<ExprData>,
    pub type_params: Option<Arc<TsTypeParamInstantiationData>>,
    pub tpl: Arc<TplData>,
}

pub struct YieldExprData {
    pub span: Span,
    pub arg: Option<Arc<ExprData>>,
    pub delegate: bool,
}

pub struct AwaitExprData {
    pub span: Span,
    pub arg: Arc<ExprData>,
}

pub struct ParenExprData {
    pub span: Span,
    pub expr: Arc<ExprData>,
}

pub struct TsTypeAssertionData {
    pub span: Span,
    pub expr: Arc<ExprData>,
    pub type_ann: Box<TsType>,
}

pub struct TsConstAssertionData {
    pub span: Span,
    pub expr: Arc<ExprData>,
}

pub struct TsNonNullExprData {
    pub span: Span,
    pub expr: Arc<ExprData>,
}

pub struct TsAsExprData {
    pub span: Span,
    pub expr: Arc<ExprData>,
    pub type_ann: Box<TsType>,
}

pub struct TsInstantiationData {
    pub span: Span,
    pub expr: Arc<ExprData>,
    pub type_args: Arc<TsTypeParamInstantiationData>,
}

pub struct TsSatisfiesExprData {
    pub span: Span,
    pub expr: Arc<ExprData>,
    pub type_ann: Box<TsType>,
}

pub fn lower_expr(expr: Expr) -> ExprData {
    match expr {
        Expr::This(e) => ExprData::This(e),
        Expr::Array(e) => ExprData::Array(ArrayLitData {
            span: e.span,
            elems: e
                .elems
                .into_iter()
                .map(|opt| opt.map(|eos| Arc::new(lower_expr(*eos.expr))))
                .collect(),
        }),
        Expr::Object(e) => ExprData::Object(Arc::new(ObjectLitData {
            span: e.span,
            props: e
                .props
                .into_iter()
                .map(|p| Arc::new(crate::pyprop::lower_prop_or_spread(p)))
                .collect(),
        })),
        Expr::Fn(e) => ExprData::Fn(Arc::new(FnExprData {
            ident: e.ident,
            function: Arc::new(crate::pyfunction::lower_function(*e.function)),
        })),
        Expr::Unary(e) => ExprData::Unary(UnaryExprData {
            span: e.span,
            op: e.op,
            arg: Arc::new(lower_expr(*e.arg)),
        }),
        Expr::Update(e) => ExprData::Update(UpdateExprData {
            span: e.span,
            op: e.op,
            prefix: e.prefix,
            arg: Arc::new(lower_expr(*e.arg)),
        }),
        Expr::Bin(e) => ExprData::Bin(BinExprData {
            span: e.span,
            op: e.op,
            left: Arc::new(lower_expr(*e.left)),
            right: Arc::new(lower_expr(*e.right)),
        }),
        Expr::Assign(e) => ExprData::Assign(AssignExprData {
            span: e.span,
            op: e.op,
            left: e.left,
            right: Arc::new(lower_expr(*e.right)),
        }),
        Expr::Member(e) => ExprData::Member(MemberExprData {
            span: e.span,
            obj: Arc::new(lower_expr(*e.obj)),
            prop: e.prop,
        }),
        Expr::SuperProp(e) => ExprData::SuperProp(e),
        Expr::Cond(e) => ExprData::Cond(CondExprData {
            span: e.span,
            test: Arc::new(lower_expr(*e.test)),
            cons: Arc::new(lower_expr(*e.cons)),
            alt: Arc::new(lower_expr(*e.alt)),
        }),
        Expr::Call(e) => ExprData::Call(CallExprData {
            span: e.span,
            ctxt: e.ctxt,
            callee: e.callee,
            args: e
                .args
                .into_iter()
                .map(|eos| Arc::new(lower_expr(*eos.expr)))
                .collect(),
            type_args: e.type_args.map(|tp| Arc::new(lower_ts_type_param_instantiation(*tp))),
        }),
        Expr::New(e) => ExprData::New(NewExprData {
            span: e.span,
            ctxt: e.ctxt,
            callee: Arc::new(lower_expr(*e.callee)),
            args: e.args.map(|args| {
                args.into_iter().map(crate::pyprop::lower_expr_or_spread).collect()
            }),
            type_args: e.type_args.map(|tp| Arc::new(lower_ts_type_param_instantiation(*tp))),
        }),
        Expr::Seq(e) => ExprData::Seq(SeqExprData {
            span: e.span,
            exprs: e.exprs.into_iter().map(|x| Arc::new(lower_expr(*x))).collect(),
        }),
        Expr::Ident(e) => ExprData::Ident(e),
        Expr::Lit(e) => ExprData::Lit(e),
        Expr::Tpl(e) => ExprData::Tpl(Arc::new(lower_tpl(e))),
        Expr::TaggedTpl(e) => ExprData::TaggedTpl(TaggedTplData {
            span: e.span,
            ctxt: e.ctxt,
            tag: Arc::new(lower_expr(*e.tag)),
            type_params: e.type_params.map(|tp| Arc::new(lower_ts_type_param_instantiation(*tp))),
            tpl: Arc::new(lower_tpl(*e.tpl)),
        }),
        Expr::Arrow(e) => ExprData::Arrow(Arc::new(ArrowExprData {
            span: e.span,
            ctxt: e.ctxt,
            params: e.params.into_iter().map(|p| Arc::new(lower_pat(p))).collect(),
            body: lower_arrow_function_body(*e.body),
            is_async: e.is_async,
            is_generator: e.is_generator,
            type_params: e.type_params.map(|tp| Arc::new(lower_ts_type_param_decl(*tp))),
            return_type: e.return_type.map(|r| Arc::new(lower_tstypeann(*r))),
        })),
        Expr::Class(e) => ExprData::Class(Arc::new(ClassExprData {
            ident: e.ident,
            class: Arc::new(crate::pyclass::lower_class(*e.class)),
        })),
        Expr::Yield(e) => ExprData::Yield(YieldExprData {
            span: e.span,
            arg: e.arg.map(|a| Arc::new(lower_expr(*a))),
            delegate: e.delegate,
        }),
        Expr::MetaProp(e) => ExprData::MetaProp(e),
        Expr::Await(e) => ExprData::Await(AwaitExprData {
            span: e.span,
            arg: Arc::new(lower_expr(*e.arg)),
        }),
        Expr::Paren(e) => ExprData::Paren(ParenExprData {
            span: e.span,
            expr: Arc::new(lower_expr(*e.expr)),
        }),
        Expr::JSXMember(e) => ExprData::JSXMember(e),
        Expr::JSXNamespacedName(e) => ExprData::JSXNamespacedName(e),
        Expr::JSXEmpty(e) => ExprData::JSXEmpty(e),
        Expr::JSXElement(e) => {
            ExprData::JSXElement(Arc::new(crate::pyjsx::lower_jsx_element(*e)))
        }
        Expr::JSXFragment(e) => {
            ExprData::JSXFragment(Arc::new(crate::pyjsx::lower_jsx_fragment(e)))
        }
        Expr::TsTypeAssertion(e) => ExprData::TsTypeAssertion(TsTypeAssertionData {
            span: e.span,
            expr: Arc::new(lower_expr(*e.expr)),
            type_ann: e.type_ann,
        }),
        Expr::TsConstAssertion(e) => ExprData::TsConstAssertion(TsConstAssertionData {
            span: e.span,
            expr: Arc::new(lower_expr(*e.expr)),
        }),
        Expr::TsNonNull(e) => ExprData::TsNonNull(TsNonNullExprData {
            span: e.span,
            expr: Arc::new(lower_expr(*e.expr)),
        }),
        Expr::TsAs(e) => ExprData::TsAs(TsAsExprData {
            span: e.span,
            expr: Arc::new(lower_expr(*e.expr)),
            type_ann: e.type_ann,
        }),
        Expr::TsInstantiation(e) => ExprData::TsInstantiation(TsInstantiationData {
            span: e.span,
            expr: Arc::new(lower_expr(*e.expr)),
            type_args: Arc::new(lower_ts_type_param_instantiation(*e.type_args)),
        }),
        Expr::TsSatisfies(e) => ExprData::TsSatisfies(TsSatisfiesExprData {
            span: e.span,
            expr: Arc::new(lower_expr(*e.expr)),
            type_ann: e.type_ann,
        }),
        Expr::PrivateName(e) => ExprData::PrivateName(e),
        Expr::OptChain(e) => ExprData::OptChain(e),
        Expr::Invalid(e) => ExprData::Invalid(e),
    }
}

pub fn lower_tpl(tpl: Tpl) -> TplData {
    TplData {
        span: tpl.span,
        exprs: tpl.exprs.into_iter().map(|x| Arc::new(lower_expr(*x))).collect(),
        quasis: tpl.quasis,
    }
}

pub fn wrap_expr_data(py: Python<'_>, data: Arc<ExprData>) -> PyResult<Py<PyExpr>> {
    let base = PyExpr {};
    Ok(match &*data {
        ExprData::This(e) => Py::new(py, (PyThisExpr::build(py, e.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Array(_) => Py::new(py, (PyArrayLitExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Object(_) => Py::new(py, (PyObjectLit::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Fn(_) => Py::new(py, (PyFnExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Unary(_) => Py::new(py, (PyUnaryExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Update(_) => Py::new(py, (PyUpdateExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Bin(_) => Py::new(py, (PyBinExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Assign(_) => Py::new(py, (PyAssignExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Member(_) => Py::new(py, (PyMemberExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::SuperProp(e) => Py::new(py, (PySuperPropExpr::build(py, e.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Cond(_) => Py::new(py, (PyCondExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Call(_) => Py::new(py, (PyCallExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::New(_) => Py::new(py, (PyNewExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Seq(_) => Py::new(py, (PySeqExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Ident(e) => Py::new(py, (PyIdentExpr::build(py, e.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Lit(e) => Py::new(py, (PyLitExpr::build(py, e.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Tpl(tpl_data) => Py::new(py, (PyTpl::from_arc(Arc::clone(tpl_data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::TaggedTpl(_) => Py::new(py, (PyTaggedTpl::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Arrow(_) => Py::new(py, (PyArrowExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Class(e) => Py::new(py, (PyClassExpr::from_arc(Arc::clone(e)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Yield(_) => Py::new(py, (PyYieldExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::MetaProp(e) => Py::new(py, (PyMetaPropExpr::build(py, e.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Await(_) => Py::new(py, (PyAwaitExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Paren(_) => Py::new(py, (PyParenExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::JSXMember(e) => Py::new(py, (PyJSXMemberExpr::build(py, e.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::JSXNamespacedName(e) => {
            Py::new(py, (PyJSXNamespacedName::build(py, e.clone())?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        ExprData::JSXEmpty(e) => Py::new(py, (PyJSXEmptyExpr::build(py, e.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::JSXElement(e) => Py::new(py, (PyJSXElement::from_arc(Arc::clone(e)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::JSXFragment(e) => Py::new(py, (PyJSXFragment::from_arc(Arc::clone(e)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::TsTypeAssertion(_) => {
            Py::new(py, (PyTsTypeAssertion::from_arc(Arc::clone(&data)), base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        ExprData::TsConstAssertion(_) => {
            Py::new(py, (PyTsConstAssertion::from_arc(Arc::clone(&data)), base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        ExprData::TsNonNull(_) => Py::new(py, (PyTsNonNullExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::TsAs(_) => Py::new(py, (PyTsAsExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::TsInstantiation(_) => {
            Py::new(py, (PyTsInstantiation::from_arc(Arc::clone(&data)), base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        ExprData::TsSatisfies(_) => Py::new(py, (PyTsSatisfiesExpr::from_arc(Arc::clone(&data)), base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::PrivateName(e) => Py::new(py, (PyPrivateNameExpr::build(py, e.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::OptChain(e) => Py::new(py, (PyOptChainExpr::build(py, e.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        ExprData::Invalid(e) => Py::new(py, (PyInvalidExpr::build(py, e.clone())?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
    })
}

pub fn wrap_tpl(py: Python<'_>, data: Arc<TplData>) -> PyResult<Py<PyTpl>> {
    Py::new(py, (PyTpl { inner: data }, PyExpr {}))
}

pub fn conv_arc_expr(py: Python<'_>, data: Arc<ExprData>) -> PyResult<Py<PyExpr>> {
    wrap_expr_data(py, data)
}

pub fn conv_option_arc_expr(
    py: Python<'_>,
    data: Option<Arc<ExprData>>,
) -> PyResult<Option<Py<PyExpr>>> {
    data.map(|d| wrap_expr_data(py, d)).transpose()
}

pub fn conv_arc_exprs(py: Python<'_>, data: Vec<Arc<ExprData>>) -> PyResult<Vec<Py<PyExpr>>> {
    data.into_iter().map(|d| wrap_expr_data(py, d)).collect()
}

pub fn conv_option_arc_expr_elems(
    py: Python<'_>,
    data: Vec<Option<Arc<ExprData>>>,
) -> PyResult<Vec<Option<Py<PyExpr>>>> {
    data.into_iter()
        .map(|opt| opt.map(|d| wrap_expr_data(py, d)).transpose())
        .collect()
}

pub fn conv_arc_tpl(py: Python<'_>, data: Arc<TplData>) -> PyResult<Py<PyTpl>> {
    wrap_tpl(py, data)
}

arc_variant_node!(PyExpr, PyArrayLitExpr, ExprData, ExprData::Array, ArrayLitData, {
    span: PySpan = conv_span,
    elems: Vec<Option<Py<PyExpr>>> = conv_option_arc_expr_elems,
});

arc_variant_node!(PyExpr, PyUnaryExpr, ExprData, ExprData::Unary, UnaryExprData, {
    span: PySpan = conv_span,
    op: PyUnaryOp = conv_unary_op,
    arg: Py<PyExpr> = conv_arc_expr,
});

arc_variant_node!(PyExpr, PyUpdateExpr, ExprData, ExprData::Update, UpdateExprData, {
    span: PySpan = conv_span,
    op: PyUpdateOp = conv_update_op,
    prefix: bool = conv_bool,
    arg: Py<PyExpr> = conv_arc_expr,
});

arc_variant_node!(PyExpr, PyBinExpr, ExprData, ExprData::Bin, BinExprData, {
    span: PySpan = conv_span,
    op: PyBinaryOp = conv_binary_op,
    left: Py<PyExpr> = conv_arc_expr,
    right: Py<PyExpr> = conv_arc_expr,
});

arc_variant_node!(PyExpr, PyAssignExpr, ExprData, ExprData::Assign, AssignExprData, {
    span: PySpan = conv_span,
    op: PyAssignOp = conv_assign_op,
    left: Py<PyAssignTarget> = conv_assign_target,
    right: Py<PyExpr> = conv_arc_expr,
});

arc_variant_node!(PyExpr, PyMemberExpr, ExprData, ExprData::Member, MemberExprData, {
    span: PySpan = conv_span,
    obj: Py<PyExpr> = conv_arc_expr,
    prop: Py<PyMemberProp> = conv_member_prop,
});

arc_variant_node!(PyExpr, PyCondExpr, ExprData, ExprData::Cond, CondExprData, {
    span: PySpan = conv_span,
    test: Py<PyExpr> = conv_arc_expr,
    cons: Py<PyExpr> = conv_arc_expr,
    alt: Py<PyExpr> = conv_arc_expr,
});

arc_variant_node!(PyExpr, PyCallExpr, ExprData, ExprData::Call, CallExprData, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    callee: Py<PyCallee> = conv_callee,
    args: Vec<Py<PyExpr>> = conv_arc_exprs,
    type_args: Option<Py<PyTsTypeParamInstantiation>> = conv_option_arc_ts_type_param_instantiation,
});

arc_variant_node!(PyExpr, PyNewExpr, ExprData, ExprData::New, NewExprData, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    callee: Py<PyExpr> = conv_arc_expr,
    args: Option<Vec<Py<crate::pyprop::PyExprOrSpread>>> = crate::pyprop::conv_option_arc_expr_or_spreads,
    type_args: Option<Py<PyTsTypeParamInstantiation>> = conv_option_arc_ts_type_param_instantiation,
});

arc_variant_node!(PyExpr, PySeqExpr, ExprData, ExprData::Seq, SeqExprData, {
    span: PySpan = conv_span,
    exprs: Vec<Py<PyExpr>> = conv_arc_exprs,
});

arc_variant_node!(PyExpr, PyYieldExpr, ExprData, ExprData::Yield, YieldExprData, {
    span: PySpan = conv_span,
    arg: Option<Py<PyExpr>> = conv_option_arc_expr,
    delegate: bool = conv_bool,
});

arc_variant_node!(PyExpr, PyAwaitExpr, ExprData, ExprData::Await, AwaitExprData, {
    span: PySpan = conv_span,
    arg: Py<PyExpr> = conv_arc_expr,
});

arc_variant_node!(PyExpr, PyParenExpr, ExprData, ExprData::Paren, ParenExprData, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_arc_expr,
});

arc_variant_node!(PyExpr, PyTsTypeAssertion, ExprData, ExprData::TsTypeAssertion, TsTypeAssertionData, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_arc_expr,
    type_ann: Py<PyTsType> = conv_boxed_tstype,
});

arc_variant_node!(PyExpr, PyTsConstAssertion, ExprData, ExprData::TsConstAssertion, TsConstAssertionData, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_arc_expr,
});

arc_variant_node!(PyExpr, PyTsNonNullExpr, ExprData, ExprData::TsNonNull, TsNonNullExprData, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_arc_expr,
});

arc_variant_node!(PyExpr, PyTsAsExpr, ExprData, ExprData::TsAs, TsAsExprData, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_arc_expr,
    type_ann: Py<PyTsType> = conv_boxed_tstype,
});

arc_variant_node!(PyExpr, PyTsInstantiation, ExprData, ExprData::TsInstantiation, TsInstantiationData, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_arc_expr,
    type_args: Py<PyTsTypeParamInstantiation> = conv_arc_ts_type_param_instantiation,
});

arc_variant_node!(PyExpr, PyTsSatisfiesExpr, ExprData, ExprData::TsSatisfies, TsSatisfiesExprData, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_arc_expr,
    type_ann: Py<PyTsType> = conv_boxed_tstype,
});

#[pyclass(extends=PyExpr)]
pub struct PyTpl {
    inner: Arc<TplData>,
}

impl PyTpl {
    pub fn from_arc(inner: Arc<TplData>) -> Self {
        PyTpl { inner }
    }
}

#[pymethods]
impl PyTpl {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.inner.span)
    }

    #[getter]
    fn exprs(&self, py: Python<'_>) -> PyResult<Vec<Py<PyExpr>>> {
        conv_arc_exprs(py, self.inner.exprs.clone())
    }

    #[getter]
    fn quasis(&self, py: Python<'_>) -> PyResult<Vec<Py<PyTplElement>>> {
        conv_tpl_elements(py, self.inner.quasis.clone())
    }
}

arc_variant_node!(PyExpr, PyTaggedTpl, ExprData, ExprData::TaggedTpl, TaggedTplData, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    tag: Py<PyExpr> = conv_arc_expr,
    type_params: Option<Py<PyTsTypeParamInstantiation>> = conv_option_arc_ts_type_param_instantiation,
    tpl: Py<PyTpl> = conv_arc_tpl,
});

ast_node_variant!(PyExpr, PyThisExpr, ThisExpr, {
    span: PySpan = conv_span,
});

pub struct ObjectLitData {
    pub span: Span,
    pub props: Vec<Arc<crate::pyprop::PropOrSpreadData>>,
}

pub struct FnExprData {
    pub ident: Option<swc_core::ecma::ast::Ident>,
    pub function: Arc<crate::pyfunction::FunctionData>,
}

#[pyclass(extends=PyExpr)]
pub struct PyObjectLit {
    inner: Arc<ExprData>,
}

impl PyObjectLit {
    pub fn from_arc(inner: Arc<ExprData>) -> Self {
        PyObjectLit { inner }
    }

    fn data(&self) -> &Arc<ObjectLitData> {
        match &*self.inner {
            ExprData::Object(d) => d,
            _ => unreachable!("ExprData/PyObjectLit mismatch"),
        }
    }
}

#[pymethods]
impl PyObjectLit {
    #[getter]
    fn span(&self, py: Python<'_>) -> PyResult<PySpan> {
        conv_span(py, self.data().span)
    }

    #[getter]
    fn props(&self, py: Python<'_>) -> PyResult<Vec<Py<PyPropOrSpread>>> {
        crate::pyprop::conv_arc_prop_or_spreads(py, self.data().props.clone())
    }
}

#[pyclass(extends=PyExpr)]
pub struct PyFnExpr {
    inner: Arc<ExprData>,
}

impl PyFnExpr {
    pub fn from_arc(inner: Arc<ExprData>) -> Self {
        PyFnExpr { inner }
    }

    fn data(&self) -> &Arc<FnExprData> {
        match &*self.inner {
            ExprData::Fn(d) => d,
            _ => unreachable!("ExprData/PyFnExpr mismatch"),
        }
    }
}

#[pymethods]
impl PyFnExpr {
    #[getter]
    fn ident(&self, py: Python<'_>) -> PyResult<Option<PyIdent>> {
        conv_option_ident(py, self.data().ident.clone())
    }

    #[getter]
    fn function(&self, py: Python<'_>) -> PyResult<Py<PyFunction>> {
        crate::pyfunction::wrap_function_data(py, Arc::clone(&self.data().function))
    }
}

#[derive(Clone)]
#[pyclass]
pub struct PySuper {
    #[pyo3(get)]
    pub span: PySpan,
}

ast_node_variant!(PyExpr, PySuperPropExpr, SuperPropExpr, {
    span: PySpan = conv_span,
    obj: PySuper = conv_super,
    prop: Py<PySuperProp> = conv_super_prop
});

#[pyclass(subclass)]
pub struct PyOptChainBase {}

#[pyclass(extends=PyOptChainBase)]
pub struct PyOptChainBaseMember {
    #[pyo3(get)]
    pub member: Py<PyMemberExpr>,
}

impl PyOptChainBaseMember {
    pub fn build(py: Python<'_>, node: MemberExpr) -> PyResult<Self> {
        let sub = PyMemberExpr::from_arc(Arc::new(lower_expr(Expr::Member(node))));
        Ok(PyOptChainBaseMember {
            member: Py::new(py, (sub, PyExpr {}))?,
        })
    }
}

ast_node_variant!(PyOptChainBase, PyOptCall, OptCall, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    callee: Py<PyExpr> = crate::conversions::conv_boxed_expr,
    args: Vec<Py<PyExpr>> = crate::conversions::conv_elems_noopt,
    type_args: Option<Py<PyTsTypeParamInstantiation>> = conv_typeparams
});

ast_node_variant!(PyExpr, PyOptChainExpr, OptChainExpr, {
    span: PySpan = conv_span,
    optional: bool = conv_bool,
    base: Py<PyOptChainBase> = conv_boxed_opt_chain_base
});

pub fn expr_to_py(py: Python<'_>, expr: Expr) -> PyResult<Py<PyExpr>> {
    wrap_expr_data(py, Arc::new(lower_expr(expr)))
}

#[pyclass(subclass)]
pub struct PySimpleAssignTarget {}

#[pyclass(extends=PySimpleAssignTarget)]
pub struct PySimpleAssignTargetIdent {
    #[pyo3(get)]
    pub ident: Py<PyBindingIdent>,
}

impl PySimpleAssignTargetIdent {
    pub fn build(py: Python<'_>, node: BindingIdent) -> PyResult<Self> {
        Ok(PySimpleAssignTargetIdent {
            ident: conv_bindingident(py, node)?,
        })
    }
}

macro_rules! simple_assign_target_expr_variant {
    ($py_name:ident, $field:ident, $py_expr_ty:ident, $swc_ty:ty, $expr_variant:ident) => {
        #[pyclass(extends=PySimpleAssignTarget)]
        pub struct $py_name {
            #[pyo3(get)]
            pub $field: Py<$py_expr_ty>,
        }

        impl $py_name {
            pub fn build(py: Python<'_>, node: $swc_ty) -> PyResult<Self> {
                let sub = $py_expr_ty::from_arc(Arc::new(lower_expr(Expr::$expr_variant(node))));
                Ok($py_name {
                    $field: Py::new(py, (sub, PyExpr {}))?,
                })
            }
        }
    };
}

simple_assign_target_expr_variant!(
    PySimpleAssignTargetMember,
    member,
    PyMemberExpr,
    MemberExpr,
    Member
);
#[pyclass(extends=PySimpleAssignTarget)]
pub struct PySimpleAssignTargetSuperProp {
    #[pyo3(get)]
    pub super_prop: Py<PySuperPropExpr>,
}

impl PySimpleAssignTargetSuperProp {
    pub fn build(py: Python<'_>, node: SuperPropExpr) -> PyResult<Self> {
        Ok(PySimpleAssignTargetSuperProp {
            super_prop: Py::new(py, (PySuperPropExpr::build(py, node)?, PyExpr {}))?,
        })
    }
}

simple_assign_target_expr_variant!(
    PySimpleAssignTargetParen,
    paren,
    PyParenExpr,
    ParenExpr,
    Paren
);

#[pyclass(extends=PySimpleAssignTarget)]
pub struct PySimpleAssignTargetOptChain {
    #[pyo3(get)]
    pub opt_chain: Py<PyOptChainExpr>,
}

impl PySimpleAssignTargetOptChain {
    pub fn build(py: Python<'_>, node: OptChainExpr) -> PyResult<Self> {
        Ok(PySimpleAssignTargetOptChain {
            opt_chain: Py::new(py, (PyOptChainExpr::build(py, node)?, PyExpr {}))?,
        })
    }
}
simple_assign_target_expr_variant!(PySimpleAssignTargetTsAs, ts_as, PyTsAsExpr, TsAsExpr, TsAs);
simple_assign_target_expr_variant!(
    PySimpleAssignTargetTsSatisfies,
    ts_satisfies,
    PyTsSatisfiesExpr,
    TsSatisfiesExpr,
    TsSatisfies
);
simple_assign_target_expr_variant!(
    PySimpleAssignTargetTsNonNull,
    ts_non_null,
    PyTsNonNullExpr,
    TsNonNullExpr,
    TsNonNull
);
simple_assign_target_expr_variant!(
    PySimpleAssignTargetTsTypeAssertion,
    ts_type_assertion,
    PyTsTypeAssertion,
    TsTypeAssertion,
    TsTypeAssertion
);
simple_assign_target_expr_variant!(
    PySimpleAssignTargetTsInstantiation,
    ts_instantiation,
    PyTsInstantiation,
    TsInstantiation,
    TsInstantiation
);

#[pyclass(extends=PySimpleAssignTarget)]
pub struct PySimpleAssignTargetInvalid {
    #[pyo3(get)]
    pub span: PySpan,
}

impl PySimpleAssignTargetInvalid {
    pub fn build(py: Python<'_>, node: Invalid) -> PyResult<Self> {
        Ok(PySimpleAssignTargetInvalid {
            span: conv_span(py, node.span)?,
        })
    }
}

pub fn conv_simple_assign_target(
    py: Python<'_>,
    target: SimpleAssignTarget,
) -> PyResult<Py<PySimpleAssignTarget>> {
    let base = PySimpleAssignTarget {};
    Ok(match target {
        SimpleAssignTarget::Ident(i) => {
            Py::new(py, (PySimpleAssignTargetIdent::build(py, i)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        SimpleAssignTarget::Member(m) => {
            Py::new(py, (PySimpleAssignTargetMember::build(py, m)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        SimpleAssignTarget::SuperProp(s) => {
            Py::new(py, (PySimpleAssignTargetSuperProp::build(py, s)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        SimpleAssignTarget::Paren(p) => {
            Py::new(py, (PySimpleAssignTargetParen::build(py, p)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        SimpleAssignTarget::OptChain(o) => {
            Py::new(py, (PySimpleAssignTargetOptChain::build(py, o)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        SimpleAssignTarget::TsAs(t) => {
            Py::new(py, (PySimpleAssignTargetTsAs::build(py, t)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        SimpleAssignTarget::TsSatisfies(t) => {
            Py::new(py, (PySimpleAssignTargetTsSatisfies::build(py, t)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        SimpleAssignTarget::TsNonNull(t) => {
            Py::new(py, (PySimpleAssignTargetTsNonNull::build(py, t)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
        SimpleAssignTarget::TsTypeAssertion(t) => Py::new(
            py,
            (PySimpleAssignTargetTsTypeAssertion::build(py, t)?, base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        SimpleAssignTarget::TsInstantiation(t) => Py::new(
            py,
            (PySimpleAssignTargetTsInstantiation::build(py, t)?, base),
        )?
        .into_bound(py)
        .into_super()
        .unbind(),
        SimpleAssignTarget::Invalid(i) => {
            Py::new(py, (PySimpleAssignTargetInvalid::build(py, i)?, base))?
                .into_bound(py)
                .into_super()
                .unbind()
        }
    })
}

#[pyclass(subclass)]
pub struct PyAssignTarget {}

#[pyclass(extends=PyAssignTarget)]
pub struct PyAssignTargetSimple {
    #[pyo3(get)]
    pub target: Py<PySimpleAssignTarget>,
}

impl PyAssignTargetSimple {
    pub fn build(py: Python<'_>, node: SimpleAssignTarget) -> PyResult<Self> {
        Ok(PyAssignTargetSimple {
            target: conv_simple_assign_target(py, node)?,
        })
    }
}

#[pyclass(extends=PyAssignTarget)]
pub struct PyAssignTargetPat {
    #[pyo3(get)]
    pub pat: Py<PyPat>,
}

impl PyAssignTargetPat {
    pub fn build(py: Python<'_>, node: AssignTargetPat) -> PyResult<Self> {
        let pat = match node {
            AssignTargetPat::Array(a) => Pat::Array(a),
            AssignTargetPat::Object(o) => Pat::Object(o),
            AssignTargetPat::Invalid(i) => Pat::Invalid(i),
        };
        Ok(PyAssignTargetPat {
            pat: conv_pat(py, pat)?,
        })
    }
}

pub fn conv_assign_target(py: Python<'_>, target: AssignTarget) -> PyResult<Py<PyAssignTarget>> {
    let base = PyAssignTarget {};
    Ok(match target {
        AssignTarget::Simple(s) => Py::new(py, (PyAssignTargetSimple::build(py, s)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
        AssignTarget::Pat(p) => Py::new(py, (PyAssignTargetPat::build(py, p)?, base))?
            .into_bound(py)
            .into_super()
            .unbind(),
    })
}

pub fn assign_target_to_py(py: Python<'_>, target: AssignTarget) -> PyResult<Py<PyAssignTarget>> {
    conv_assign_target(py, target)
}

#[pyclass(extends=PyExpr)]
pub struct PyIdentExpr {
    #[pyo3(get)]
    pub ident: PyIdent,
}

impl PyIdentExpr {
    pub fn build(py: Python<'_>, node: swc_core::ecma::ast::Ident) -> PyResult<Self> {
        Ok(PyIdentExpr {
            ident: conv_ident(py, node)?,
        })
    }
}

#[pyclass(extends=PyExpr)]
pub struct PyLitExpr {
    #[pyo3(get)]
    pub lit: Py<PyLit>,
}

impl PyLitExpr {
    pub fn build(py: Python<'_>, node: Lit) -> PyResult<Self> {
        Ok(PyLitExpr {
            lit: Py::new(py, PyLit { lit: node })?,
        })
    }
}

#[derive(Clone)]
pub enum ArrowFunctionBodyData {
    FunctionBody(Arc<crate::pyfunction::FunctionBodyData>),
    Expr(Arc<ExprData>),
}

pub fn lower_arrow_function_body(
    b: swc_core::ecma::ast::ArrowFunctionBody,
) -> ArrowFunctionBodyData {
    match b {
        swc_core::ecma::ast::ArrowFunctionBody::FunctionBody(fb) => {
            ArrowFunctionBodyData::FunctionBody(Arc::new(crate::pyfunction::lower_function_body(fb)))
        }
        swc_core::ecma::ast::ArrowFunctionBody::Expr(e) => {
            ArrowFunctionBodyData::Expr(Arc::new(lower_expr(*e)))
        }
    }
}

pub struct ArrowExprData {
    pub span: Span,
    pub ctxt: swc_core::common::SyntaxContext,
    pub params: Vec<Arc<PatData>>,
    pub body: ArrowFunctionBodyData,
    pub is_async: bool,
    pub is_generator: bool,
    pub type_params: Option<Arc<TsTypeParamDeclData>>,
    pub return_type: Option<Arc<crate::pytypeinfo::TsTypeAnnData>>,
}

#[pyclass]
pub struct PyArrowFunctionBody {
    #[pyo3(get)]
    pub function_body: Option<Py<PyFunctionBody>>,
    #[pyo3(get)]
    pub expr: Option<Py<PyExpr>>,
}

pub fn wrap_arrow_function_body_data(
    py: Python<'_>,
    data: ArrowFunctionBodyData,
) -> PyResult<Py<PyArrowFunctionBody>> {
    match data {
        ArrowFunctionBodyData::FunctionBody(fb) => Py::new(
            py,
            PyArrowFunctionBody {
                function_body: Some(crate::pyfunction::wrap_function_body_data(py, fb)?),
                expr: None,
            },
        ),
        ArrowFunctionBodyData::Expr(e) => Py::new(
            py,
            PyArrowFunctionBody {
                function_body: None,
                expr: Some(conv_arc_expr(py, e)?),
            },
        ),
    }
}

arc_variant_node!(PyExpr, PyArrowExpr, ExprData, ExprData::Arrow, Arc<ArrowExprData>, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    params: Vec<Py<PyPat>> = crate::pypat::conv_arc_pats,
    body: Py<PyArrowFunctionBody> = wrap_arrow_function_body_data,
    is_async: bool = conv_bool,
    is_generator: bool = conv_bool,
    type_params: Option<Py<PyTsTypeParamDecl>> = conv_option_arc_ts_type_param_decl,
    return_type: Option<Py<crate::pytypeinfo::PyTsTypeAnn>> = crate::pytypeinfo::conv_option_arc_tstypeann,
});

pub struct ClassExprData {
    pub ident: Option<swc_core::ecma::ast::Ident>,
    pub class: Arc<crate::pyclass::ClassData>,
}

#[pyclass(extends=PyExpr)]
pub struct PyClassExpr {
    inner: Arc<ClassExprData>,
}

impl PyClassExpr {
    pub fn from_arc(inner: Arc<ClassExprData>) -> Self {
        PyClassExpr { inner }
    }
}

#[pymethods]
impl PyClassExpr {
    #[getter]
    fn ident(&self, py: Python<'_>) -> PyResult<Option<PyIdent>> {
        conv_option_ident(py, self.inner.ident.clone())
    }

    #[getter]
    fn class(&self, py: Python<'_>) -> PyResult<Py<PyClass>> {
        crate::pyclass::wrap_class_data(py, Arc::clone(&self.inner.class))
    }
}

ast_node_variant!(PyExpr, PyMetaPropExpr, MetaPropExpr, {
    span: PySpan = conv_span,
    kind: PyMetaPropKind = conv_meta_prop_kind
});

#[pyclass(extends=PyExpr)]
pub struct PyPrivateNameExpr {
    #[pyo3(get)]
    pub name: PyPrivateName,
}

impl PyPrivateNameExpr {
    pub fn build(py: Python<'_>, node: PrivateName) -> PyResult<Self> {
        Ok(PyPrivateNameExpr {
            name: crate::conversions::conv_private_name(py, node)?,
        })
    }
}

ast_node_variant!(PyExpr, PyInvalidExpr, Invalid, {
    span: PySpan = conv_span
});
