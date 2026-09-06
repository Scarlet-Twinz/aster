use crate::bytecode::{Chunk, Constant, Function, OpCode};

pub fn disassemble(chunk: &Chunk) -> String {
    let mut output = String::new();
    output.push_str("== ASTER main ==\n");
    format_code(&mut output, &chunk.code, &chunk.constants, &chunk.names);

    for function in &chunk.functions {
        output.push_str(&format!("\n== ASTER function {}({}) ==\n", function.name, function.arity));
        format_function(&mut output, function);
    }

    output
}

fn format_function(output: &mut String, function: &Function) {
    if !function.local_names.is_empty() {
        output.push_str(&format!("locals: {}\n", function.local_names.join(", ")));
    }
    format_code(output, &function.code, &function.constants, &[]);
}

fn format_code(output: &mut String, code: &[OpCode], constants: &[Constant], names: &[String]) {
    for (offset, instruction) in code.iter().enumerate() {
        output.push_str(&format!("{:04}  {}\n", offset, format_instruction(instruction, constants, names)));
    }
}

fn format_instruction(instruction: &OpCode, constants: &[Constant], names: &[String]) -> String {
    match instruction {
        OpCode::Constant(index) => format!("CONSTANT {} ({})", index, constant_name(constants, *index)),
        OpCode::LoadGlobal(index) => format!("LOAD_GLOBAL {} ({})", index, name_name(names, *index)),
        OpCode::StoreGlobal(index) => format!("STORE_GLOBAL {} ({})", index, name_name(names, *index)),
        OpCode::LoadLocal(index) => format!("LOAD_LOCAL {}", index),
        OpCode::StoreLocal(index) => format!("STORE_LOCAL {}", index),
        OpCode::Pop => "POP".into(),
        OpCode::Negate => "NEGATE".into(),
        OpCode::Not => "NOT".into(),
        OpCode::Add => "ADD".into(),
        OpCode::Subtract => "SUBTRACT".into(),
        OpCode::Multiply => "MULTIPLY".into(),
        OpCode::Divide => "DIVIDE".into(),
        OpCode::Modulo => "MODULO".into(),
        OpCode::Equal => "EQUAL".into(),
        OpCode::NotEqual => "NOT_EQUAL".into(),
        OpCode::Less => "LESS".into(),
        OpCode::LessEqual => "LESS_EQUAL".into(),
        OpCode::Greater => "GREATER".into(),
        OpCode::GreaterEqual => "GREATER_EQUAL".into(),
        OpCode::And => "AND".into(),
        OpCode::Or => "OR".into(),
        OpCode::JumpIfFalse(target) => format!("JUMP_IF_FALSE {}", target),
        OpCode::Jump(target) => format!("JUMP {}", target),
        OpCode::Print => "PRINT".into(),
        OpCode::Call(index) => format!("CALL function#{}", index),
        OpCode::Return => "RETURN".into(),
        OpCode::Halt => "HALT".into(),
    }
}

fn constant_name(constants: &[Constant], index: usize) -> String {
    constants
        .get(index)
        .map(|constant| format!("{:?}", constant))
        .unwrap_or_else(|| "<invalid constant>".into())
}

fn name_name(names: &[String], index: usize) -> String {
    names.get(index).cloned().unwrap_or_else(|| "<invalid name>".into())
}
