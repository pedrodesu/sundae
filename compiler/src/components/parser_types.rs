#[derive(Debug, Clone)]
pub struct Type(pub Vec<String>);

#[derive(Debug, Clone)]
pub struct ArgumentName(pub String, pub Type);

#[derive(Debug, Clone)]
pub struct Name(pub String, pub Option<Type>);
