#[derive(Debug, Clone)]
pub enum TypedExpression {
    Number32 {
        value: i32,
        // span: Span,
    },
    Number64 {
        value: i64,
    },
    String {
        value: String,
    },
    Bool {
        value: bool,
    },
    Binary {
        left: Box<TypedExpression>,
        right: Box<TypedExpression>,
        op: String,
    },
    CallStmt {
        callee: Box<TypedExpression>,
        args: Vec<TypedExpression>,
    },
    FuncStmt {
        name: String,
        args: Vec<(String, ResolvedType)>,
        return_type: ResolvedType,
        body: Box<TypedExpression>,
    },
    BlockStmt {
        statements: Vec<TypedExpression>,
    },
    Variable {
        name: String,
    },
    Print {
        value: Box<TypedExpression>,
    },
    ReturnStmt {
        value: Box<TypedExpression>,
    },
    BreakStmt,
    LetStmt {
        name: String,
        var_type: Option<ResolvedType>,
        value: Box<TypedExpression>,
    },
    IfStmt {
        condition: Box<TypedExpression>,
        then_branch: Box<TypedExpression>,
        else_branch: Option<Box<TypedExpression>>,
    },
    WhileStmt {
        condition: Box<TypedExpression>,
        body: Box<TypedExpression>,
    },
    AssignStmt {
        name: String,
        value: Box<TypedExpression>,
    },
    Grouping {
        inner: Box<TypedExpression>,
    },
    List {
        elements: Vec<TypedExpression>,
        element_type: ResolvedType,
    },
    ListIndex {
        list: Box<TypedExpression>,
        index: Box<TypedExpression>,
    },
    ListAssign {
        name: String,
        index: Box<TypedExpression>,
        value: Box<TypedExpression>,
    },
    Len {
        value: Box<TypedExpression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedType {
    I32,
    I64,
    String,
    Bool,
    Binary(Box<ResolvedType>, String, Box<ResolvedType>),
    List(Box<ResolvedType>),
    Function(Vec<ResolvedType>, Box<ResolvedType>),
    CallStmt(Vec<ResolvedType>, Box<ResolvedType>),
    Void,
}
