#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Number,
    String,
    Boolean,
    Void,
    Unknown,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Type::Number => "number",
            Type::String => "string",
            Type::Boolean => "bool",
            Type::Void => "void",
            Type::Unknown => "unknown",
        };
        f.write_str(name)
    }
}
