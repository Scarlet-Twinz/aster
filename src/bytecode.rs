use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Number(f64),
    String(String),
    Boolean(bool),
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Constant(usize),
    LoadGlobal(usize),
    StoreGlobal(usize),
    LoadLocal(usize),
    StoreLocal(usize),
    Pop,
    Negate,
    Not,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    JumpIfFalse(usize),
    Jump(usize),
    Print,
    Call(usize),
    Return,
    Halt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub local_names: Vec<String>,
    pub constants: Vec<Constant>,
    pub code: Vec<OpCode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub constants: Vec<Constant>,
    pub names: Vec<String>,
    pub code: Vec<OpCode>,
    pub functions: Vec<Function>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            names: Vec::new(),
            code: Vec::new(),
            functions: Vec::new(),
        }
    }

    fn add_constant(&mut self, constant: Constant) -> usize {
        let index = self.constants.len();
        self.constants.push(constant);
        index
    }

    fn add_name(&mut self, name: &str) -> usize {
        if let Some(index) = self.names.iter().position(|existing| existing == name) {
            index
        } else {
            let index = self.names.len();
            self.names.push(name.to_string());
            index
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BytecodeError {
    pub message: String,
}

pub struct Compiler {
    chunk: Chunk,
    functions: HashMap<String, usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Self { chunk: Chunk::new(), functions: HashMap::new() }
    }

    pub fn compile(mut self, program: &Program) -> Result<Chunk, BytecodeError> {
        for statement in &program.statements {
            if let Stmt::Function { name, params, .. } = statement {
                let index = self.chunk.functions.len();
                if self.functions.insert(name.clone(), index).is_some() {
                    return Err(BytecodeError { message: format!("duplicate function '{}'", name) });
                }
                self.chunk.functions.push(Function {
                    name: name.clone(),
                    arity: params.len(),
                    local_names: Vec::new(),
                    constants: Vec::new(),
                    code: Vec::new(),
                });
            }
        }

        for statement in &program.statements {
            if let Stmt::Function { name, params, body } = statement {
                self.compile_function(name, params, body)?;
            } else {
                self.compile_statement(statement)?;
            }
        }

        self.chunk.code.push(OpCode::Halt);
        Ok(self.chunk)
    }

    fn compile_function(&mut self, name: &str, params: &[String], body: &[Stmt]) -> Result<(), BytecodeError> {
        let index = *self.functions.get(name).ok_or_else(|| BytecodeError { message: format!("unknown function '{}'", name) })?;
        let mut compiler = FunctionCompiler::new(params);
        for statement in body {
            compiler.compile_statement(statement, &self.functions)?;
        }
        let void_index = compiler.add_constant(Constant::Void);
        compiler.code.push(OpCode::Constant(void_index));
        compiler.code.push(OpCode::Return);
        self.chunk.functions[index] = Function {
            name: name.to_string(),
            arity: params.len(),
            local_names: compiler.local_names,
            constants: compiler.constants,
            code: compiler.code,
        };
        Ok(())
    }

    fn compile_statement(&mut self, statement: &Stmt) -> Result<(), BytecodeError> {
        match statement {
            Stmt::Let { name, initializer } => {
                self.compile_expression(initializer)?;
                let name_index = self.chunk.add_name(name);
                self.chunk.code.push(OpCode::StoreGlobal(name_index));
            }
            Stmt::Expression(expression) => {
                self.compile_expression(expression)?;
                self.chunk.code.push(OpCode::Pop);
            }
            Stmt::Print(expression) => {
                self.compile_expression(expression)?;
                self.chunk.code.push(OpCode::Print);
            }
            Stmt::Return(_) | Stmt::Function { .. } => {
                return Err(BytecodeError { message: "return/function declaration is only valid inside function compilation".into() });
            }
            Stmt::Block(statements) => {
                for statement in statements {
                    self.compile_statement(statement)?;
                }
            }
            Stmt::If { condition, then_branch, else_branch } => {
                self.compile_expression(condition)?;
                let jump_if_false = self.emit_placeholder_jump_if_false();
                self.compile_statement(then_branch)?;
                if let Some(else_branch) = else_branch {
                    let jump_end = self.emit_placeholder_jump();
                    self.patch_jump(jump_if_false)?;
                    self.compile_statement(else_branch)?;
                    self.patch_jump(jump_end)?;
                } else {
                    self.patch_jump(jump_if_false)?;
                }
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expression: &Expr) -> Result<(), BytecodeError> {
        match expression {
            Expr::Number(value) => { let index = self.chunk.add_constant(Constant::Number(*value)); self.chunk.code.push(OpCode::Constant(index)); }
            Expr::String(value) => { let index = self.chunk.add_constant(Constant::String(value.clone())); self.chunk.code.push(OpCode::Constant(index)); }
            Expr::Boolean(value) => { let index = self.chunk.add_constant(Constant::Boolean(*value)); self.chunk.code.push(OpCode::Constant(index)); }
            Expr::Variable(name) => { let index = self.chunk.add_name(name); self.chunk.code.push(OpCode::LoadGlobal(index)); }
            Expr::Assign { name, value } => {
                self.compile_expression(value)?;
                let index = self.chunk.add_name(name);
                self.chunk.code.push(OpCode::StoreGlobal(index));
                self.chunk.code.push(OpCode::LoadGlobal(index));
            }
            Expr::Unary { operator, operand } => {
                self.compile_expression(operand)?;
                self.chunk.code.push(match operator { UnaryOp::Negate => OpCode::Negate, UnaryOp::Not => OpCode::Not });
            }
            Expr::Binary { left, operator, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                self.chunk.code.push(match operator {
                    BinaryOp::Add => OpCode::Add, BinaryOp::Subtract => OpCode::Subtract, BinaryOp::Multiply => OpCode::Multiply,
                    BinaryOp::Divide => OpCode::Divide, BinaryOp::Modulo => OpCode::Modulo, BinaryOp::Equal => OpCode::Equal,
                    BinaryOp::NotEqual => OpCode::NotEqual, BinaryOp::Less => OpCode::Less, BinaryOp::LessEqual => OpCode::LessEqual,
                    BinaryOp::Greater => OpCode::Greater, BinaryOp::GreaterEqual => OpCode::GreaterEqual, BinaryOp::And => OpCode::And,
                    BinaryOp::Or => OpCode::Or,
                });
            }
            Expr::Call { callee, arguments } => {
                let name = match callee.as_ref() { Expr::Variable(name) => name, _ => return Err(BytecodeError { message: "only named functions can be called".into() }) };
                let function_index = *self.functions.get(name).ok_or_else(|| BytecodeError { message: format!("undefined function '{}'", name) })?;
                for argument in arguments { self.compile_expression(argument)?; }
                self.chunk.code.push(OpCode::Call(function_index));
            }
        }
        Ok(())
    }

    fn emit_placeholder_jump_if_false(&mut self) -> usize { let index = self.chunk.code.len(); self.chunk.code.push(OpCode::JumpIfFalse(usize::MAX)); index }
    fn emit_placeholder_jump(&mut self) -> usize { let index = self.chunk.code.len(); self.chunk.code.push(OpCode::Jump(usize::MAX)); index }
    fn patch_jump(&mut self, index: usize) -> Result<(), BytecodeError> {
        let target = self.chunk.code.len();
        let instruction = self.chunk.code.get_mut(index).ok_or_else(|| BytecodeError { message: "invalid jump patch location".into() })?;
        *instruction = match instruction { OpCode::JumpIfFalse(_) => OpCode::JumpIfFalse(target), OpCode::Jump(_) => OpCode::Jump(target), _ => return Err(BytecodeError { message: "invalid jump instruction".into() }) };
        Ok(())
    }
}

struct FunctionCompiler {
    local_names: Vec<String>,
    locals: HashMap<String, usize>,
    constants: Vec<Constant>,
    code: Vec<OpCode>,
}

impl FunctionCompiler {
    fn new(params: &[String]) -> Self {
        let mut compiler = Self { local_names: Vec::new(), locals: HashMap::new(), constants: Vec::new(), code: Vec::new() };
        for param in params { compiler.define_local(param); }
        compiler
    }

    fn define_local(&mut self, name: &str) -> usize {
        if let Some(index) = self.locals.get(name).copied() { return index; }
        let index = self.local_names.len();
        self.local_names.push(name.to_string());
        self.locals.insert(name.to_string(), index);
        index
    }

    fn add_constant(&mut self, constant: Constant) -> usize { let index = self.constants.len(); self.constants.push(constant); index }

    fn compile_statement(&mut self, statement: &Stmt, functions: &HashMap<String, usize>) -> Result<(), BytecodeError> {
        match statement {
            Stmt::Let { name, initializer } => {
                self.compile_expression(initializer, functions)?;
                let index = self.define_local(name);
                self.code.push(OpCode::StoreLocal(index));
            }
            Stmt::Expression(expression) => { self.compile_expression(expression, functions)?; self.code.push(OpCode::Pop); }
            Stmt::Print(expression) => { self.compile_expression(expression, functions)?; self.code.push(OpCode::Print); }
            Stmt::Return(value) => {
                if let Some(expression) = value { self.compile_expression(expression, functions)?; }
                else { let index = self.add_constant(Constant::Void); self.code.push(OpCode::Constant(index)); }
                self.code.push(OpCode::Return);
            }
            Stmt::Block(statements) => { for statement in statements { self.compile_statement(statement, functions)?; } }
            Stmt::If { condition, then_branch, else_branch } => {
                self.compile_expression(condition, functions)?;
                let jump_if_false = self.emit_placeholder_jump_if_false();
                self.compile_statement(then_branch, functions)?;
                if let Some(else_branch) = else_branch {
                    let jump_end = self.emit_placeholder_jump();
                    self.patch_jump(jump_if_false)?;
                    self.compile_statement(else_branch, functions)?;
                    self.patch_jump(jump_end)?;
                } else { self.patch_jump(jump_if_false)?; }
            }
            Stmt::Function { .. } => return Err(BytecodeError { message: "nested function declarations are not yet supported".into() }),
        }
        Ok(())
    }

    fn compile_expression(&mut self, expression: &Expr, functions: &HashMap<String, usize>) -> Result<(), BytecodeError> {
        match expression {
            Expr::Number(value) => { let i = self.add_constant(Constant::Number(*value)); self.code.push(OpCode::Constant(i)); }
            Expr::String(value) => { let i = self.add_constant(Constant::String(value.clone())); self.code.push(OpCode::Constant(i)); }
            Expr::Boolean(value) => { let i = self.add_constant(Constant::Boolean(*value)); self.code.push(OpCode::Constant(i)); }
            Expr::Variable(name) => {
                let i = *self.locals.get(name).ok_or_else(|| BytecodeError { message: format!("undefined local variable '{}'", name) })?;
                self.code.push(OpCode::LoadLocal(i));
            }
            Expr::Assign { name, value } => {
                self.compile_expression(value, functions)?;
                let i = *self.locals.get(name).ok_or_else(|| BytecodeError { message: format!("undefined local variable '{}'", name) })?;
                self.code.push(OpCode::StoreLocal(i));
                self.code.push(OpCode::LoadLocal(i));
            }
            Expr::Unary { operator, operand } => { self.compile_expression(operand, functions)?; self.code.push(match operator { UnaryOp::Negate => OpCode::Negate, UnaryOp::Not => OpCode::Not }); }
            Expr::Binary { left, operator, right } => {
                self.compile_expression(left, functions)?; self.compile_expression(right, functions)?;
                self.code.push(match operator {
                    BinaryOp::Add => OpCode::Add, BinaryOp::Subtract => OpCode::Subtract, BinaryOp::Multiply => OpCode::Multiply,
                    BinaryOp::Divide => OpCode::Divide, BinaryOp::Modulo => OpCode::Modulo, BinaryOp::Equal => OpCode::Equal,
                    BinaryOp::NotEqual => OpCode::NotEqual, BinaryOp::Less => OpCode::Less, BinaryOp::LessEqual => OpCode::LessEqual,
                    BinaryOp::Greater => OpCode::Greater, BinaryOp::GreaterEqual => OpCode::GreaterEqual, BinaryOp::And => OpCode::And,
                    BinaryOp::Or => OpCode::Or,
                });
            }
            Expr::Call { callee, arguments } => {
                let name = match callee.as_ref() { Expr::Variable(name) => name, _ => return Err(BytecodeError { message: "only named functions can be called".into() }) };
                let index = *functions.get(name).ok_or_else(|| BytecodeError { message: format!("undefined function '{}'", name) })?;
                for argument in arguments { self.compile_expression(argument, functions)?; }
                self.code.push(OpCode::Call(index));
            }
        }
        Ok(())
    }

    fn emit_placeholder_jump_if_false(&mut self) -> usize { let i = self.code.len(); self.code.push(OpCode::JumpIfFalse(usize::MAX)); i }
    fn emit_placeholder_jump(&mut self) -> usize { let i = self.code.len(); self.code.push(OpCode::Jump(usize::MAX)); i }
    fn patch_jump(&mut self, index: usize) -> Result<(), BytecodeError> {
        let target = self.code.len();
        let instruction = self.code.get_mut(index).ok_or_else(|| BytecodeError { message: "invalid jump patch location".into() })?;
        *instruction = match instruction { OpCode::JumpIfFalse(_) => OpCode::JumpIfFalse(target), OpCode::Jump(_) => OpCode::Jump(target), _ => return Err(BytecodeError { message: "invalid jump instruction".into() }) };
        Ok(())
    }
}

impl Default for Compiler { fn default() -> Self { Self::new() } }

pub fn compile(program: &Program) -> Result<Chunk, BytecodeError> { Compiler::new().compile(program) }
