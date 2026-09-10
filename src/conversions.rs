use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use swc_core::ecma::visit::NodeRef::MemberProp;
use crate::pyexpr::expr_to_py;
use crate::pyfunction::{PyCallee, PyDecorator, PyFunction, PyFunctionBody, PyParam, PyTsThisParam};
use crate::pyident::{PyBindingIdent, PyPrivateName};
use crate::pyident::PyIdent;
use crate::pyident::PyIdentName;
use crate::pyspan::PySpan;
use crate::pystmts::PyStmt;
use crate::pystmts::PyBlockStmt;
use crate::pytypeinfo::PyTsKeywordType;
use crate::pytypeinfo::PyTsThisType;
use crate::pytypeinfo::PyTsType;
use crate::pytypeinfo::PyTsTypeAnn;
use crate::pytypeinfo::PyTsTypeRef;

use swc_core::ecma::ast::{*};
use swc_core::ecma::atoms::{Atom, Wtf8Atom};
use swc_core::common::{Span, SyntaxContext};
use crate::pystmts::stmt_to_py;
use crate::pypat::{
    PyArrayPat, PyAssignPat, PyAssignPatProp, PyBindingIdentPat, PyExprPat, PyInvalidPat,
    PyKeyValuePatProp, PyObjectPat, PyObjectPatProp, PyObjectPatRestProp, PyPat, PyRestPat,
};
use crate::pyprop::{
    PyAssignProp, PyBigIntPropName, PyComputedPropName, PyExprOrSpread, PyGetterProp, PyIdentPropName, PyKeyValueProp, PyMemberProp, PyMethodProp, PyNumPropName, PyProp, PyPropName, PyPropOrSpread, PyPropOrSpreadProp, PySetterProp, PyShorthandProp, PySpreadElement, PyStrPropName, PySuperProp,
};
use crate::pystmts::{PyCatchClause, PyForHead, PySwitchCase, PyVarDeclOrExpr};
use crate::pyexpr::*;
use crate::pytypeinfo::{
    PyTsArrayType, PyTsCallSignatureDecl, PyTsConditionalType, PyTsConstructSignatureDecl,
    PyTsConstructorType, PyTsEntityName, PyTsFnType, PyTsGetterSignature,
    PyTsImportCallOptions, PyTsImportType, PyTsIndexSignature, PyTsIndexedAccessType,
    PyTsInferType, PyTsIntersectionType, PyTsLit, PyTsLitBigInt, PyTsLitBool, PyTsLitNumber,
    PyTsLitStr, PyTsLitTpl, PyTsLitType, PyTsMappedType, PyTsMethodSignature,
    PyTsOptionalType, PyTsParenthesizedType, PyTsPropertySignature, PyTsQualifiedName,
    PyTsRestType, PyTsSetterSignature, PyTsThisTypeOrIdent, PyTsThisTypeOrIdentIdent,
    PyTsThisTypeOrIdentThis, PyTsTupleElement, PyTsTupleType, PyTsTypeElement, PyTsTypeLit,
    PyTsTypeOperator, PyTsTypeParam, PyTsTypeParamDecl, PyTsTypeParamInstantiation,
    PyTsTypeQuery, PyTsTypeQueryExpr, PyTsTypeQueryExprEntityName, PyTsTypeQueryExprImport,
    PyTsTypePredicate, PyTsUnionType, PyTplElement, PyTsExprWithTypeArgs,
};
use crate::pyclass::{
    PyAutoAccessor, PyClass, PyClassEmptyMember, PyClassIndexSignature, PyClassMember,
    PyClassMethod, PyClassProp, PyConstructor, PyKey, PyParamOrTsParamProp,
    PyParamOrTsParamPropParam, PyParamOrTsParamPropTsParamProp, PyPrivateMethod, PyPrivateProp,
    PyStaticBlock,
};
use crate::pydecl::*;
use crate::pymodule::*;
use crate::pyjsx::*;

pub fn conv_member_prop(py: Python<'_>, expr: swc_core::ecma::ast::MemberProp) -> PyResult<Py<PyMemberProp>>{
    if let swc_core::ecma::ast::MemberProp::Ident(id) = expr{
        let idname = conv_identname(py, id)?;
        return Py::new(py, PyMemberProp{ident: Some(idname), private_name: None, computed: None});
    }
    else if let swc_core::ecma::ast::MemberProp::PrivateName(p) = expr{
        let privname = conv_private_name(py, p)?;
        return Py::new(py, PyMemberProp { ident: None, private_name: Some(privname), computed: None });
    }
    else{
        if let swc_core::ecma::ast::MemberProp::Computed(c) = expr{
            let base = PyPropName {  };
            let prop_name = Py::new(py, (PyComputedPropName::build(py, c)?, base))?;
            return Py::new(py, PyMemberProp { ident: None, private_name: None, computed: Some(prop_name) });
        }
        return Err(PyValueError::new_err("Weird MemberProp conversion"));
    }
}

pub fn conv_unary_op(py: Python<'_>, op: UnaryOp) -> PyResult<u32>{
    return Ok(op as u32)
}

pub fn conv_private_name(py: Python<'_>, name: PrivateName) -> PyResult<PyPrivateName>{
    return Ok(PyPrivateName { span: conv_span(py, name.span)?, name: conv_atom(py, name.name)? })
}

pub fn conv_assign_op(py: Python<'_>, op: AssignOp) -> PyResult<u32>{
    return Ok(op as u32)
}

pub fn conv_update_op(py: Python<'_>, op: UpdateOp) -> PyResult<u32>{
    return Ok(op as u32)
}

pub fn conv_binary_op(py: Python<'_>, op: BinaryOp) -> PyResult<u32>{
    return Ok(op as u32)
}

pub fn conv_tsqualfiiedname(py: Python<'_>, name: TsQualifiedName) -> PyResult<PyTsQualifiedName>{
    return Ok(PyTsQualifiedName { span: conv_span(py, name.span)?, left: conv_tsentityname(py, name.left)?, right: conv_identname(py, name.right)? })
}

pub fn conv_identname(py: Python<'_>, name: IdentName) -> PyResult<PyIdentName>{
    return Ok(PyIdentName{
        span: conv_span(py, name.span)?,
        sym: name.sym.to_string()
    })
}

pub fn conv_ident(py: Python<'_>, ident: Ident) -> PyResult<PyIdent>{
    return Ok(PyIdent{
        span: conv_span(py, ident.span)?,
        ctxt: conv_ctxt(py, ident.ctxt)?,
        sym: ident.sym.to_string()
    })
}

pub fn conv_option_ident(py: Python<'_>, ident: Option<Ident>) -> PyResult<Option<PyIdent>>{
    if ident.is_none(){
        return Ok(None);
    }

    let id = ident.unwrap();
    let pyid = conv_ident(py, id)?;
    return Ok(Some(pyid));
}

pub fn conv_tsentityname(py: Python<'_>, name: TsEntityName) -> PyResult<Py<PyTsEntityName>>{
    if let TsEntityName::TsQualifiedName(name) = name{
        return Py::new(py, PyTsEntityName{
            ident: None,
            qualified_name: Some(Py::new(py, conv_tsqualfiiedname(py, *name)?)?)
        });
    }
    else
    {
        if !name.is_ident(){
            return Err(PyValueError::new_err("Cannot convert TsEntityName"));
        }
        return Py::new(py, PyTsEntityName{
            ident: Some(conv_ident(py, name.expect_ident())?),
            qualified_name: None
        });
    }
}

pub fn conv_tstypeann(py: Python<'_>, ann: TsTypeAnn) -> PyResult<PyTsTypeAnn>{
    return Ok(PyTsTypeAnn{
        span: conv_span(py, ann.span)?,
        type_ann: conv_tstype(py, *ann.type_ann)?
    })
}

pub fn conv_tstype(py: Python<'_>, ts_type: TsType) -> PyResult<Py<PyTsType>>{
    let base = PyTsType { };
    Ok(match ts_type {
        TsType::TsKeywordType(e) => Py::new(py, (PyTsKeywordType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsThisType(e) => Py::new(py, (PyTsThisType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsFnOrConstructorType(TsFnOrConstructorType::TsFnType(e)) => Py::new(py, (PyTsFnType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsFnOrConstructorType(TsFnOrConstructorType::TsConstructorType(e)) => Py::new(py, (PyTsConstructorType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsTypeRef(e) => Py::new(py, (PyTsTypeRef::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsTypeQuery(e) => Py::new(py, (PyTsTypeQuery::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsTypeLit(e) => Py::new(py, (PyTsTypeLit::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsArrayType(e) => Py::new(py, (PyTsArrayType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsTupleType(e) => Py::new(py, (PyTsTupleType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsOptionalType(e) => Py::new(py, (PyTsOptionalType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsRestType(e) => Py::new(py, (PyTsRestType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(e)) => Py::new(py, (PyTsUnionType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsIntersectionType(e)) => Py::new(py, (PyTsIntersectionType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsConditionalType(e) => Py::new(py, (PyTsConditionalType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsInferType(e) => Py::new(py, (PyTsInferType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsParenthesizedType(e) => Py::new(py, (PyTsParenthesizedType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsTypeOperator(e) => Py::new(py, (PyTsTypeOperator::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsIndexedAccessType(e) => Py::new(py, (PyTsIndexedAccessType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsMappedType(e) => Py::new(py, (PyTsMappedType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsLitType(e) => Py::new(py, (PyTsLitType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsTypePredicate(e) => Py::new(py, (PyTsTypePredicate::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsType::TsImportType(e) => Py::new(py, (PyTsImportType::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_tstypes(py: Python<'_>, ts_types: Vec<Box<TsType>>) -> PyResult<Vec<Py<PyTsType>>>{
    ts_types
        .into_iter()
        .map(|eos| conv_tstype(py, *eos))
        .collect()
}

pub fn conv_typeparams(py: Python<'_>, type_params: Option<Box<TsTypeParamInstantiation>>) -> PyResult<Option<Py<PyTsTypeParamInstantiation>>>{
    if type_params.is_none(){
        return Ok(None);
    }

    let tp = type_params.unwrap();

    return Ok(Some(Py::new(py, PyTsTypeParamInstantiation{
        span: conv_span(py, tp.span)?,
        params: conv_tstypes(py, tp.params)?
    })?))
}

pub fn conv_tskeywordtypekind(_py: Python<'_>, kind: TsKeywordTypeKind) -> PyResult<u32>{
    Ok(kind as u32)
}

pub fn conv_span(_py: Python<'_>, span: Span) -> PyResult<PySpan> {
    Ok(span.into())
}

pub fn conv_elems(py: Python<'_>, elems: Vec<Option<ExprOrSpread>>) -> PyResult<Vec<Option<Py<PyExpr>>>> {
    elems
        .into_iter()
        .map(|opt| opt.map(|eos| expr_to_py(py, *eos.expr)).transpose())
        .collect()
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

pub fn conv_stmts(py: Python<'_>, stmts: Vec<Stmt>) -> PyResult<Vec<Py<PyStmt>>> {
    stmts.into_iter().map(|s| stmt_to_py(py, s)).collect()
}

pub fn conv_block_stmt(py: Python<'_>, stmt: BlockStmt) -> PyResult<Py<PyBlockStmt>>{
    let base = PyStmt { stmt: Stmt::Block(stmt.clone()) };
    let sub = PyBlockStmt::build(py, stmt)?;
    Py::new(py, (sub, base))
}

pub fn conv_option_block_stmt(py: Python<'_>, stmt: Option<BlockStmt>) -> PyResult<Option<Py<PyBlockStmt>>>{
    if stmt.is_none(){
        return Ok(None);
    }

    return Ok(Some(conv_block_stmt(py, stmt.expect("Internal Error"))?))
}

pub fn conv_option_pat(py: Python<'_>, pat: Option<Pat>) -> PyResult<Option<Py<PyPat>>>{
    pat.map(|p| conv_pat(py, p)).transpose()
}

pub fn conv_pat_elems(py: Python<'_>, elems: Vec<Option<Pat>>) -> PyResult<Vec<Option<Py<PyPat>>>>{
    elems
        .into_iter()
        .map(|opt| opt.map(|p| conv_pat(py, p)).transpose())
        .collect()
}

pub fn conv_boxed_pat(py: Python<'_>, pat: Box<Pat>) -> PyResult<Py<PyPat>>{
    conv_pat(py, *pat)
}

pub fn conv_pat(py: Python<'_>, pat: Pat) -> PyResult<Py<PyPat>>{
    let base = PyPat { };
    Ok(match pat {
        Pat::Ident(p) => Py::new(py, (PyBindingIdentPat::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        Pat::Array(p) => Py::new(py, (PyArrayPat::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        Pat::Rest(p) => Py::new(py, (PyRestPat::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        Pat::Object(p) => Py::new(py, (PyObjectPat::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        Pat::Assign(p) => Py::new(py, (PyAssignPat::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        Pat::Invalid(p) => Py::new(py, (PyInvalidPat::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        Pat::Expr(p) => Py::new(py, (PyExprPat::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_bindingident(py: Python<'_>, ident: BindingIdent) -> PyResult<Py<PyBindingIdent>>{
    Py::new(py, PyBindingIdent{
        id: conv_ident(py, ident.id)?,
        type_ann: conv_option_tstypeann(py, ident.type_ann)?
    })
}

pub fn conv_option_tstypeann(py: Python<'_>, ann: Option<Box<TsTypeAnn>>) -> PyResult<Option<Py<PyTsTypeAnn>>>{
    match ann {
        None => Ok(None),
        Some(a) => Ok(Some(Py::new(py, conv_tstypeann(py, *a)?)?))
    }
}

pub fn conv_option_boxed_expr(py: Python<'_>, expr: Option<Box<Expr>>) -> PyResult<Option<Py<PyExpr>>>{
    expr.map(|e| conv_boxed_expr(py, e)).transpose()
}

pub fn conv_bool(_py: Python<'_>, value: bool) -> PyResult<bool>{
    Ok(value)
}

pub fn conv_f64(_py: Python<'_>, value: f64) -> PyResult<f64>{
    Ok(value)
}

pub fn conv_atom(_py: Python<'_>, atom: Atom) -> PyResult<String>{
    Ok(atom.to_string())
}

pub fn conv_wtf8atom(_py: Python<'_>, atom: Wtf8Atom) -> PyResult<String>{
    Ok(atom.to_atom_lossy().to_string())
}

pub fn conv_bigint_value(_py: Python<'_>, value: Box<BigIntValue>) -> PyResult<num_bigint::BigInt>{
    Ok((*value).clone())
}

pub fn conv_propname(py: Python<'_>, name: PropName) -> PyResult<Py<PyPropName>>{
    let base = PyPropName { };
    Ok(match name {
        PropName::Ident(n) => Py::new(py, (PyIdentPropName::build(py, n)?, base))?.into_bound(py).into_super().unbind(),
        PropName::Str(n) => Py::new(py, (PyStrPropName::build(py, n)?, base))?.into_bound(py).into_super().unbind(),
        PropName::Num(n) => Py::new(py, (PyNumPropName::build(py, n)?, base))?.into_bound(py).into_super().unbind(),
        PropName::Computed(n) => Py::new(py, (PyComputedPropName::build(py, n)?, base))?.into_bound(py).into_super().unbind(),
        PropName::BigInt(n) => Py::new(py, (PyBigIntPropName::build(py, n)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_object_pat_prop(py: Python<'_>, prop: ObjectPatProp) -> PyResult<Py<PyObjectPatProp>>{
    let base = PyObjectPatProp { };
    Ok(match prop {
        ObjectPatProp::KeyValue(p) => Py::new(py, (PyKeyValuePatProp::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        ObjectPatProp::Assign(p) => Py::new(py, (PyAssignPatProp::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        ObjectPatProp::Rest(p) => Py::new(py, (PyObjectPatRestProp::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_object_pat_props(py: Python<'_>, props: Vec<ObjectPatProp>) -> PyResult<Vec<Py<PyObjectPatProp>>>{
    props.into_iter().map(|p| conv_object_pat_prop(py, p)).collect()
}

pub fn conv_option_catch_clause(py: Python<'_>, handler: Option<CatchClause>) -> PyResult<Option<Py<PyCatchClause>>>{
    if handler.is_none(){
        return Ok(None);
    }
    let catch = handler.expect("Internal Error");
    Ok(
        Some(Py::new(py, PyCatchClause{
            span: conv_span(py, catch.span)?,
            param: conv_option_pat(py, catch.param)?,
            body: conv_block_stmt(py, catch.body)?
        })?)
    )
}

pub fn conv_boxed_expr(py: Python<'_>, expr: Box<Expr>) -> PyResult<Py<PyExpr>>{
    conv_expr(py, *expr)
}

pub fn conv_expr(py: Python<'_>, expr: Expr) -> PyResult<Py<PyExpr>>{
    let base = PyExpr { expr: expr.clone() };
    Ok(match expr {
        Expr::This(e) => Py::new(py, (PyThisExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Array(e) => Py::new(py, (PyArrayLitExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Object(e) => Py::new(py, (PyObjectLit::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Fn(e) => Py::new(py, (PyFnExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Unary(e) => Py::new(py, (PyUnaryExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Update(e) => Py::new(py, (PyUpdateExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Bin(e) => Py::new(py, (PyBinExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Assign(e) => Py::new(py, (PyAssignExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Member(e) => Py::new(py, (PyMemberExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::SuperProp(e) => Py::new(py, (PySuperPropExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Cond(e) => Py::new(py, (PyCondExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Call(e) => Py::new(py, (PyCallExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::New(e) => Py::new(py, (PyNewExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Seq(e) => Py::new(py, (PySeqExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Ident(e) => Py::new(py, (PyIdentExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Lit(e) => Py::new(py, (PyLitExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Tpl(e) => Py::new(py, (PyTpl::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::TaggedTpl(e) => Py::new(py, (PyTaggedTpl::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Arrow(e) => Py::new(py, (PyArrowExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Class(e) => Py::new(py, (PyClassExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Yield(e) => Py::new(py, (PyYieldExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::MetaProp(e) => Py::new(py, (PyMetaPropExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Await(e) => Py::new(py, (PyAwaitExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Paren(e) => Py::new(py, (PyParenExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::JSXMember(e) => Py::new(py, (PyJSXMemberExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::JSXNamespacedName(e) => Py::new(py, (PyJSXNamespacedName::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::JSXEmpty(e) => Py::new(py, (PyJSXEmptyExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::JSXElement(e) => Py::new(py, (PyJSXElement::build(py, *e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::JSXFragment(e) => Py::new(py, (PyJSXFragment::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::TsTypeAssertion(e) => Py::new(py, (PyTsTypeAssertion::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::TsConstAssertion(e) => Py::new(py, (PyTsConstAssertion::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::TsNonNull(e) => Py::new(py, (PyTsNonNullExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::TsAs(e) => Py::new(py, (PyTsAsExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::TsInstantiation(e) => Py::new(py, (PyTsInstantiation::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::TsSatisfies(e) => Py::new(py, (PyTsSatisfiesExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::PrivateName(e) => Py::new(py, (PyPrivateNameExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::OptChain(e) => Py::new(py, (PyOptChainExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        Expr::Invalid(e) => Py::new(py, (PyInvalidExpr::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_callee(py: Python<'_>, callee: Callee) -> PyResult<Py<PyCallee>>{
    if callee.is_super_(){
        return Py::new(py, PyCallee{
            span: Some(conv_span(py, callee.expect_super_().span)?), is_super: true, phase: None, expr: None});
    }
    else
    {
        if callee.is_expr(){
            return Py::new(py, PyCallee{
                span: None, is_super: false, phase: None, expr: Some(conv_boxed_expr(py, callee.expect_expr())?)});
        }
        else{
            let import = callee.expect_import();
            return Py::new(py, PyCallee { span: Some(conv_span(py, import.span)?), is_super: false, phase: Some(import.phase), expr: None });
        }
    }
}

pub fn conv_boxed_tstype(py: Python<'_>, ts_type: Box<TsType>) -> PyResult<Py<PyTsType>>{
    conv_tstype(py, *ts_type)
}

pub fn conv_option_boxed_tstype(py: Python<'_>, ts_type: Option<Box<TsType>>) -> PyResult<Option<Py<PyTsType>>>{
    ts_type.map(|t| conv_boxed_tstype(py, t)).transpose()
}

pub fn conv_boxed_tstypeann(py: Python<'_>, ann: Box<TsTypeAnn>) -> PyResult<Py<PyTsTypeAnn>>{
    Py::new(py, conv_tstypeann(py, *ann)?)
}

pub fn conv_str(_py: Python<'_>, s: Str) -> PyResult<String>{
    Ok(s.value.to_atom_lossy().to_string())
}

pub fn conv_boxed_object_lit(py: Python<'_>, lit: Box<ObjectLit>) -> PyResult<Py<PyExpr>>{
    conv_expr(py, Expr::Object(*lit))
}

pub fn conv_option_entity_name(py: Python<'_>, name: Option<TsEntityName>) -> PyResult<Option<Py<PyTsEntityName>>>{
    match name {
        None => Ok(None),
        Some(n) => Ok(Some(conv_tsentityname(py, n)?))
    }
}

pub fn conv_option_import_call_options(py: Python<'_>, opts: Option<TsImportCallOptions>) -> PyResult<Option<Py<PyTsImportCallOptions>>>{
    match opts {
        None => Ok(None),
        Some(o) => Ok(Some(Py::new(py, PyTsImportCallOptions{
            span: conv_span(py, o.span)?,
            with: conv_boxed_object_lit(py, o.with)?
        })?))
    }
}

pub fn conv_ts_import_type(py: Python<'_>, node: TsImportType) -> PyResult<Py<PyTsImportType>>{
    Py::new(py, (PyTsImportType::build(py, node)?, PyTsType { }))
}

pub fn conv_ts_type_query_expr(py: Python<'_>, expr: TsTypeQueryExpr) -> PyResult<Py<PyTsTypeQueryExpr>>{
    let base = PyTsTypeQueryExpr { };
    Ok(match expr {
        TsTypeQueryExpr::TsEntityName(n) => Py::new(py, (PyTsTypeQueryExprEntityName::build(py, n)?, base))?.into_bound(py).into_super().unbind(),
        TsTypeQueryExpr::Import(i) => Py::new(py, (PyTsTypeQueryExprImport::build(py, i)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_fn_param(py: Python<'_>, param: TsFnParam) -> PyResult<Py<PyPat>>{
    let base = PyPat { };
    Ok(match param {
        TsFnParam::Ident(p) => Py::new(py, (PyBindingIdentPat::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        TsFnParam::Array(p) => Py::new(py, (PyArrayPat::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        TsFnParam::Rest(p) => Py::new(py, (PyRestPat::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        TsFnParam::Object(p) => Py::new(py, (PyObjectPat::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_fn_params(py: Python<'_>, params: Vec<TsFnParam>) -> PyResult<Vec<Py<PyPat>>>{
    params.into_iter().map(|p| conv_fn_param(py, p)).collect()
}

pub fn conv_type_param(py: Python<'_>, param: TsTypeParam) -> PyResult<Py<PyTsTypeParam>>{
    Py::new(py, PyTsTypeParam{
        span: conv_span(py, param.span)?,
        name: conv_ident(py, param.name)?,
        is_in: param.is_in,
        is_out: param.is_out,
        is_const: param.is_const,
        constraint: conv_option_boxed_tstype(py, param.constraint)?,
        default: conv_option_boxed_tstype(py, param.default)?
    })
}

pub fn conv_type_params(py: Python<'_>, params: Vec<TsTypeParam>) -> PyResult<Vec<Py<PyTsTypeParam>>>{
    params.into_iter().map(|p| conv_type_param(py, p)).collect()
}

pub fn conv_type_param_decl(py: Python<'_>, decl: TsTypeParamDecl) -> PyResult<Py<PyTsTypeParamDecl>>{
    Py::new(py, PyTsTypeParamDecl{
        span: conv_span(py, decl.span)?,
        params: conv_type_params(py, decl.params)?
    })
}

pub fn conv_option_type_param_decl(py: Python<'_>, decl: Option<Box<TsTypeParamDecl>>) -> PyResult<Option<Py<PyTsTypeParamDecl>>>{
    match decl {
        None => Ok(None),
        Some(d) => Ok(Some(conv_type_param_decl(py, *d)?))
    }
}

pub fn conv_ts_type_element(py: Python<'_>, elem: TsTypeElement) -> PyResult<Py<PyTsTypeElement>>{
    let base = PyTsTypeElement { };
    Ok(match elem {
        TsTypeElement::TsCallSignatureDecl(e) => Py::new(py, (PyTsCallSignatureDecl::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsTypeElement::TsConstructSignatureDecl(e) => Py::new(py, (PyTsConstructSignatureDecl::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsTypeElement::TsPropertySignature(e) => Py::new(py, (PyTsPropertySignature::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsTypeElement::TsGetterSignature(e) => Py::new(py, (PyTsGetterSignature::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsTypeElement::TsSetterSignature(e) => Py::new(py, (PyTsSetterSignature::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsTypeElement::TsMethodSignature(e) => Py::new(py, (PyTsMethodSignature::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        TsTypeElement::TsIndexSignature(e) => Py::new(py, (PyTsIndexSignature::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_ts_type_elements(py: Python<'_>, elems: Vec<TsTypeElement>) -> PyResult<Vec<Py<PyTsTypeElement>>>{
    elems.into_iter().map(|e| conv_ts_type_element(py, e)).collect()
}

pub fn conv_tuple_element(py: Python<'_>, elem: TsTupleElement) -> PyResult<Py<PyTsTupleElement>>{
    Py::new(py, PyTsTupleElement{
        span: conv_span(py, elem.span)?,
        label: conv_option_pat(py, elem.label)?,
        ty: conv_boxed_tstype(py, elem.ty)?
    })
}

pub fn conv_tuple_elements(py: Python<'_>, elems: Vec<TsTupleElement>) -> PyResult<Vec<Py<PyTsTupleElement>>>{
    elems.into_iter().map(|e| conv_tuple_element(py, e)).collect()
}

pub fn conv_option_true_plus_minus(_py: Python<'_>, v: Option<TruePlusMinus>) -> PyResult<Option<u32>>{
    Ok(v.map(|t| t as u32))
}

pub fn conv_ts_type_operator_op(_py: Python<'_>, op: TsTypeOperatorOp) -> PyResult<u32>{
    Ok(op as u32)
}

pub fn conv_ts_lit(py: Python<'_>, lit: TsLit) -> PyResult<Py<PyTsLit>>{
    let base = PyTsLit { };
    Ok(match lit {
        TsLit::Number(n) => Py::new(py, (PyTsLitNumber::build(py, n)?, base))?.into_bound(py).into_super().unbind(),
        TsLit::Str(s) => Py::new(py, (PyTsLitStr::build(py, s)?, base))?.into_bound(py).into_super().unbind(),
        TsLit::Bool(b) => Py::new(py, (PyTsLitBool::build(py, b)?, base))?.into_bound(py).into_super().unbind(),
        TsLit::BigInt(b) => Py::new(py, (PyTsLitBigInt::build(py, b)?, base))?.into_bound(py).into_super().unbind(),
        TsLit::Tpl(t) => Py::new(py, (PyTsLitTpl::build(py, t)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_tpl_element(py: Python<'_>, el: TplElement) -> PyResult<Py<PyTplElement>>{
    Py::new(py, PyTplElement{
        span: conv_span(py, el.span)?,
        tail: el.tail,
        cooked: el.cooked.map(|a| a.to_atom_lossy().to_string()),
        raw: el.raw.to_string()
    })
}

pub fn conv_tpl_elements(py: Python<'_>, els: Vec<TplElement>) -> PyResult<Vec<Py<PyTplElement>>>{
    els.into_iter().map(|e| conv_tpl_element(py, e)).collect()
}

pub fn conv_ts_this_type_or_ident(py: Python<'_>, node: TsThisTypeOrIdent) -> PyResult<Py<PyTsThisTypeOrIdent>>{
    let base = PyTsThisTypeOrIdent { };
    Ok(match node {
        TsThisTypeOrIdent::TsThisType(t) => Py::new(py, (PyTsThisTypeOrIdentThis::build(py, t)?, base))?.into_bound(py).into_super().unbind(),
        TsThisTypeOrIdent::Ident(i) => Py::new(py, (PyTsThisTypeOrIdentIdent::build(py, i)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_decorator(py: Python<'_>, dec: Decorator) -> PyResult<Py<PyDecorator>>{
    Py::new(py, PyDecorator{
        span: conv_span(py, dec.span)?,
        expr: conv_boxed_expr(py, dec.expr)?
    })
}

pub fn conv_decorators(py: Python<'_>, decs: Vec<Decorator>) -> PyResult<Vec<Py<PyDecorator>>>{
    decs.into_iter().map(|d| conv_decorator(py, d)).collect()
}

pub fn conv_param(py: Python<'_>, param: Param) -> PyResult<Py<PyParam>>{
    Py::new(py, PyParam{
        span: conv_span(py, param.span)?,
        decorators: conv_decorators(py, param.decorators)?,
        pat: conv_pat(py, param.pat)?
    })
}

pub fn conv_params(py: Python<'_>, params: Vec<Param>) -> PyResult<Vec<Py<PyParam>>>{
    params.into_iter().map(|p| conv_param(py, p)).collect()
}

pub fn conv_ts_this_param(py: Python<'_>, node: TsThisParam) -> PyResult<Py<PyTsThisParam>>{
    Py::new(py, PyTsThisParam{
        span: conv_span(py, node.span)?,
        this_span: conv_span(py, node.this_span)?,
        type_ann: conv_option_tstypeann(py, node.type_ann)?
    })
}

pub fn conv_option_boxed_ts_this_param(py: Python<'_>, node: Option<Box<TsThisParam>>) -> PyResult<Option<Py<PyTsThisParam>>>{
    match node {
        None => Ok(None),
        Some(n) => Ok(Some(conv_ts_this_param(py, *n)?))
    }
}

pub fn conv_function_body(py: Python<'_>, body: FunctionBody) -> PyResult<Py<PyFunctionBody>>{
    Py::new(py, PyFunctionBody{
        span: conv_span(py, body.span)?,
        stmts: conv_stmts(py, body.stmts)?
    })
}

pub fn conv_option_function_body(py: Python<'_>, body: Option<FunctionBody>) -> PyResult<Option<Py<PyFunctionBody>>>{
    match body {
        None => Ok(None),
        Some(b) => Ok(Some(conv_function_body(py, b)?))
    }
}

pub fn conv_function(py: Python<'_>, func: Function) -> PyResult<Py<PyFunction>>{
    Py::new(py, PyFunction{
        this_param: conv_option_boxed_ts_this_param(py, func.this_param)?,
        params: conv_params(py, func.params)?,
        decorators: conv_decorators(py, func.decorators)?,
        span: conv_span(py, func.span)?,
        ctxt: conv_ctxt(py, func.ctxt)?,
        body: conv_option_function_body(py, func.body)?,
        is_generator: func.is_generator,
        is_async: func.is_async,
        type_params: conv_option_type_param_decl(py, func.type_params)?,
        return_type: conv_option_tstypeann(py, func.return_type)?
    })
}

pub fn conv_boxed_function(py: Python<'_>, func: Box<Function>) -> PyResult<Py<PyFunction>>{
    conv_function(py, *func)
}

pub fn conv_prop(py: Python<'_>, prop: Prop) -> PyResult<Py<PyProp>>{
    let base = PyProp { };
    Ok(match prop {
        Prop::Shorthand(i) => Py::new(py, (PyShorthandProp::build(py, i)?, base))?.into_bound(py).into_super().unbind(),
        Prop::KeyValue(p) => Py::new(py, (PyKeyValueProp::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        Prop::Assign(p) => Py::new(py, (PyAssignProp::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        Prop::Getter(p) => Py::new(py, (PyGetterProp::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        Prop::Setter(p) => Py::new(py, (PySetterProp::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        Prop::Method(p) => Py::new(py, (PyMethodProp::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_boxed_prop(py: Python<'_>, prop: Box<Prop>) -> PyResult<Py<PyProp>>{
    conv_prop(py, *prop)
}

pub fn conv_prop_or_spread(py: Python<'_>, node: PropOrSpread) -> PyResult<Py<PyPropOrSpread>>{
    let base = PyPropOrSpread { };
    Ok(match node {
        PropOrSpread::Spread(s) => Py::new(py, (PySpreadElement::build(py, s)?, base))?.into_bound(py).into_super().unbind(),
        PropOrSpread::Prop(p) => Py::new(py, (PyPropOrSpreadProp::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_prop_or_spreads(py: Python<'_>, nodes: Vec<PropOrSpread>) -> PyResult<Vec<Py<PyPropOrSpread>>>{
    nodes.into_iter().map(|n| conv_prop_or_spread(py, n)).collect()
}

pub fn conv_expr_or_spread(py: Python<'_>, eos: ExprOrSpread) -> PyResult<Py<PyExprOrSpread>>{
    Py::new(py, PyExprOrSpread{
        spread: eos.spread.map(|s| conv_span(py, s)).transpose()?,
        expr: conv_boxed_expr(py, eos.expr)?
    })
}

pub fn conv_expr_or_spreads(py: Python<'_>, elems: Vec<Option<ExprOrSpread>>) -> PyResult<Vec<Option<Py<PyExprOrSpread>>>>{
    elems.into_iter().map(|opt| opt.map(|eos| conv_expr_or_spread(py, eos)).transpose()).collect()
}

pub fn conv_expr_or_spreads_noopt(py: Python<'_>, elems: Vec<ExprOrSpread>) -> PyResult<Vec<Py<PyExprOrSpread>>>{
    elems.into_iter().map(|eos| conv_expr_or_spread(py, eos)).collect()
}

pub fn conv_super(py: Python<'_>, node: Super) -> PyResult<PySuper>{
    Ok(PySuper { span: conv_span(py, node.span)? })
}

pub fn conv_super_prop(py: Python<'_>, prop: SuperProp) -> PyResult<Py<PySuperProp>>{
    match prop {
        SuperProp::Ident(id) => {
            let idname = conv_identname(py, id)?;
            Py::new(py, PySuperProp { ident: Some(idname), computed: None })
        },
        SuperProp::Computed(c) => {
            let base = PyPropName { };
            let prop_name = Py::new(py, (PyComputedPropName::build(py, c)?, base))?;
            Py::new(py, PySuperProp { ident: None, computed: Some(prop_name) })
        }
    }
}

pub fn conv_opt_chain_base(py: Python<'_>, base: OptChainBase) -> PyResult<Py<PyOptChainBase>>{
    let b = PyOptChainBase { };
    Ok(match base {
        OptChainBase::Member(m) => Py::new(py, (PyOptChainBaseMember::build(py, m)?, b))?.into_bound(py).into_super().unbind(),
        OptChainBase::Call(c) => Py::new(py, (PyOptCall::build(py, c)?, b))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_boxed_opt_chain_base(py: Python<'_>, base: Box<OptChainBase>) -> PyResult<Py<PyOptChainBase>>{
    conv_opt_chain_base(py, *base)
}

pub fn conv_boxed_type_param_instantiation(py: Python<'_>, type_args: Box<TsTypeParamInstantiation>) -> PyResult<Py<PyTsTypeParamInstantiation>>{
    let tp = *type_args;
    Py::new(py, PyTsTypeParamInstantiation{
        span: conv_span(py, tp.span)?,
        params: conv_tstypes(py, tp.params)?
    })
}

pub fn conv_option_accessibility(_py: Python<'_>, acc: Option<Accessibility>) -> PyResult<Option<u32>>{
    Ok(acc.map(|a| a as u32))
}

pub fn conv_method_kind(_py: Python<'_>, kind: MethodKind) -> PyResult<u32>{
    Ok(kind as u32)
}

pub fn conv_key(py: Python<'_>, key: Key) -> PyResult<Py<PyKey>>{
    match key {
        Key::Private(p) => Py::new(py, PyKey { private: Some(conv_private_name(py, p)?), public: None }),
        Key::Public(p) => Py::new(py, PyKey { private: None, public: Some(conv_propname(py, p)?) }),
    }
}

pub fn conv_ts_expr_with_type_args(py: Python<'_>, node: TsExprWithTypeArgs) -> PyResult<Py<PyTsExprWithTypeArgs>>{
    Py::new(py, PyTsExprWithTypeArgs{
        span: conv_span(py, node.span)?,
        expr: conv_boxed_expr(py, node.expr)?,
        type_args: conv_typeparams(py, node.type_args)?
    })
}

pub fn conv_ts_expr_with_type_args_vec(py: Python<'_>, nodes: Vec<TsExprWithTypeArgs>) -> PyResult<Vec<Py<PyTsExprWithTypeArgs>>>{
    nodes.into_iter().map(|n| conv_ts_expr_with_type_args(py, n)).collect()
}

pub fn conv_class_member(py: Python<'_>, member: ClassMember) -> PyResult<Py<PyClassMember>>{
    let base = PyClassMember { };
    Ok(match member {
        ClassMember::Constructor(c) => Py::new(py, (PyConstructor::build(py, c)?, base))?.into_bound(py).into_super().unbind(),
        ClassMember::Method(m) => Py::new(py, (PyClassMethod::build(py, m)?, base))?.into_bound(py).into_super().unbind(),
        ClassMember::PrivateMethod(m) => Py::new(py, (PyPrivateMethod::build(py, m)?, base))?.into_bound(py).into_super().unbind(),
        ClassMember::ClassProp(p) => Py::new(py, (PyClassProp::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        ClassMember::PrivateProp(p) => Py::new(py, (PyPrivateProp::build(py, p)?, base))?.into_bound(py).into_super().unbind(),
        ClassMember::TsIndexSignature(s) => Py::new(py, (PyClassIndexSignature::build(py, s)?, base))?.into_bound(py).into_super().unbind(),
        ClassMember::Empty(e) => Py::new(py, (PyClassEmptyMember::build(py, e)?, base))?.into_bound(py).into_super().unbind(),
        ClassMember::StaticBlock(s) => Py::new(py, (PyStaticBlock::build(py, s)?, base))?.into_bound(py).into_super().unbind(),
        ClassMember::AutoAccessor(a) => Py::new(py, (PyAutoAccessor::build(py, a)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_class_members(py: Python<'_>, members: Vec<ClassMember>) -> PyResult<Vec<Py<PyClassMember>>>{
    members.into_iter().map(|m| conv_class_member(py, m)).collect()
}

pub fn conv_class(py: Python<'_>, class: Class) -> PyResult<Py<PyClass>>{
    Py::new(py, PyClass{
        span: conv_span(py, class.span)?,
        ctxt: conv_ctxt(py, class.ctxt)?,
        decorators: conv_decorators(py, class.decorators)?,
        body: conv_class_members(py, class.body)?,
        super_class: conv_option_boxed_expr(py, class.super_class)?,
        is_abstract: class.is_abstract,
        type_params: conv_option_type_param_decl(py, class.type_params)?,
        super_type_params: conv_typeparams(py, class.super_type_params)?,
        implements: conv_ts_expr_with_type_args_vec(py, class.implements)?
    })
}

pub fn conv_boxed_class(py: Python<'_>, class: Box<Class>) -> PyResult<Py<PyClass>>{
    conv_class(py, *class)
}

pub fn conv_var_decl_kind(_py: Python<'_>, kind: VarDeclKind) -> PyResult<u32>{
    Ok(kind as u32)
}

pub fn conv_var_declarator(py: Python<'_>, node: VarDeclarator) -> PyResult<Py<PyVarDeclarator>>{
    Py::new(py, PyVarDeclarator{
        span: conv_span(py, node.span)?,
        name: conv_pat(py, node.name)?,
        init: conv_option_boxed_expr(py, node.init)?,
        definite: node.definite
    })
}

pub fn conv_var_declarators(py: Python<'_>, nodes: Vec<VarDeclarator>) -> PyResult<Vec<Py<PyVarDeclarator>>>{
    nodes.into_iter().map(|n| conv_var_declarator(py, n)).collect()
}

pub fn conv_ts_interface_body(py: Python<'_>, node: TsInterfaceBody) -> PyResult<Py<PyTsInterfaceBody>>{
    Py::new(py, PyTsInterfaceBody{
        span: conv_span(py, node.span)?,
        body: conv_ts_type_elements(py, node.body)?
    })
}

pub fn conv_ts_enum_member(py: Python<'_>, node: TsEnumMember) -> PyResult<Py<PyTsEnumMember>>{
    let (id_ident, id_str_span, id_str_value) = match node.id {
        TsEnumMemberId::Ident(i) => (Some(conv_ident(py, i)?), None, None),
        TsEnumMemberId::Str(s) => (None, Some(conv_span(py, s.span)?), Some(conv_str(py, s)?)),
    };
    Py::new(py, PyTsEnumMember{
        span: conv_span(py, node.span)?,
        id_ident,
        id_str_span,
        id_str_value,
        init: conv_option_boxed_expr(py, node.init)?
    })
}

pub fn conv_ts_enum_members(py: Python<'_>, nodes: Vec<TsEnumMember>) -> PyResult<Vec<Py<PyTsEnumMember>>>{
    nodes.into_iter().map(|n| conv_ts_enum_member(py, n)).collect()
}

pub fn conv_ts_module_name(py: Python<'_>, name: TsModuleName) -> PyResult<PyTsModuleName>{
    match name {
        TsModuleName::Ident(i) => Ok(PyTsModuleName{ ident: Some(conv_ident(py, i)?), str_span: None, str_value: None }),
        TsModuleName::Str(s) => Ok(PyTsModuleName{ ident: None, str_span: Some(conv_span(py, s.span)?), str_value: Some(conv_str(py, s)?) }),
    }
}

pub fn conv_ts_namespace_body(py: Python<'_>, body: TsNamespaceBody) -> PyResult<Py<PyTsNamespaceBody>>{
    let base = PyTsNamespaceBody { };
    Ok(match body {
        TsNamespaceBody::TsModuleBlock(b) => Py::new(py, (PyTsModuleBlock::build(py, b)?, base))?.into_bound(py).into_super().unbind(),
        TsNamespaceBody::TsNamespaceDecl(d) => Py::new(py, (PyTsNamespaceDecl::build(py, d)?, base))?.into_bound(py).into_super().unbind(),
    })
}

pub fn conv_option_ts_namespace_body(py: Python<'_>, body: Option<TsNamespaceBody>) -> PyResult<Option<Py<PyTsNamespaceBody>>>{
    match body {
        None => Ok(None),
        Some(b) => Ok(Some(conv_ts_namespace_body(py, b)?))
    }
}

pub fn conv_boxed_ts_namespace_body(py: Python<'_>, body: Box<TsNamespaceBody>) -> PyResult<Py<PyTsNamespaceBody>>{
    conv_ts_namespace_body(py, *body)
}

pub fn conv_module_items(py: Python<'_>, items: Vec<ModuleItem>) -> PyResult<Vec<Py<PyModuleItem>>>{
    items.into_iter().map(|i| conv_module_item(py, i)).collect()
}

pub fn conv_import_phase(_py: Python<'_>, phase: ImportPhase) -> PyResult<u32>{
    Ok(phase as u32)
}

pub fn conv_boxed_str(py: Python<'_>, s: Box<Str>) -> PyResult<String>{
    conv_str(py, *s)
}

pub fn conv_option_boxed_str(py: Python<'_>, s: Option<Box<Str>>) -> PyResult<Option<String>>{
    match s {
        None => Ok(None),
        Some(s) => Ok(Some(conv_boxed_str(py, s)?))
    }
}

pub fn conv_option_boxed_object_lit(py: Python<'_>, lit: Option<Box<ObjectLit>>) -> PyResult<Option<Py<PyExpr>>>{
    match lit {
        None => Ok(None),
        Some(l) => Ok(Some(conv_boxed_object_lit(py, l)?))
    }
}

pub fn conv_module_export_name(py: Python<'_>, name: ModuleExportName) -> PyResult<PyModuleExportName>{
    match name {
        ModuleExportName::Ident(i) => Ok(PyModuleExportName{ ident: Some(conv_ident(py, i)?), str_span: None, str_value: None }),
        ModuleExportName::Str(s) => Ok(PyModuleExportName{ ident: None, str_span: Some(conv_span(py, s.span)?), str_value: Some(conv_str(py, s)?) }),
    }
}

pub fn conv_option_module_export_name(py: Python<'_>, name: Option<ModuleExportName>) -> PyResult<Option<PyModuleExportName>>{
    match name {
        None => Ok(None),
        Some(n) => Ok(Some(conv_module_export_name(py, n)?))
    }
}

pub fn conv_ts_module_ref(py: Python<'_>, node: TsModuleRef) -> PyResult<Py<PyTsModuleRef>>{
    match node {
        TsModuleRef::TsEntityName(n) => Py::new(py, PyTsModuleRef{ entity_name: Some(conv_tsentityname(py, n)?), external_module_ref: None }),
        TsModuleRef::TsExternalModuleRef(r) => {
            let external = Py::new(py, PyTsExternalModuleRef{ span: conv_span(py, r.span)?, expr: conv_str(py, r.expr)? })?;
            Py::new(py, PyTsModuleRef{ entity_name: None, external_module_ref: Some(external) })
        },
    }
}

pub fn module_to_py(py: Python<'_>, module: Module) -> PyResult<Py<PySourceModule>>{
    conv_module(py, module)
}

pub fn conv_jsx_object(py: Python<'_>, obj: JSXObject) -> PyResult<Py<PyJSXObject>>{
    match obj {
        JSXObject::JSXMemberExpr(m) => Py::new(py, PyJSXObject{ jsx_member_expr: Some(conv_jsx_member_expr(py, *m)?), ident: None }),
        JSXObject::Ident(i) => Py::new(py, PyJSXObject{ jsx_member_expr: None, ident: Some(conv_ident(py, i)?) }),
    }
}

pub fn conv_jsx_member_expr(py: Python<'_>, node: JSXMemberExpr) -> PyResult<Py<PyJSXMemberExpr>>{
    let base = PyExpr { expr: Expr::JSXMember(node.clone()) };
    let sub = PyJSXMemberExpr::build(py, node)?;
    Py::new(py, (sub, base))
}

pub fn conv_jsx_member_expr_data(py: Python<'_>, node: JSXMemberExpr) -> PyResult<Py<PyJSXMemberExprData>>{
    Py::new(py, PyJSXMemberExprData{
        span: conv_span(py, node.span)?,
        obj: conv_jsx_object(py, node.obj)?,
        prop: conv_identname(py, node.prop)?
    })
}

pub fn conv_jsx_expr(py: Python<'_>, expr: JSXExpr) -> PyResult<Py<PyJSXExpr>>{
    match expr {
        JSXExpr::JSXEmptyExpr(e) => Py::new(py, PyJSXExpr{ empty_span: Some(conv_span(py, e.span)?), expr: None }),
        JSXExpr::Expr(e) => Py::new(py, PyJSXExpr{ empty_span: None, expr: Some(conv_boxed_expr(py, e)?) }),
    }
}

pub fn conv_jsx_element_name(py: Python<'_>, name: JSXElementName) -> PyResult<Py<PyJSXElementName>>{
    match name {
        JSXElementName::Ident(i) => Py::new(py, PyJSXElementName{ ident: Some(conv_ident(py, i)?), jsx_member_expr: None, jsx_namespaced_name: None }),
        JSXElementName::JSXMemberExpr(m) => {
            let data = conv_jsx_member_expr_data(py, m)?;
            Py::new(py, PyJSXElementName{ ident: None, jsx_member_expr: Some(data), jsx_namespaced_name: None })
        },
        JSXElementName::JSXNamespacedName(n) => Py::new(py, PyJSXElementName{
            ident: None,
            jsx_member_expr: None,
            jsx_namespaced_name: Some(conv_jsx_namespaced_name_data(py, n)?)
        }),
    }
}

pub fn conv_jsx_namespaced_name_data(py: Python<'_>, node: JSXNamespacedName) -> PyResult<PyJSXNamespacedNameData>{
    Ok(PyJSXNamespacedNameData{
        span: conv_span(py, node.span)?,
        ns: conv_identname(py, node.ns)?,
        name: conv_identname(py, node.name)?
    })
}

pub fn conv_jsx_attr_name(py: Python<'_>, name: JSXAttrName) -> PyResult<PyJSXAttrName>{
    match name {
        JSXAttrName::Ident(i) => Ok(PyJSXAttrName{ ident: Some(conv_identname(py, i)?), jsx_namespaced_name: None }),
        JSXAttrName::JSXNamespacedName(n) => Ok(PyJSXAttrName{ ident: None, jsx_namespaced_name: Some(conv_jsx_namespaced_name_data(py, n)?) }),
    }
}

pub fn conv_jsx_element(py: Python<'_>, node: JSXElement) -> PyResult<Py<PyJSXElement>>{
    let base = PyExpr { expr: Expr::JSXElement(Box::new(node.clone())) };
    let sub = PyJSXElement::build(py, node)?;
    Py::new(py, (sub, base))
}

pub fn conv_option_jsx_closing_element(py: Python<'_>, node: Option<JSXClosingElement>) -> PyResult<Option<Py<PyJSXClosingElement>>>{
    match node {
        None => Ok(None),
        Some(e) => Ok(Some(Py::new(py, PyJSXClosingElement::build(py, e)?)?))
    }
}

pub fn conv_jsx_opening_element(py: Python<'_>, node: JSXOpeningElement) -> PyResult<Py<PyJSXOpeningElement>>{
    Py::new(py, PyJSXOpeningElement::build(py, node)?)
}

pub fn conv_jsx_fragment(py: Python<'_>, node: JSXFragment) -> PyResult<Py<PyJSXFragment>>{
    let base = PyExpr { expr: Expr::JSXFragment(node.clone()) };
    let sub = PyJSXFragment::build(py, node)?;
    Py::new(py, (sub, base))
}

pub fn conv_boxed_exprs(py: Python<'_>, exprs: Vec<Box<Expr>>) -> PyResult<Vec<Py<PyExpr>>>{
    exprs.into_iter().map(|e| conv_boxed_expr(py, e)).collect()
}

pub fn conv_option_expr_or_spreads_noopt(py: Python<'_>, elems: Option<Vec<ExprOrSpread>>) -> PyResult<Option<Vec<Py<PyExprOrSpread>>>>{
    match elems {
        None => Ok(None),
        Some(e) => Ok(Some(conv_expr_or_spreads_noopt(py, e)?))
    }
}

pub fn conv_pats(py: Python<'_>, pats: Vec<Pat>) -> PyResult<Vec<Py<PyPat>>>{
    pats.into_iter().map(|p| conv_pat(py, p)).collect()
}

pub fn conv_boxed_tpl(py: Python<'_>, tpl: Box<Tpl>) -> PyResult<Py<PyTpl>>{
    let base = PyExpr { expr: Expr::Tpl(*tpl.clone()) };
    let sub = PyTpl::build(py, *tpl)?;
    Py::new(py, (sub, base))
}

pub fn conv_arrow_function_body(py: Python<'_>, body: ArrowFunctionBody) -> PyResult<Py<PyArrowFunctionBody>>{
    match body {
        ArrowFunctionBody::FunctionBody(b) => {
            let fb = conv_function_body(py, b)?;
            Py::new(py, PyArrowFunctionBody{ function_body: Some(fb), expr: None })
        },
        ArrowFunctionBody::Expr(e) => {
            let ex = conv_boxed_expr(py, e)?;
            Py::new(py, PyArrowFunctionBody{ function_body: None, expr: Some(ex) })
        }
    }
}

pub fn conv_boxed_arrow_function_body(py: Python<'_>, body: Box<ArrowFunctionBody>) -> PyResult<Py<PyArrowFunctionBody>>{
    conv_arrow_function_body(py, *body)
}

pub fn conv_meta_prop_kind(_py: Python<'_>, kind: MetaPropKind) -> PyResult<u32>{
    Ok(kind as u32)
}

pub fn conv_boxed_stmt(py: Python<'_>, stmt: Box<Stmt>) -> PyResult<Py<PyStmt>>{
    stmt_to_py(py, *stmt)
}

pub fn conv_option_boxed_stmt(py: Python<'_>, stmt: Option<Box<Stmt>>) -> PyResult<Option<Py<PyStmt>>>{
    match stmt {
        None => Ok(None),
        Some(s) => Ok(Some(conv_boxed_stmt(py, s)?))
    }
}

pub fn conv_switch_case(py: Python<'_>, node: SwitchCase) -> PyResult<Py<PySwitchCase>>{
    Py::new(py, PySwitchCase{
        span: conv_span(py, node.span)?,
        test: conv_option_boxed_expr(py, node.test)?,
        cons: conv_stmts(py, node.cons)?
    })
}

pub fn conv_switch_cases(py: Python<'_>, nodes: Vec<SwitchCase>) -> PyResult<Vec<Py<PySwitchCase>>>{
    nodes.into_iter().map(|n| conv_switch_case(py, n)).collect()
}

pub fn conv_var_decl_or_expr(py: Python<'_>, node: VarDeclOrExpr) -> PyResult<Py<PyVarDeclOrExpr>>{
    match node {
        VarDeclOrExpr::VarDecl(v) => {
            let vd = crate::pydecl::PyVarDecl::build(py, *v)?;
            let vd_py = Py::new(py, (vd, crate::pydecl::PyDecl {}))?;
            Py::new(py, PyVarDeclOrExpr{ var_decl: Some(vd_py), expr: None })
        },
        VarDeclOrExpr::Expr(e) => Py::new(py, PyVarDeclOrExpr{ var_decl: None, expr: Some(conv_boxed_expr(py, e)?) }),
    }
}

pub fn conv_option_var_decl_or_expr(py: Python<'_>, node: Option<VarDeclOrExpr>) -> PyResult<Option<Py<PyVarDeclOrExpr>>>{
    match node {
        None => Ok(None),
        Some(n) => Ok(Some(conv_var_decl_or_expr(py, n)?))
    }
}

pub fn conv_for_head(py: Python<'_>, node: ForHead) -> PyResult<Py<PyForHead>>{
    match node {
        ForHead::VarDecl(v) => {
            let vd = crate::pydecl::PyVarDecl::build(py, *v)?;
            let vd_py = Py::new(py, (vd, crate::pydecl::PyDecl {}))?;
            Py::new(py, PyForHead{ var_decl: Some(vd_py), using_decl: None, pat: None })
        },
        ForHead::UsingDecl(u) => {
            let ud = crate::pydecl::PyUsingDecl::build(py, *u)?;
            let ud_py = Py::new(py, (ud, crate::pydecl::PyDecl {}))?;
            Py::new(py, PyForHead{ var_decl: None, using_decl: Some(ud_py), pat: None })
        },
        ForHead::Pat(p) => Py::new(py, PyForHead{ var_decl: None, using_decl: None, pat: Some(conv_pat(py, *p)?) }),
    }
}

