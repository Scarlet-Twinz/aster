use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Number(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Constant(usize),
    LoadGlobal(usize),
    StoreGlobal(usize),
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
    Halt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub constants: Vec<Constant>,
    pub names: Vec<String>,
    pub code: Vec<OpCode>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            names: Vec::new(),
            code: Vec::new(),
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
}

impl Compiler {
    pub fn new() -> Self {
        Self { chunk: Chunk::new() }
    }

    pub fn compile(mut self, program: &Program) -> Result<Chunk, BytecodeError> {
        for statement in &program.statements {
            self.compile_statement(statement)?;
        }
        self.chunk.code.push(OpCode::Halt);
        Ok(self.chunk)
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
                return Err(BytecodeError {
                    message: "functions and return statements are not yet supported by the bytecode backend".into(),
                });
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
            Expr::Number(value) => {
                let index = self.chunk.add_constant(Constant::Number(*value));
                self.chunk.code.push(OpCode::Constant(index));
            }
            Expr::String(value) => {
                let index = self.chunk.add_constant(Constant::String(value.clone()));
                self.chunk.code.push(OpCode::Constant(index));
            }
            Expr::Boolean(value) => {
                let index = self.chunk.add_constant(Constant::Boolean(*value));
                self.chunk.code.push(OpCode::Constant(index));
            }
            Expr::Variable(name) => {
                let index = self.chunk.add_name(name);
                self.chunk.code.push(OpCode::LoadGlobal(index));
            }
            Expr::Assign { name, value } => {
                self.compile_expression(value)?;
                let index = self.chunk.add_name(name);
                self.chunk.code.push(OpCode::StoreGlobal(index));
                self.chunk.code.push(OpCode::LoadGlobal(index));
            }
            Expr::Unary { operator, operand } => {
                self.compile_expression(operand)?;
                self.chunk.code.push(match operator {
                    UnaryOp::Negate => OpCode::Negate,
                    UnaryOp::Not => OpCode::Not,
                });
            }
            Expr::Binary { left, operator, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                self.chunk.code.push(match operator {
                    BinaryOp::Add => OpCode::Add,
                    BinaryOp::Subtract => OpCode::Subtract,
                    BinaryOp::Multiply => OpCode::Multiply,
                    BinaryOp::Divide => OpCode::Divide,
                    BinaryOp::Modulo => OpCode::Modulo,
                    BinaryOp::Equal => OpCode::Equal,
                    BinaryOp::NotEqual => OpCode::NotEqual,
                    BinaryOp::Less => OpCode::Less,
                    BinaryOp::LessEqual => OpCode::LessEqual,
                    BinaryOp::Greater => OpCode::Greater,
                    BinaryOp::GreaterEqual => OpCode::GreaterEqual,
                    BinaryOp::And => OpCode::And,
                    BinaryOp::Or => OpCode::Or,
                });
            }
            Expr::Call { .. } => {
                return Err(BytecodeError {
                    message: "function calls are not yet supported by the bytecode backend".into(),
                });
            }
        }
        Ok(())
    }

    fn emit_placeholder_jump_if_false(&mut self) -> usize {
        let index = self.chunk.code.len();
        self.chunk.code.push(OpCode::JumpIfFalse(usize::MAX));
        index
    }

    fn emit_placeholder_jump(&mut self) -> usize {
        let index = self.chunk.code.len();
        self.chunk.code.push(OpCode::Jump(usize::MAX));
        index
    }

    fn patch_jump(&mut self, index: usize) -> Result<(), BytecodeError> {
        let target = self.chunk.code.len();
        let instruction = self.chunk.code.get_mut(index).ok_or_else(|| BytecodeError {
            message: "invalid jump patch location".into(),
        })?;
        *instruction = match instruction {
            OpCode::JumpIfFalse(_) => OpCode::JumpIfFalse(target),
            OpCode::Jump(_) => OpCode::Jump(target),
            _ => return Err(BytecodeError { message: "invalid jump instruction".into() }),
        };
        Ok(())
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

pub fn compile(program: &Program) -> Result<Chunk, BytecodeError> {
    Compiler::new().compile(program)
}
