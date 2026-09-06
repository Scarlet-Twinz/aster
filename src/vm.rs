use std::collections::HashMap;

use crate::bytecode::{Chunk, Constant, OpCode};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(value) => write!(f, "{}", value),
            Value::String(value) => f.write_str(value),
            Value::Boolean(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmError {
    pub message: String,
}

pub struct Vm {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    ip: usize,
    output: Vec<String>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
            ip: 0,
            output: Vec::new(),
        }
    }

    pub fn run(mut self, chunk: &Chunk) -> Result<Vec<String>, VmError> {
        while self.ip < chunk.code.len() {
            let instruction = chunk.code[self.ip];
            self.ip += 1;

            match instruction {
                OpCode::Constant(index) => {
                    let constant = chunk.constants.get(index).ok_or_else(|| self.error("invalid constant index"))?;
                    self.stack.push(Self::constant_to_value(constant));
                }
                OpCode::LoadGlobal(index) => {
                    let name = chunk.names.get(index).ok_or_else(|| self.error("invalid global name index"))?;
                    let value = self.globals.get(name).cloned().ok_or_else(|| {
                        self.error(&format!("undefined runtime variable '{}'", name))
                    })?;
                    self.stack.push(value);
                }
                OpCode::StoreGlobal(index) => {
                    let name = chunk.names.get(index).ok_or_else(|| self.error("invalid global name index"))?.clone();
                    let value = self.pop()?;
                    self.globals.insert(name, value);
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::Negate => {
                    let value = self.pop()?;
                    match value {
                        Value::Number(value) => self.stack.push(Value::Number(-value)),
                        other => return Err(self.error(&format!("cannot negate {}", other))),
                    }
                }
                OpCode::Not => {
                    let value = self.pop()?;
                    match value {
                        Value::Boolean(value) => self.stack.push(Value::Boolean(!value)),
                        other => return Err(self.error(&format!("cannot apply '!' to {}", other))),
                    }
                }
                OpCode::Add => self.binary_number(|a, b| a + b)?,
                OpCode::Subtract => self.binary_number(|a, b| a - b)?,
                OpCode::Multiply => self.binary_number(|a, b| a * b)?,
                OpCode::Divide => self.binary_number(|a, b| a / b)?,
                OpCode::Modulo => self.binary_number(|a, b| a % b)?,
                OpCode::Equal => self.binary_compare(|a, b| a == b)?,
                OpCode::NotEqual => self.binary_compare(|a, b| a != b)?,
                OpCode::Less => self.binary_number_compare(|a, b| a < b)?,
                OpCode::LessEqual => self.binary_number_compare(|a, b| a <= b)?,
                OpCode::Greater => self.binary_number_compare(|a, b| a > b)?,
                OpCode::GreaterEqual => self.binary_number_compare(|a, b| a >= b)?,
                OpCode::And => self.binary_boolean(|a, b| a && b)?,
                OpCode::Or => self.binary_boolean(|a, b| a || b)?,
                OpCode::JumpIfFalse(target) => {
                    let condition = self.pop()?;
                    match condition {
                        Value::Boolean(false) => self.jump(target, chunk.code.len())?,
                        Value::Boolean(true) => {}
                        other => return Err(self.error(&format!("expected bool in conditional, got {}", other))),
                    }
                }
                OpCode::Jump(target) => self.jump(target, chunk.code.len())?,
                OpCode::Print => {
                    let value = self.pop()?;
                    self.output.push(value.to_string());
                    println!("{}", value);
                }
                OpCode::Halt => break,
            }
        }
        Ok(self.output)
    }

    fn constant_to_value(constant: &Constant) -> Value {
        match constant {
            Constant::Number(value) => Value::Number(*value),
            Constant::String(value) => Value::String(value.clone()),
            Constant::Boolean(value) => Value::Boolean(*value),
        }
    }

    fn binary_number<F>(&mut self, operation: F) -> Result<(), VmError>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        let right = self.pop()?;
        let left = self.pop()?;
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => {
                self.stack.push(Value::Number(operation(left, right)));
                Ok(())
            }
            (left, right) => Err(self.error(&format!("arithmetic requires numbers, got {} and {}", left, right))),
        }
    }

    fn binary_number_compare<F>(&mut self, operation: F) -> Result<(), VmError>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let right = self.pop()?;
        let left = self.pop()?;
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => {
                self.stack.push(Value::Boolean(operation(left, right)));
                Ok(())
            }
            (left, right) => Err(self.error(&format!("comparison requires numbers, got {} and {}", left, right))),
        }
    }

    fn binary_compare<F>(&mut self, operation: F) -> Result<(), VmError>
    where
        F: FnOnce(&Value, &Value) -> bool,
    {
        let right = self.pop()?;
        let left = self.pop()?;
        self.stack.push(Value::Boolean(operation(&left, &right)));
        Ok(())
    }

    fn binary_boolean<F>(&mut self, operation: F) -> Result<(), VmError>
    where
        F: FnOnce(bool, bool) -> bool,
    {
        let right = self.pop()?;
        let left = self.pop()?;
        match (left, right) {
            (Value::Boolean(left), Value::Boolean(right)) => {
                self.stack.push(Value::Boolean(operation(left, right)));
                Ok(())
            }
            (left, right) => Err(self.error(&format!("logical operators require bools, got {} and {}", left, right))),
        }
    }

    fn jump(&mut self, target: usize, code_len: usize) -> Result<(), VmError> {
        if target >= code_len {
            return Err(self.error("jump target is outside bytecode"));
        }
        self.ip = target;
        Ok(())
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or_else(|| self.error("stack underflow"))
    }

    fn error(&self, message: &str) -> VmError {
        VmError { message: message.into() }
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run(chunk: &Chunk) -> Result<Vec<String>, VmError> {
    Vm::new().run(chunk)
}
