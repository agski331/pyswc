use pyo3::prelude::*;

use swc_core::common::Span;
use swc_core::ecma::ast::{
    ArrayLit, AssignExpr, AssignTarget, AssignTargetPat, AwaitExpr, BinExpr, BindingIdent,
    CallExpr, ClassExpr, CondExpr, Expr, ExprOrSpread, FnExpr, Invalid, Lit, MemberExpr,
    MetaPropExpr, NewExpr, ObjectLit, OptCall, OptChainExpr, ParenExpr, Pat, PrivateName, SeqExpr,
    SimpleAssignTarget, SuperPropExpr, TaggedTpl, ThisExpr, Tpl, TsAsExpr, TsConstAssertion,
    TsInstantiation, TsNonNullExpr, TsSatisfiesExpr, TsTypeAssertion, UnaryExpr, UpdateExpr,
    YieldExpr,
};

use crate::conversions::{
    conv_binary_op, conv_bindingident, conv_bool, conv_boxed_class, conv_boxed_expr,
    conv_boxed_function, conv_boxed_opt_chain_base, conv_boxed_tstype,
    conv_boxed_type_param_instantiation, conv_callee, conv_ctxt, conv_elems, conv_elems_noopt,
    conv_expr, conv_function, conv_ident, conv_member_prop, conv_meta_prop_kind,
    conv_option_boxed_expr, conv_option_function_body, conv_option_ident,
    conv_option_type_param_decl, conv_pat, conv_pats, conv_prop_or_spreads, conv_span, conv_super,
    conv_super_prop, conv_tpl_elements, conv_typeparams, conv_unary_op, conv_update_op,
};
use crate::macros::ast_node_variant;
use crate::pyclass::PyClass;
use crate::pyfunction::{PyCallee, PyFunction, PyFunctionBody};
use crate::pyident::PyIdent;
use crate::pyident::{PyBindingIdent, PyPrivateName};
use crate::pylit::PyLit;
use crate::pypat::PyPat;
use crate::pyprop::{PyMemberProp, PyPropOrSpread, PySuperProp};
use crate::pyspan::PySpan;
use crate::pytypeinfo::{PyTplElement, PyTsType, PyTsTypeParamDecl, PyTsTypeParamInstantiation};

#[derive(Clone)]
#[pyclass(subclass)]
pub struct PyExpr {
    pub expr: Expr,
}

ast_node_variant!(PyExpr, PyThisExpr, ThisExpr, {
    span: PySpan = conv_span,
});

ast_node_variant!(PyExpr, PyArrayLitExpr, ArrayLit, {
    span: PySpan = conv_span,
    elems: Vec<Option<Py<PyExpr>>> = conv_elems,
});

ast_node_variant!(PyExpr, PyObjectLit, ObjectLit, {
    span: PySpan = conv_span,
    props: Vec<Py<PyPropOrSpread>> = conv_prop_or_spreads
});

ast_node_variant!(PyExpr, PyCallExpr, CallExpr, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    callee: Py<PyCallee> = conv_callee,
    args: Vec<Py<PyExpr>> = conv_elems_noopt,
});

ast_node_variant!(PyExpr, PyFnExpr, FnExpr, {
    ident: Option<PyIdent> = conv_option_ident,
    function: Py<PyFunction> = conv_boxed_function
});

ast_node_variant!(PyExpr, PyUnaryExpr, UnaryExpr, {
    span: PySpan = conv_span,
    op: u32 = conv_unary_op,
    arg: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyExpr, PyUpdateExpr, UpdateExpr, {
    span: PySpan = conv_span,
    op: u32 = conv_update_op,
    prefix: bool = conv_bool,
    arg: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyExpr, PyBinExpr, BinExpr, {
    span: PySpan = conv_span,
    op: u32 = conv_binary_op,
    left: Py<PyExpr> = conv_boxed_expr,
    right: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyExpr, PyMemberExpr, MemberExpr, {
    span: PySpan = conv_span,
    obj: Py<PyExpr> = conv_boxed_expr,
    prop: Py<PyMemberProp> = conv_member_prop
});

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

ast_node_variant!(PyExpr, PyParenExpr, ParenExpr, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_boxed_expr
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
        let base = PyExpr {
            expr: Expr::Member(node.clone()),
        };
        let sub = PyMemberExpr::build(py, node)?;
        Ok(PyOptChainBaseMember {
            member: Py::new(py, (sub, base))?,
        })
    }
}

ast_node_variant!(PyOptChainBase, PyOptCall, OptCall, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    callee: Py<PyExpr> = conv_boxed_expr,
    args: Vec<Py<PyExpr>> = conv_elems_noopt,
    type_args: Option<Py<PyTsTypeParamInstantiation>> = conv_typeparams
});

ast_node_variant!(PyExpr, PyOptChainExpr, OptChainExpr, {
    span: PySpan = conv_span,
    optional: bool = conv_bool,
    base: Py<PyOptChainBase> = conv_boxed_opt_chain_base
});

ast_node_variant!(PyExpr, PyTsAsExpr, TsAsExpr, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_boxed_expr,
    type_ann: Py<PyTsType> = conv_boxed_tstype
});

ast_node_variant!(PyExpr, PyTsSatisfiesExpr, TsSatisfiesExpr, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_boxed_expr,
    type_ann: Py<PyTsType> = conv_boxed_tstype
});

ast_node_variant!(PyExpr, PyTsNonNullExpr, TsNonNullExpr, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyExpr, PyTsTypeAssertion, TsTypeAssertion, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_boxed_expr,
    type_ann: Py<PyTsType> = conv_boxed_tstype
});

ast_node_variant!(PyExpr, PyTsInstantiation, TsInstantiation, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_boxed_expr,
    type_args: Py<PyTsTypeParamInstantiation> = conv_boxed_type_param_instantiation
});

pub fn expr_to_py(py: Python<'_>, expr: Expr) -> PyResult<Py<PyExpr>> {
    conv_expr(py, expr)
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
                let base = PyExpr {
                    expr: Expr::$expr_variant(node.clone()),
                };
                let sub = $py_expr_ty::build(py, node)?;
                Ok($py_name {
                    $field: Py::new(py, (sub, base))?,
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
simple_assign_target_expr_variant!(
    PySimpleAssignTargetSuperProp,
    super_prop,
    PySuperPropExpr,
    SuperPropExpr,
    SuperProp
);
simple_assign_target_expr_variant!(
    PySimpleAssignTargetParen,
    paren,
    PyParenExpr,
    ParenExpr,
    Paren
);
simple_assign_target_expr_variant!(
    PySimpleAssignTargetOptChain,
    opt_chain,
    PyOptChainExpr,
    OptChainExpr,
    OptChain
);
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

ast_node_variant!(PyExpr, PyAssignExpr, AssignExpr, {
    span: PySpan = conv_span,
    op: u32 = crate::conversions::conv_assign_op,
    left: Py<PyAssignTarget> = conv_assign_target,
    right: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyExpr, PyCondExpr, CondExpr, {
    span: PySpan = conv_span,
    test: Py<PyExpr> = conv_boxed_expr,
    cons: Py<PyExpr> = conv_boxed_expr,
    alt: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyExpr, PyNewExpr, NewExpr, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    callee: Py<PyExpr> = conv_boxed_expr,
    args: Option<Vec<Py<crate::pyprop::PyExprOrSpread>>> = crate::conversions::conv_option_expr_or_spreads_noopt,
    type_args: Option<Py<PyTsTypeParamInstantiation>> = conv_typeparams
});

ast_node_variant!(PyExpr, PySeqExpr, SeqExpr, {
    span: PySpan = conv_span,
    exprs: Vec<Py<PyExpr>> = crate::conversions::conv_boxed_exprs
});

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

ast_node_variant!(PyExpr, PyTpl, Tpl, {
    span: PySpan = conv_span,
    exprs: Vec<Py<PyExpr>> = crate::conversions::conv_boxed_exprs,
    quasis: Vec<Py<PyTplElement>> = conv_tpl_elements
});

ast_node_variant!(PyExpr, PyTaggedTpl, TaggedTpl, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    tag: Py<PyExpr> = conv_boxed_expr,
    type_params: Option<Py<PyTsTypeParamInstantiation>> = conv_typeparams,
    tpl: Py<PyTpl> = crate::conversions::conv_boxed_tpl
});

#[pyclass]
pub struct PyArrowFunctionBody {
    #[pyo3(get)]
    pub function_body: Option<Py<PyFunctionBody>>,
    #[pyo3(get)]
    pub expr: Option<Py<PyExpr>>,
}

ast_node_variant!(PyExpr, PyArrowExpr, swc_core::ecma::ast::ArrowExpr, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    params: Vec<Py<PyPat>> = conv_pats,
    body: Py<PyArrowFunctionBody> = crate::conversions::conv_boxed_arrow_function_body,
    is_async: bool = conv_bool,
    is_generator: bool = conv_bool,
    type_params: Option<Py<PyTsTypeParamDecl>> = conv_option_type_param_decl,
    return_type: Option<Py<crate::pytypeinfo::PyTsTypeAnn>> = crate::conversions::conv_option_tstypeann
});

#[pyclass(extends=PyExpr)]
pub struct PyClassExpr {
    #[pyo3(get)]
    pub ident: Option<PyIdent>,
    #[pyo3(get)]
    pub class: Py<PyClass>,
}

impl PyClassExpr {
    pub fn build(py: Python<'_>, node: ClassExpr) -> PyResult<Self> {
        Ok(PyClassExpr {
            ident: conv_option_ident(py, node.ident)?,
            class: conv_boxed_class(py, node.class)?,
        })
    }
}

ast_node_variant!(PyExpr, PyYieldExpr, YieldExpr, {
    span: PySpan = conv_span,
    arg: Option<Py<PyExpr>> = conv_option_boxed_expr,
    delegate: bool = conv_bool
});

ast_node_variant!(PyExpr, PyMetaPropExpr, MetaPropExpr, {
    span: PySpan = conv_span,
    kind: u32 = conv_meta_prop_kind
});

ast_node_variant!(PyExpr, PyAwaitExpr, AwaitExpr, {
    span: PySpan = conv_span,
    arg: Py<PyExpr> = conv_boxed_expr
});

ast_node_variant!(PyExpr, PyTsConstAssertion, TsConstAssertion, {
    span: PySpan = conv_span,
    expr: Py<PyExpr> = conv_boxed_expr
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
