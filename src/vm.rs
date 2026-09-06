use std::collections::HashMap;
use crate::bytecode::{Chunk, Constant, OpCode};

#[derive(Debug, Clone, PartialEq)]
pub enum Value { Number(f64), String(String), Boolean(bool), Void }
impl std::fmt::Display for Value { fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{match self{Value::Number(v)=>write!(f,"{}",v),Value::String(v)=>f.write_str(v),Value::Boolean(v)=>write!(f,"{}",v),Value::Void=>f.write_str("void")}} }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmError { pub message:String }
struct Frame { function_index:usize, ip:usize, locals:Vec<Value> }
pub struct Vm { stack:Vec<Value>, globals:HashMap<String,Value>, ip:usize, frames:Vec<Frame>, output:Vec<String> }
impl Vm {
 pub fn new()->Self{Self{stack:Vec::new(),globals:HashMap::new(),ip:0,frames:Vec::new(),output:Vec::new()}}
 pub fn run(mut self,chunk:&Chunk)->Result<Vec<String>,VmError>{
  while self.ip<chunk.code.len()||!self.frames.is_empty(){
   let instruction=self.fetch_instruction(chunk)?;
   match instruction{
    OpCode::Constant(i)=>{let c=self.current_constant(chunk,i)?;self.stack.push(Self::constant_to_value(c));}
    OpCode::LoadGlobal(i)=>{let name=chunk.names.get(i).ok_or_else(||self.error("invalid global name index"))?;let value=self.globals.get(name).cloned().ok_or_else(||self.error(&format!("undefined runtime variable '{}'",name)))?;self.stack.push(value);}
    OpCode::StoreGlobal(i)=>{let name=chunk.names.get(i).ok_or_else(||self.error("invalid global name index"))?.clone();let value=self.pop()?;self.globals.insert(name,value);}
    OpCode::LoadLocal(i)=>{let value=match self.frames.last(){Some(f)=>f.locals.get(i).cloned(),None=>None}.ok_or_else(||self.error("invalid local variable index"))?;self.stack.push(value);}
    OpCode::StoreLocal(i)=>{let value=self.pop()?;match self.frames.last_mut(){Some(f) if i<f.locals.len()=>f.locals[i]=value,Some(_)=>return Err(self.error("invalid local variable index")),None=>return Err(self.error("local variable outside function"))}}
    OpCode::Pop=>{self.pop()?;}
    OpCode::Negate=>{let v=self.pop()?;match v{Value::Number(n)=>self.stack.push(Value::Number(-n)),other=>return Err(self.error(&format!("cannot negate {}",other)))}}
    OpCode::Not=>{let v=self.pop()?;match v{Value::Boolean(b)=>self.stack.push(Value::Boolean(!b)),other=>return Err(self.error(&format!("cannot apply '!' to {}",other)))}}
    OpCode::Add=>self.binary_number(|a,b|a+b)?,OpCode::Subtract=>self.binary_number(|a,b|a-b)?,OpCode::Multiply=>self.binary_number(|a,b|a*b)?,OpCode::Divide=>self.binary_number(|a,b|a/b)?,OpCode::Modulo=>self.binary_number(|a,b|a%b)?,
    OpCode::Equal=>self.binary_compare(|a,b|a==b)?,OpCode::NotEqual=>self.binary_compare(|a,b|a!=b)?,OpCode::Less=>self.binary_number_compare(|a,b|a<b)?,OpCode::LessEqual=>self.binary_number_compare(|a,b|a<=b)?,OpCode::Greater=>self.binary_number_compare(|a,b|a>b)?,OpCode::GreaterEqual=>self.binary_number_compare(|a,b|a>=b)?,
    OpCode::And=>self.binary_boolean(|a,b|a&&b)?,OpCode::Or=>self.binary_boolean(|a,b|a||b)?,
    OpCode::JumpIfFalse(t)=>{let c=self.pop()?;match c{Value::Boolean(false)=>self.jump(t,self.current_code_len(chunk))?,Value::Boolean(true)=>{},other=>return Err(self.error(&format!("expected bool in conditional, got {}",other)))}}
    OpCode::Jump(t)=>self.jump(t,self.current_code_len(chunk))?,
    OpCode::Print=>{let v=self.pop()?;self.output.push(v.to_string());println!("{}",v);}
    OpCode::Call(i)=>self.call(chunk,i)?,
    OpCode::Return=>self.return_from_function()?,
    OpCode::Halt=>break,
   }
  }
  Ok(self.output)
 }
 fn fetch_instruction(&mut self,chunk:&Chunk)->Result<OpCode,VmError>{if self.frames.is_empty(){if self.ip>=chunk.code.len(){return Err(self.error("instruction pointer escaped program"));}let i=chunk.code[self.ip];self.ip+=1;Ok(i)}else{let f=self.frames.last_mut().expect("frame exists");let idx=f.function_index;let len=match chunk.functions.get(idx){Some(x)=>x.code.len(),None=>return Err(self.error("invalid function index"))};if f.ip>=len{return Err(self.error("instruction pointer escaped function"));}let i=chunk.functions[idx].code[f.ip];f.ip+=1;Ok(i)}}
 fn current_constant<'a>(&self,chunk:&'a Chunk,index:usize)->Result<&'a Constant,VmError>{if let Some(f)=self.frames.last(){chunk.functions.get(f.function_index).and_then(|x|x.constants.get(index)).ok_or_else(||self.error("invalid function constant index"))}else{chunk.constants.get(index).ok_or_else(||self.error("invalid constant index"))}}
 fn call(&mut self,chunk:&Chunk,index:usize)->Result<(),VmError>{let(arity,local_count)=match chunk.functions.get(index){Some(f)=>(f.arity,f.local_names.len()),None=>return Err(self.error("invalid function index"))};if self.stack.len()<arity{return Err(self.error("not enough arguments for function call"));}let mut args=Vec::with_capacity(arity);for _ in 0..arity{args.push(self.pop()?);}args.reverse();let mut locals=vec![Value::Void;local_count];for(i,arg)in args.into_iter().enumerate(){locals[i]=arg;}self.frames.push(Frame{function_index:index,ip:0,locals});Ok(())}
 fn return_from_function(&mut self)->Result<(),VmError>{let value=self.pop()?;if self.frames.pop().is_none(){return Err(self.error("return executed outside function"));}self.stack.push(value);Ok(())}
 fn current_code_len(&self,chunk:&Chunk)->usize{self.frames.last().and_then(|f|chunk.functions.get(f.function_index)).map(|f|f.code.len()).unwrap_or(chunk.code.len())}
 fn constant_to_value(c:&Constant)->Value{match c{Constant::Number(v)=>Value::Number(*v),Constant::String(v)=>Value::String(v.clone()),Constant::Boolean(v)=>Value::Boolean(*v),Constant::Void=>Value::Void}}
 fn binary_number<F>(&mut self,op:F)->Result<(),VmError>where F:FnOnce(f64,f64)->f64{let r=self.pop()?;let l=self.pop()?;match(l,r){(Value::Number(a),Value::Number(b))=>{self.stack.push(Value::Number(op(a,b)));Ok(())},(a,b)=>Err(self.error(&format!("arithmetic requires numbers, got {} and {}",a,b)))}}
 fn binary_number_compare<F>(&mut self,op:F)->Result<(),VmError>where F:FnOnce(f64,f64)->bool{let r=self.pop()?;let l=self.pop()?;match(l,r){(Value::Number(a),Value::Number(b))=>{self.stack.push(Value::Boolean(op(a,b)));Ok(())},(a,b)=>Err(self.error(&format!("comparison requires numbers, got {} and {}",a,b)))}}
 fn binary_compare<F>(&mut self,op:F)->Result<(),VmError>where F:FnOnce(&Value,&Value)->bool{let r=self.pop()?;let l=self.pop()?;self.stack.push(Value::Boolean(op(&l,&r)));Ok(())}
 fn binary_boolean<F>(&mut self,op:F)->Result<(),VmError>where F:FnOnce(bool,bool)->bool{let r=self.pop()?;let l=self.pop()?;match(l,r){(Value::Boolean(a),Value::Boolean(b))=>{self.stack.push(Value::Boolean(op(a,b)));Ok(())},(a,b)=>Err(self.error(&format!("logical operators require bools, got {} and {}",a,b)))}}
 fn jump(&mut self,target:usize,len:usize)->Result<(),VmError>{if target>=len{return Err(self.error("jump target is outside bytecode"));}if let Some(f)=self.frames.last_mut(){f.ip=target}else{self.ip=target}Ok(())}
 fn pop(&mut self)->Result<Value,VmError>{self.stack.pop().ok_or_else(||self.error("stack underflow"))}
 fn error(&self,message:&str)->VmError{VmError{message:message.into()}}
}
impl Default for Vm{fn default()->Self{Self::new()}}
pub fn run(chunk:&Chunk)->Result<Vec<String>,VmError>{Vm::new().run(chunk)}
