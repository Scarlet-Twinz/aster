use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::type_system::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError { pub message: String }

#[derive(Debug, Clone, Copy)]
struct FunctionInfo { arity: usize }

#[derive(Debug, Default)]
pub struct Analyzer {
    scopes: Vec<HashMap<String, Type>>,
    functions: HashMap<String, FunctionInfo>,
    errors: Vec<SemanticError>,
}

impl Analyzer {
    pub fn new() -> Self { Self { scopes: vec![HashMap::new()], functions: HashMap::new(), errors: Vec::new() } }
    pub fn analyze(mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        self.register_functions(program);
        for statement in &program.statements { self.check_statement(statement); }
        if self.errors.is_empty() { Ok(()) } else { Err(self.errors) }
    }
    fn register_functions(&mut self, program: &Program) {
        for statement in &program.statements {
            if let Stmt::Function { name, params, .. } = statement {
                if self.functions.contains_key(name) { self.error(format!("function '{}' is already declared", name)); }
                else { self.functions.insert(name.clone(), FunctionInfo { arity: params.len() }); }
            }
        }
    }
    fn check_statement(&mut self, statement: &Stmt) -> Type {
        match statement {
            Stmt::Let { name, type_annotation, initializer } => {
                let value_type = self.check_expression(initializer);
                if let Some(expected) = type_annotation {
                    if *expected == Type::Void { self.error(format!("variable '{}' cannot have type void", name)); }
                    else if value_type != Type::Unknown && value_type != *expected {
                        self.error(format!("type annotation for '{}' expects {}, found {}", name, expected, value_type));
                    }
                }
                if self.current_scope_contains(name) { self.error(format!("variable '{}' is already declared in this scope", name)); }
                else { self.define(name.clone(), type_annotation.as_ref().copied().unwrap_or(value_type)); }
                Type::Void
            }
            Stmt::Expression(expression) => { self.check_expression(expression); Type::Void }
            Stmt::Print(expression) => { self.check_expression(expression); Type::Void }
            Stmt::Return(value) => { if let Some(expression) = value { self.check_expression(expression) } else { Type::Void } }
            Stmt::Block(statements) => { self.begin_scope(); for statement in statements { self.check_statement(statement); } self.end_scope(); Type::Void }
            Stmt::If { condition, then_branch, else_branch } => {
                let condition_type = self.check_expression(condition);
                self.require_type(condition_type, Type::Boolean, "if condition must be bool");
                self.check_statement(then_branch);
                if let Some(branch) = else_branch { self.check_statement(branch); }
                Type::Void
            }
            Stmt::Function { params, body, .. } => {
                self.begin_scope();
                for parameter in params {
                    if self.current_scope_contains(parameter) { self.error(format!("parameter '{}' is declared more than once", parameter)); }
                    else { self.define(parameter.clone(), Type::Unknown); }
                }
                for statement in body { self.check_statement(statement); }
                self.end_scope();
                Type::Void
            }
        }
    }
    fn check_expression(&mut self, expression: &Expr) -> Type {
        match expression {
            Expr::Number(_) => Type::Number,
            Expr::String(_) => Type::String,
            Expr::Boolean(_) => Type::Boolean,
            Expr::Variable(name) => match self.lookup(name) { Some(ty) => ty, None => { self.error(format!("undefined variable '{}'", name)); Type::Unknown } },
            Expr::Assign { name, value } => {
                let value_type = self.check_expression(value);
                if let Some(existing_type) = self.lookup(name) {
                    if existing_type != Type::Unknown && value_type != Type::Unknown && existing_type != value_type { self.error(format!("cannot assign {} to variable '{}' of type {}", value_type, name, existing_type)); }
                } else { self.error(format!("cannot assign to undefined variable '{}'", name)); }
                value_type
            }
            Expr::Unary { operator, operand } => {
                let operand_type = self.check_expression(operand);
                match operator {
                    UnaryOp::Negate => { self.require_type(operand_type, Type::Number, "unary '-' requires a number"); Type::Number }
                    UnaryOp::Not => { self.require_type(operand_type, Type::Boolean, "unary '!' requires a bool"); Type::Boolean }
                }
            }
            Expr::Binary { left, operator, right } => {
                let left_type = self.check_expression(left); let right_type = self.check_expression(right); self.check_binary(*operator, left_type, right_type)
            }
            Expr::Call { callee, arguments } => {
                let callee_name = match callee.as_ref() { Expr::Variable(name) => Some(name.as_str()), _ => None };
                for argument in arguments { self.check_expression(argument); }
                match callee_name {
                    Some(name) => match self.functions.get(name).copied() {
                        Some(function) => { if function.arity != arguments.len() { self.error(format!("function '{}' expects {} argument(s), got {}", name, function.arity, arguments.len())); } Type::Unknown }
                        None => { self.error(format!("undefined function '{}'", name)); Type::Unknown }
                    },
                    None => { self.error("only named functions can be called".into()); Type::Unknown }
                }
            }
        }
    }
    fn check_binary(&mut self, operator: BinaryOp, left: Type, right: Type) -> Type {
        match operator {
            BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => { self.require_type(left, Type::Number, "arithmetic operators require numbers"); self.require_type(right, Type::Number, "arithmetic operators require numbers"); Type::Number }
            BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => { self.require_type(left, Type::Number, "comparison operators require numbers"); self.require_type(right, Type::Number, "comparison operators require numbers"); Type::Boolean }
            BinaryOp::Equal | BinaryOp::NotEqual => { if left != Type::Unknown && right != Type::Unknown && left != right { self.error(format!("cannot compare {} with {}", left, right)); } Type::Boolean }
            BinaryOp::And | BinaryOp::Or => { self.require_type(left, Type::Boolean, "logical operators require bools"); self.require_type(right, Type::Boolean, "logical operators require bools"); Type::Boolean }
        }
    }
    fn require_type(&mut self, actual: Type, expected: Type, message: &str) { if actual != Type::Unknown && actual != expected { self.error(format!("{} (found {}, expected {})", message, actual, expected)); } }
    fn begin_scope(&mut self) { self.scopes.push(HashMap::new()); }
    fn end_scope(&mut self) { let _ = self.scopes.pop(); }
    fn define(&mut self, name: String, ty: Type) { if let Some(scope) = self.scopes.last_mut() { scope.insert(name, ty); } }
    fn current_scope_contains(&self, name: &str) -> bool { self.scopes.last().map(|scope| scope.contains_key(name)).unwrap_or(false) }
    fn lookup(&self, name: &str) -> Option<Type> { self.scopes.iter().rev().find_map(|scope| scope.get(name).copied()) }
    fn error(&mut self, message: String) { self.errors.push(SemanticError { message }); }
}

pub fn analyze(program: &Program) -> Result<(), Vec<SemanticError>> { Analyzer::new().analyze(program) }
