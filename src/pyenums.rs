use crate::macros::py_enum;
use swc_core::ecma::ast::{
    Accessibility, AssignOp, BinaryOp, ImportPhase, MetaPropKind, MethodKind, TruePlusMinus,
    TsKeywordTypeKind, TsTypeOperatorOp, UnaryOp, UpdateOp, VarDeclKind,
};

py_enum!(PyUnaryOp, UnaryOp, {
    Minus, Plus, Bang, Tilde, TypeOf, Void, Delete
});

py_enum!(PyAssignOp, AssignOp, {
    Assign, AddAssign, SubAssign, MulAssign, DivAssign, ModAssign, LShiftAssign, RShiftAssign,
    ZeroFillRShiftAssign, BitOrAssign, BitXorAssign, BitAndAssign, ExpAssign, AndAssign,
    OrAssign, NullishAssign
});

py_enum!(PyUpdateOp, UpdateOp, {
    PlusPlus, MinusMinus
});

py_enum!(PyBinaryOp, BinaryOp, {
    EqEq, NotEq, EqEqEq, NotEqEq, Lt, LtEq, Gt, GtEq, LShift, RShift, ZeroFillRShift, Add, Sub,
    Mul, Div, Mod, BitOr, BitXor, BitAnd, LogicalOr, LogicalAnd, In, InstanceOf, Exp,
    NullishCoalescing
});

py_enum!(PyTsKeywordTypeKind, TsKeywordTypeKind, {
    TsAnyKeyword, TsUnknownKeyword, TsNumberKeyword, TsObjectKeyword, TsBooleanKeyword,
    TsBigIntKeyword, TsStringKeyword, TsSymbolKeyword, TsVoidKeyword, TsUndefinedKeyword,
    TsNullKeyword, TsNeverKeyword, TsIntrinsicKeyword
});

py_enum!(PyTruePlusMinus, TruePlusMinus, {
    True, Plus, Minus
});

py_enum!(PyTsTypeOperatorOp, TsTypeOperatorOp, {
    KeyOf, Unique, ReadOnly
});

py_enum!(PyAccessibility, Accessibility, {
    Public, Protected, Private
});

py_enum!(PyMethodKind, MethodKind, {
    Method, Getter, Setter
});

py_enum!(PyVarDeclKind, VarDeclKind, {
    Var, Let, Const
});

py_enum!(PyImportPhase, ImportPhase, {
    Evaluation, Source, Defer
});

py_enum!(PyMetaPropKind, MetaPropKind, {
    NewTarget, ImportMeta
});
