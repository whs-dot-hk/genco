//! Specialization for JavaScript code generation.
//!
//! # Examples
//!
//! Basic example:
//!
//! ```rust
//! #[feature(proc_macro_hygiene)]
//! use genco::prelude::*;
//!
//! let toks: js::Tokens = quote! {
//!     function foo(v) {
//!         return v + ", World";
//!     }
//!
//!     foo("Hello");
//! };
//!
//! assert_eq!(
//!     vec![
//!         "function foo(v) {",
//!         "    return v + \", World\";",
//!         "}",
//!         "",
//!         "foo(\"Hello\");",
//!     ],
//!     toks.to_file_vec().unwrap()
//! );
//! ```
//!
//! String quoting in JavaScript:
//!
//! ```rust
//! #[feature(proc_macro_hygiene)]
//! use genco::prelude::*;
//!
//! let toks: js::Tokens = quote!(#("hello \n world".quoted()));
//! assert_eq!("\"hello \\n world\"", toks.to_string().unwrap());
//! ```

use crate::{Formatter, ItemStr, Lang, LangItem};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write};

/// Tokens container specialization for Rust.
pub type Tokens = crate::Tokens<JavaScript>;

impl_type_basics!(JavaScript, TypeEnum<'a>, TypeTrait, TypeBox, TypeArgs, {Import, ImportDefault, Local});

/// Trait implemented by all types.
pub trait TypeTrait: 'static + fmt::Debug + LangItem<JavaScript> {
    /// Coerce trait into an enum that can be used for type-specific operations.
    fn as_enum(&self) -> TypeEnum<'_>;
}

/// An imported item in JavaScript.
///
/// Created using the [import()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// Module of the imported name.
    module: ItemStr,
    /// Name imported.
    name: ItemStr,
    /// Alias of an imported item.
    ///
    /// If this is set, you'll get an import like:
    ///
    /// ```text
    /// import {<name> as <alias>} from <module>
    /// ```
    alias: Option<ItemStr>,
}

impl Import {
    /// Alias of an imported item.
    ///
    /// If this is set, you'll get an import like:
    ///
    /// ```text
    /// import {<name> as <alias>} from <module>
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// #![feature(proc_macro_hygiene)]
    /// use genco::prelude::*;
    ///
    /// let a = js::import("collections", "vec");
    /// let b = js::import("collections", "vec").alias("list");
    ///
    /// let toks = quote! {
    ///     #a
    ///     #b
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import {vec, vec as list} from \"collections\";",
    ///         "",
    ///         "vec",
    ///         "list",
    ///     ],
    ///     toks.to_file_vec().unwrap()
    /// );
    /// ```
    pub fn alias<N: Into<ItemStr>>(self, alias: N) -> Self {
        Self {
            alias: Some(alias.into()),
            ..self
        }
    }
}

impl TypeTrait for Import {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Import(self)
    }
}

impl LangItem<JavaScript> for Import {
    fn format(&self, out: &mut Formatter, _: &mut (), _: usize) -> fmt::Result {
        if let Some(alias) = &self.alias {
            out.write_str(alias)?;
        } else {
            out.write_str(&self.name)?;
        }

        Ok(())
    }

    fn as_import(&self) -> Option<&dyn TypeTrait> {
        Some(self)
    }
}

/// The default imported item.
///
/// Created using the [import_default()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ImportDefault {
    /// Module of the imported name.
    module: ItemStr,
    /// Name imported.
    name: ItemStr,
}

impl TypeTrait for ImportDefault {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::ImportDefault(self)
    }
}

impl LangItem<JavaScript> for ImportDefault {
    fn format(&self, out: &mut Formatter, _: &mut (), _: usize) -> fmt::Result {
        out.write_str(&self.name)
    }

    fn as_import(&self) -> Option<&dyn TypeTrait> {
        Some(self)
    }
}

/// A local name.
///
/// Created using the [local()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Local {
    /// The local name.
    name: ItemStr,
}

impl TypeTrait for Local {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Local(self)
    }
}

impl LangItem<JavaScript> for Local {
    fn format(&self, out: &mut Formatter, _: &mut (), _: usize) -> fmt::Result {
        out.write_str(&self.name)
    }

    fn as_import(&self) -> Option<&dyn TypeTrait> {
        None
    }
}

/// JavaScript language specialization.
pub struct JavaScript(());

impl JavaScript {
    /// Translate imports into the necessary tokens.
    fn imports(tokens: &Tokens, output: &mut Tokens) {
        use crate as genco;
        use crate::prelude::*;

        let mut modules = BTreeMap::<&ItemStr, Module<'_>>::new();

        for import in tokens.walk_imports() {
            match import.as_enum() {
                TypeEnum::Import(this) => {
                    let module = modules.entry(&this.module).or_default();

                    module.set.insert(match &this.alias {
                        None => ImportedElement::Plain(&this.name),
                        Some(alias) => ImportedElement::Aliased(&this.name, alias),
                    });
                }
                TypeEnum::ImportDefault(this) => {
                    let module = modules.entry(&this.module).or_default();
                    module.default_import = Some(&this.name);
                }
                _ => (),
            }
        }

        if modules.is_empty() {
            return;
        }

        for (name, module) in modules {
            output.push();
            quote_in! { output =>
                import #{ *tokens => {
                    if let Some(default) = module.default_import {
                        tokens.append(ItemStr::from(default));

                        if !module.set.is_empty() {
                            tokens.append(",");
                            tokens.spacing();
                        }
                    }

                    if !module.set.is_empty() {
                        tokens.append("{");

                        let mut it = module.set.iter().peekable();

                        while let Some(el) = it.next() {
                            match *el {
                                ImportedElement::Plain(name) => {
                                    tokens.append(name);
                                },
                                ImportedElement::Aliased(name, alias) => {
                                    quote_in!(tokens => #name as #alias);
                                }
                            }

                            if it.peek().is_some() {
                                tokens.append(",");
                                tokens.spacing();
                            }
                        }

                        tokens.append("}");
                    }
                }} from #(name.quoted());
            };
        }

        output.push_line();

        #[derive(Default)]
        struct Module<'a> {
            default_import: Option<&'a ItemStr>,
            set: BTreeSet<ImportedElement<'a>>,
        }

        #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
        enum ImportedElement<'a> {
            Plain(&'a ItemStr),
            Aliased(&'a ItemStr, &'a ItemStr),
        }
    }
}

impl Lang for JavaScript {
    type Config = ();
    type Import = dyn TypeTrait;

    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_char('"')?;

        for c in input.chars() {
            match c {
                '\t' => out.write_str("\\t")?,
                '\u{0007}' => out.write_str("\\b")?,
                '\n' => out.write_str("\\n")?,
                '\r' => out.write_str("\\r")?,
                '\u{0014}' => out.write_str("\\f")?,
                '\'' => out.write_str("\\'")?,
                '"' => out.write_str("\\\"")?,
                '\\' => out.write_str("\\\\")?,
                c => out.write_char(c)?,
            };
        }

        out.write_char('"')?;

        Ok(())
    }

    fn write_file(
        tokens: Tokens,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        let mut toks = Tokens::new();
        Self::imports(&tokens, &mut toks);
        toks.extend(tokens);
        toks.format(out, config, level)
    }
}

/// Import an element from a module
///
/// # Examples
///
/// ```rust
/// #![feature(proc_macro_hygiene)]
/// use genco::prelude::*;
///
/// let a = js::import("collections", "vec");
/// let b = js::import("collections", "vec").alias("list");
///
/// let toks = quote! {
///     #a
///     #b
/// };
///
/// assert_eq!(
///     vec![
///         "import {vec, vec as list} from \"collections\";",
///         "",
///         "vec",
///         "list",
///     ],
///     toks.to_file_vec().unwrap()
/// );
/// ```
pub fn import<M, N>(module: M, name: N) -> Import
where
    M: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Import {
        module: module.into(),
        name: name.into(),
        alias: None,
    }
}

/// Import the default element from the specified module.
///
/// Note that the default element may only be aliased once, so multiple aliases
/// will cause an error.
///
/// # Examples
///
/// ```rust
/// #![feature(proc_macro_hygiene)]
/// use genco::prelude::*;
///
/// let a = js::import_default("collections", "defaultVec");
/// let b = js::import("collections", "vec");
/// let c = js::import("collections", "vec").alias("list");
///
/// let toks = quote! {
///     #a
///     #b
///     #c
/// };
///
/// assert_eq!(
///     vec![
///         "import defaultVec, {vec, vec as list} from \"collections\";",
///         "",
///         "defaultVec",
///         "vec",
///         "list",
///     ],
///     toks.to_file_vec().unwrap()
/// );
/// ```
pub fn import_default<M, N>(module: M, name: N) -> ImportDefault
where
    M: Into<ItemStr>,
    N: Into<ItemStr>,
{
    ImportDefault {
        module: module.into(),
        name: name.into(),
    }
}

/// Setup a local element.
///
/// # Examples
///
/// ```rust
/// #![feature(proc_macro_hygiene)]
/// use genco::prelude::*;
///
/// let toks = quote!(#(js::local("MyType")));
/// assert_eq!(vec!["MyType"], toks.to_file_vec().unwrap());
/// ```
pub fn local<N>(name: N) -> Local
where
    N: Into<ItemStr>,
{
    Local { name: name.into() }
}
