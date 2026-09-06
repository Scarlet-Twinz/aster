use std::collections::HashMap;
use crate::bytecode::{Chunk, Constant, OpCode};

#[derive(Debug, Clone, PartialEq)]
pub enum Value { Number(f64), String(String), Boolean(bool), Void }

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Value::Number(v) => write!(f, "{}", v), Value::String(v) => f.write_str(v), Value::Boolean(v) => write!(f, "{}", v), Value::Void => f.write_str("void") }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmError { pub message: String }

struct Frame { function_index: usize, ip: usize, locals: Vec<Value> }

pub struct Vm { stack: Vec<Value>, globals: HashMap<String, Value>, ip: usize, frames: Vec<Frame>, output: Vec<String> }

impl Vm {
    pub fn new() -> Self { Self { stack: Vec::new(), globals: HashMap::new(), ip: 0, frames: Vec::new(), output: Vec::new() } }

    pub fn run(mut self, chunk: &Chunk) -> Result<Vec<String>, VmError> {
        while self.ip < chunk.code.len() || !self.frames.is_empty() {
            let instruction = self.fetch_instruction(chunk)?;
            match instruction {
                OpCode::Constant(index) => { let c = self.current_constant(chunk, index)?; self.stack.push(Self::constant_to_value(c)); }
                OpCode::LoadGlobal(index) => { let name = chunk.names.get(index).ok_or_else(|| self.error("invalid global name index"))?; let value = self.globals.get(name).cloned().ok_or_else(|| self.error(&format!("undefined runtime variable '{}'", name)))?; self.stack.push(value); }
                OpCode::StoreGlobal(index) => { let name = chunk.names.get(index).ok_or_else(|| self.error("invalid global name index"))?.clone(); let value = self.pop()?; self.globals.insert(name, value); }
                OpCode::LoadLocal(index) => { let value = self.frames.last().and_then(|f| f.locals.get(index)).cloned().ok_or_else(|| self.error("invalid local variable index"))?; self.stack.push(value); }
                OpCode::StoreLocal(index) => { let value = self.pop()?; let frame = self.frames.last_mut().ok_or_else(|| self.error("local variable outside function"))?; if index >= frame.locals.len() { return Err(self.error("invalid local variable index")); } frame.locals[index] = value; }
                OpCode::Pop => { self.pop()?; }
                OpCode::Negate => { let v = self.pop()?; match v { Value::Number(n) => self.stack.push(Value::Number(-n)), other => return Err(self.error(&format!("cannot negate {}", other))) } }
                OpCode::Not => { let v = self.pop()?; match v { Value::Boolean(b) => self.stack.push(Value::Boolean(!b)), other => return Err(self.error(&format!("cannot apply '!' to {}", other))) } }
                OpCode::Add => self.binary_number(|a,b| a+b)?, OpCode::Subtract => self.binary_number(|a,b| a-b)?, OpCode::Multiply => self.binary_number(|a,b| a*b)?, OpCode::Divide => self.binary_number(|a,b| a/b)?, OpCode::Modulo => self.binary_number(|a,b| a%b)?,
                OpCode::Equal => self.binary_compare(|a,b| a==b)?, OpCode::NotEqual => self.binary_compare(|a,b| a!=b)?, OpCode::Less => self.binary_number_compare(|a,b| a<b)?, OpCode::LessEqual => self.binary_number_compare(|a,b| a<=b)?, OpCode::Greater => self.binary_number_compare(|a,b| a>b)?, OpCode::GreaterEqual => self.binary_number_compare(|a,b| a>=b)?,
                OpCode::And => self.binary_boolean(|a,b| a&&b)?, OpCode::Or => self.binary_boolean(|a,b| a||b)?,
                OpCode::JumpIfFalse(target) => { let condition=self.pop()?; match condition { Value::Boolean(false)=>self.jump(target,self.current_code_len(chunk))?, Value::Boolean(true)=>{}, other=>return Err(self.error(&format!("expected bool in conditional, got {}",other))) } }
                OpCode::Jump(target) => self.jump(target,self.current_code_len(chunk))?,
                OpCode::Print => { let value=self.pop()?; self.output.push(value.to_string()); println!("{}",value); }
                OpCode::Call(index) => self.call(chunk,index)?,
                OpCode::Return => self.return_from_function()?,
                OpCode::Halt => break,
            }
        }
        Ok(self.output)
    }

    fn fetch_instruction(&mut self, chunk:&Chunk)->Result<OpCode,VmError>{
        if let Some(frame)=self.frames.last_mut(){ let function=chunk.functions.get(frame.function_index).ok_or_else(||self.error("invalid function index"))?; if frame.ip>=function.code.len(){return Err(self.error("instruction pointer escaped function"));} let instruction=function.code[frame.ip]; frame.ip+=1; Ok(instruction) }
        else { if self.ip>=chunk.code.len(){return Err(self.error("instruction pointer escaped program"));} let instruction=chunk.code[self.ip]; self.ip+=1; Ok(instruction) }
    }

    fn current_constant<'a>(&self,chunk:&'a Chunk,index:usize)->Result<&'a Constant,VmError>{ if let Some(frame)=self.frames.last(){ chunk.functions.get(frame.function_index).and_then(|f|f.constants.get(index)).ok_or_else(||self.error("invalid function constant index")) } else { chunk.constants.get(index).ok_or_else(||self.error("invalid constant index")) } }
    fn call(&mut self,chunk:&Chunk,function_index:usize)->Result<(),VmError>{ let function=chunk.functions.get(function_index).ok_or_else(||self.error("invalid function index"))?; let arity=function.arity; if self.stack.len()<arity{return Err(self.error("not enough arguments for function call"));} let mut args=Vec::with_capacity(arity); for _ in 0..arity{args.push(self.pop()?);} args.reverse(); self.frames.push(Frame{function_index,ip:0,locals:args}); Ok(()) }
    fn return_from_function(&mut self)->Result<(),VmError>{ let value=self.pop()?; if self.frames.pop().is_none(){return Err(self.error("return executed outside function"));} self.stack.push(value); Ok(()) }
    fn current_code_len(&self,chunk:&Chunk)->usize{ self.frames.last().and_then(|f|chunk.functions.get(f.function_index)).map(|f|f.code.len()).unwrap_or(chunk.code.len()) }
    fn constant_to_value(c:&Constant)->Value{match c{Constant::Number(v)=>Value::Number(*v),Constant::String(v)=>Value::String(v.clone()),Constant::Boolean(v)=>Value::Boolean(*v),Constant::Void=>Value::Void}}
    fn binary_number<F>(&mut self,op:F)->Result<(),VmError>where F:FnOnce(f64,f64)->f64{let r=self.pop()?;let l=self.pop()?;match(l,r){(Value::Number(a),Value::Number(b))=>{self.stack.push(Value::Number(op(a,b)));Ok(())},(a,b)=>Err(self.error(&format!("arithmetic requires numbers, got {} and {}",a,b)))}}
    fn binary_number_compare<F>(&mut self,op:F)->Result<(),VmError>where F:FnOnce(f64,f64)->bool{let r=self.pop()?;let l=self.pop()?;match(l,r){(Value::Number(a),Value::Number(b))=>{self.stack.push(Value::Boolean(op(a,b)));Ok(())},(a,b)=>Err(self.error(&format!("comparison requires numbers, got {} and {}",a,b)))}}
    fn binary_compare<F>(&mut self,op:F)->Result<(),VmError>where F:FnOnce(&Value,&Value)->bool{let r=self.pop()?;let l=self.pop()?;self.stack.push(Value::Boolean(op(&l,&r)));Ok(())}
    fn binary_boolean<F>(&mut self,op:F)->Result<(),VmError>where F:FnOnce(bool,bool)->bool{let r=self.pop()?;let l=self.pop()?;match(l,r){(Value::Boolean(a),Value::Boolean(b))=>{self.stack.push(Value::Boolean(op(a,b)));Ok(())},(a,b)=>Err(self.error(&format!("logical operators require bools, got {} and {}",a,b)))}}
    fn jump(&mut self,target:usize,len:usize)->Result<(),VmError>{if target>=len{return Err(self.error("jump target is outside bytecode"));}if let Some(frame)=self.frames.last_mut(){frame.ip=target}else{self.ip=target}Ok(())}
    fn pop(&mut self)->Result<Value,VmError>{self.stack.pop().ok_or_else(||self.error("stack underflow"))}
    fn error(&self,message:&str)->VmError{VmError{message:message.into()}}
}

impl Default for Vm{fn default()->Self{Self::new()}}
pub fn run(chunk:&Chunk)->Result<Vec<String>,VmError>{Vm::new().run(chunk)}
