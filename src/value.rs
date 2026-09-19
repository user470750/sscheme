use internment::Intern;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Value {
    Nil,
    Number(i32),
    Identifier(Intern<str>),
    List(Vec<Value>),
}
