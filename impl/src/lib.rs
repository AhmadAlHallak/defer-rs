use syn::{
    parse::{Parse, ParseStream},
    Stmt,
};

// This will be no longer needed when either the `index` macro meta variable expression (#122808) or `std::ops::Fn::call` method land in stable,
#[doc(hidden)]
#[proc_macro]
pub fn call_indexed(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::ExprCall);

    let func = input.func;
    let args = input.args.iter();
    let i = (0..args.len()).map(syn::Index::from);
    quote::quote! {
        {
            #func(#(___deferred_code_captured_args.#i, )*);
        }
    }
    .into()
}

struct DeferStmtExpr {
    move_kw: Option<syn::token::Move>,
    deferred: Vec<Stmt>,
}

impl Parse for DeferStmtExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            move_kw: input.parse()?,
            deferred: input.call(syn::Block::parse_within)?,
        })
    }
}


/// A macro for deferring execution of code until the closest scope containing a previously invoked [`defer_scope_init!`] macro ends.
///
/// Use `defer_scope!` when you want to defer execution not to the end of the current active scope, but to the end of a larger parent scope. 
/// The specific parent scope is determined by invoking `defer_scope_init!`.
///
/// **Important Notes**:
/// - The [`defer_scope_init!`] macro **must** be invoked before using `defer_scope!`, and both macros must share a scope.
/// - You can invoke the `defer_scope!` macro multiple times for a given `defer_scope_init!` invocation.
///
/// # Examples
/// 
/// ## Basic usage:
///
/// ```rust
/// use defer_rs::{defer_scope, defer_scope_init};
/// 
/// defer_scope_init!();
/// defer_scope! {
///     println!("This will be executed when `defer_scope_init!()`'s scope exits.");
/// }
/// ```
/// ### Expands to:
/// ```rust
/// let mut ___deferred_code_group = ::defer_rs::DeferGroup::new();
///  ___deferred_code_group.add(Box::new(( || { 
///     println!("This will be executed when `defer_scope_init!()`'s scope exits.");
/// })));
/// ```
/// 
/// Ignoring the ability to specify the scope and the need for invoking `defer_scope_init!` beforehand, 
/// `defer_scope!` is otherwise identical to [`defer!`](https://docs.rs/defer_rs/latest/defer_rs/macro.defer.html).
///
/// For more usage examples, refer to the documentation for the [`defer!`](https://docs.rs/defer_rs/latest/defer_rs/macro.defer.html) macro, 
/// simply replace `defer!` with `defer_scope!` and add an invocation of [`defer_scope_init!`] beforehand.
///
/// See also: [`DeferGroup`](https://docs.rs/defer_rs/latest/defer_rs/struct.DeferGroup.html), [`defer_scope_init!`], and [`defer!`](https://docs.rs/defer_rs/latest/defer_rs/macro.defer.html).
// THIS DOC COMMENT MUST BE KEPT IN SYNC WITH THE DOC COMMENT ON THE FAKE `cfg(doc)` `defer_scope!` DECLARTIVE MACRO IN THE PARENT `defer_rs` CRATE!
#[doc(hidden)]
// A proc_macro is used instead of `macro_rules` to bypass identifier hygiene
#[proc_macro]
pub fn defer_scope(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::Result<syn::ExprCall> = syn::parse(input.clone());
    if let Ok(call) = ast {
        let func = call.func;
        let args = call.args.iter();
        let i = (0..args.len()).map(syn::Index::from);
        quote::quote! {

            let ___deferred_code_captured_args = ( #( #args, )* );
            {
                ___deferred_code_group.add(::std::boxed::Box::new( move || {
                    #func(#(___deferred_code_captured_args.#i, )*);
                }));
            }
        }
        .into()
    } else {
        let DeferStmtExpr { move_kw, deferred } = syn::parse(input).unwrap();
        quote::quote! {
            {
                ___deferred_code_group.add(::std::boxed::Box::new(#move_kw || {
                    #(#deferred)*;
                }));
            }
        }
        .into()
    }
}


/// Initializes a [DeferGroup], which is an empty collection of closures to run at the end of the scope containing the invocation.
/// It provides no functionality by itself and should be called before any [defer_scope!] invocation(s).
/// 
/// No arguments should be passed to the macro invocation.
/// 
/// # Usage
/// 
/// ```rust
/// defer_rs::defer_scope_init!();
/// ```
/// ## Expands to:
/// ```rust
/// let mut ___deferred_code_group = ::defer_rs::DeferGroup::new();
/// ```
///
/// For more detailed examples, refer to the documentation for [defer_scope!].
///
/// See also: [`DeferGroup`](https://docs.rs/defer_rs/latest/defer_rs/struct.DeferGroup.html), [`defer_scope!`], and [`defer!`](https://docs.rs/defer_rs/latest/defer_rs/macro.defer.html).
// THIS DOC COMMENT MUST BE KEPT IN SYNC WITH THE DOC COMMENT ON THE FAKE `cfg(doc)` `defer_scope_init!` DECLARTIVE MACRO IN THE PARENT `defer_rs` CRATE!
#[doc(hidden)]
// This is used to bypass `macro_rules` identifier hygiene
#[proc_macro]
pub fn defer_scope_init(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if !input.is_empty() {
        return quote::quote! {compile_error!("deferfn_init! doesn't take any arguments")}.into();
    }
    "let mut ___deferred_code_group = ::defer_rs::DeferGroup::new();"
        .parse()
        .unwrap()
}
