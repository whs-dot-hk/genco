//! ## Simple and flexible code generator (genco)
#![deny(missing_docs)]

#[macro_use]
mod macros;
mod con_;
mod custom;
mod element;
mod formatter;
mod quoted;
mod tokens;
mod into_tokens;
mod write_tokens;
mod cons;
pub mod csharp;
pub mod go;
pub mod java;
pub mod js;
pub mod python;
pub mod rust;
pub mod swift;

pub use self::custom::Custom;
pub use self::element::Element;
pub use self::java::Java;
pub use self::csharp::Csharp;
pub use self::rust::Rust;
pub use self::js::JavaScript;
pub use self::python::Python;
pub use self::tokens::Tokens;
pub use self::into_tokens::IntoTokens;
pub use self::formatter::{Formatter, IoFmt};
pub use self::write_tokens::WriteTokens;
pub use self::quoted::Quoted;
pub use self::cons::Cons;

#[cfg(test)]
mod tests {
    use tokens::Tokens;
    use rust::Rust;

    #[test]
    fn test_nested() {
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.push("fn foo() -> u32 {");
        toks.nested("return 42;");
        toks.push("}");

        let output = toks.to_string().unwrap();
        assert_eq!("fn foo() -> u32 {\n  return 42;\n}", output.as_str());
    }
}
