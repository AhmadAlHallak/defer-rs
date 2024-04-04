#![doc = include_str!("../README.md")]

// This `extern` is to facilitate easier crate resolution in tests for the proc generated code
extern crate self as defer_rs;

#[cfg(not(doc))]
pub use defer_rs_impl::{defer_scope, defer_scope_init};

/// A utility struct for deferred execution of a closure.
///
/// The `Defer` struct allows you to execute a closure once the `Defer` instance goes out of scope.
/// It is commonly used for resource management, cleanup, or any other deferred actions.
///
/// **Note: `Defer` MUST be bound to a variable to function properly; otherwise, it will be dropped immediately, executing the enclosed closure!**
///
/// # Example
///
/// ```
/// use defer_rs::*;
///
/// # fn acquire_resource(){}
/// # fn release_resource(_: ()){}
///
/// fn main() {
///     let resource = acquire_resource();
///
///     // Create a `Defer` instance with a closure that will be executed when it goes out of scope.
///     let cleanup = Defer::new(|| {
///         release_resource(resource);
///     });
///
///     // ... do some work ...
///
///     // The closure will be executed automatically when `cleanup` goes out of scope.
/// }
/// ```
///
/// See also: [`defer!`], and [`DeferGroup`].
#[must_use = "Defer MUST be bound to a variable to function properly; otherwise, it will be dropped immediately, executing the enclosed closure!"]
pub struct Defer<T: FnOnce()>(Option<T>);

impl<T: FnOnce()> Defer<T> {
    /// Creates a new `Defer` instance with the given deferred closure.
    ///
    /// The closure will be executed when the `Defer` instance goes out of scope.
    ///
    /// **Note: `Defer` MUST be bound to a variable to function properly; otherwise, it will be dropped immediately, executing the enclosed closure!**
    ///
    /// # Example
    ///
    /// ```rust
    /// use defer_rs::Defer;
    ///
    /// let defer_instance = Defer::new(|| {
    ///     println!("Deferred action executed!");
    /// });
    ///
    /// // ... other code ...
    ///
    /// // The deferred action will be executed when `defer_instance` goes out of scope.
    /// ```
    pub fn new(deferred: T) -> Self {
        Self(Some(deferred))
    }
}

impl<T: FnOnce()> Drop for Defer<T> {
    fn drop(&mut self) {
        // This is safe, as there is no way to have a `Defer` struct containing a `None` value
        unsafe { (self.0.take().unwrap_unchecked())() }
    }
}

/// A utility struct for explicitly scoped deferred execution of closures.
///
/// The `DeferGroup` allows you to add closures (functions) that will be executed
/// when the `DeferGroup` instance goes out of scope. It is particularly useful
/// for resource cleanup or deferred actions.
///
/// **Note: `DeferGroup` MUST be bound to a variable to function properly; otherwise, it will be dropped immediately, executing the enclosed closures!**
///
/// # Example
///
/// ```rust
/// use defer_rs::DeferGroup;
///
/// let mut defer_group = DeferGroup::new();
///
/// // Add a function to be executed when `defer_group` goes out of scope
/// defer_group.add(Box::new(|| {
///     println!("Deferred action: Cleaning up resources...");
/// }));
///
/// // Some other code...
///
/// // The deferred (queued) actions will be executed here, when the `defer_group` is dropped.
/// ```
///
/// See also: [`defer_scope!`], [`defer_scope_init!`], [`Defer`], and [`defer!`].
#[must_use = "DeferGroup MUST be bound to a variable to function properly; otherwise, it will be dropped immediately, executing the enclosed closure!"]
pub struct DeferGroup<'a>(Vec<Option<Box<dyn FnOnce() + 'a>>>);

impl<'a> DeferGroup<'a> {
    /// Creates a new `DeferGroup`.
    ///
    /// **Note: `DeferGroup` MUST be bound to a variable to function properly; otherwise, it will be dropped immediately, executing the enclosed closures!**
    ///
    /// # Example
    ///
    /// ```
    /// use defer_rs::DeferGroup;
    ///
    /// let mut defer_group = DeferGroup::new();
    /// // Add deferred actions...
    /// ```
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Adds a deferred closure to the start (0-index) of the `DeferGroup` queue.
    ///
    /// The closures queued in `DeferGroup` will be executed first to last
    /// when the the `DeferGroup` instance goes out of scope.
    ///
    /// # Example
    ///
    /// ```
    /// use defer_rs::DeferGroup;
    ///
    /// let mut defer_group = DeferGroup::new();
    /// {
    ///     defer_group.add(Box::new(|| {
    ///         println!("This will be printed 2nd");        
    ///     }));
    ///     defer_group.add(Box::new(|| {
    ///         println!("This will be printed 1st");
    ///     }));
    /// }
    /// ```
    pub fn add(&mut self, f: Box<dyn FnOnce() + 'a>) {
        self.0.insert(0, Some(f));
    }

    /// Pushes a deferred closure to the end of the `DeferGroup` queue.
    ///
    /// The closures queued in `DeferGroup` will be executed first to last
    /// when the the `DeferGroup` instance goes out of scope.
    ///
    /// # Example
    ///
    /// ```
    /// use defer_rs::DeferGroup;
    ///
    /// let mut defer_group = DeferGroup::new();
    /// {
    ///     defer_group.push(Box::new(|| {
    ///         println!("This will be printed 1st");
    ///     }));
    ///     defer_group.push(Box::new(|| {
    ///         println!("This will be printed 2nd");        
    ///     }));
    /// }    
    /// ```
    pub fn push(&mut self, f: Box<dyn FnOnce() + 'a>) {
        self.0.push(Some(f));
    }
}

impl<'a> Drop for DeferGroup<'a> {
    fn drop(self: &mut DeferGroup<'a>) {
        for deferred in &mut self.0 {
            unsafe { deferred.take().unwrap_unchecked()() };
        }
    }
}

/// A macro for deferring execution of code until the current scope exits.
///
/// The `defer!` macro allows you to specify code that should be executed when the current
/// scope (such as a function or block) is exited, regardless of whether the exit occurs normally
/// or due to an early return, panic, or other unwinding.
///
/// # Examples
///
/// ## Basic usage:
///
/// ```rust
/// use defer_rs::defer;
/// defer! {
///     println!("This will be executed when the current scope exits.");
/// }
/// ```
/// ### Expands to:
/// ```rust
/// let ___deferred_code = ::defer_rs::Defer::new( || {
///     println!("This will be executed when the current scope exits.");
/// });
/// ```
///
/// ## Multiple statements:
/// Multiple statements also work (enclosed in a block or not).
///
/// ```rust
/// use defer_rs::defer;
/// defer! {
///     println!("1st statement.");
///     println!("2nd statement.");
/// }
/// ```
/// ### Expands to:
///
/// ```rust
/// let ___deferred_code = ::defer_rs::Defer::new( || {
///     println!("1st statement.");
///     println!("2nd statement.");
/// });
/// ```
///
/// ## Move capturred values:
/// Sometimes it's necessary to `move` the values capturred from the environment into the generated closure,
/// in this case `move` can be added before the deferred statements and it will be added to the closure.
///
/// ```rust
/// use defer_rs::defer;
/// let val = "Variable that must be passed by value!";
/// defer!( move {
///     println!("`val` is captured by value!");
///     println!("{}", val);
/// });
/// ```
/// ### Expands to:
///
/// ```rust
/// let val = "Variable that must be passed by value!";
/// let ___deferred_code = ::defer_rs::Defer::new( move || {
///     println!("`val` is captured by value!");
///     println!("{}", val);
/// });
/// ```
///
/// ## Immmediately evaluating passed arguments:
/// A common pattern in languages with a `defer` keyword (i.e., `Go`),
/// is to defer a single function/method call,
/// in that case, arguments passed to the call are usually evaluated immediately,
/// this can be mimicked using the `defer!` macro on a single call expression,
/// This behavior can easily be disabled by postfixing the  call expression.
///
/// Note: `move` cannot be used when the macro is used this way,
/// as it's implied.
///
/// ```rust
/// use defer_rs::defer;
/// use std::cell::Cell;
///
/// fn print(to_print: String) {
///     println!("{to_print}");
/// }
///
/// let x = Cell::new(0);   
/// // This will evaluate the arguments passed to the invoked at function at time of macro invocation (now), results in `0`
/// defer!(print(format!("Var x now is: {}", x.get())));
/// // This will evaluate the arguments passed to the invoked at function at call time (deferred-to time, later), results in `3`
/// defer!(print(format!("Var x later is: {}", x.get())););
/// x.set(3);
/// ```
/// ### Expands to:
///
/// ```rust
/// use std::cell::Cell;
///
/// fn print(to_print: String) {
///     println!("{to_print}");
/// }
///
/// let x = Cell::new(0);
/// let ___deferred_code_captured_args = (format!("Var x now is: {}", x.get()), );
/// let ___deferred_code = ::defer_rs::Defer::new( move || {
///                 print(___deferred_code_captured_args.0);
/// });
/// let ___deferred_code = ::defer_rs::Defer::new(|| {
///     print(format!("Var x later is: {}", x.get()))
/// });
/// x.set(3);
/// ```
///
/// See also: [`Defer`], [`DeferGroup`], and [`defer_scope!`].
#[macro_export]
macro_rules! defer{
    // This pattern doesn't match the code directly (unless the input is a block statement), but takes the results from the last two patterns!
    ($(@$move_kw:ident@)? $body:block$(;)?) => {
        let ___deferred_code =$crate::Defer::new($($move_kw)?||
            $body
        );
    };

    // This either matches immediately or doesn't at all!
    ($func:ident($($arg:expr),* $(,)? )) => {
        let ___deferred_code_captured_args = ( $( $arg, )* );
        let ___deferred_code =$crate::Defer::new(move|| {
            ::defer_rs_impl::call_indexed!($func($($arg),*));
        });
    };

    // The following two patterns are only here to surround the input in a block statement and to filter the `move` keyword
    // and pass it back (if it exists) recursively to the the first case to handle the actual code generation
    (move $($body:tt)+ ) => {
        defer!(@move@ {$($body)*})
    };

    ($($body:tt)+ ) => {
        defer!({$($body)*})
    };
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
/// `defer_scope!` is otherwise identical to [`defer!`].
///
/// For more usage examples, refer to the documentation for the [`defer!`] macro,
/// simply replace `defer!` with `defer_scope!` and add an invocation of [`defer_scope_init!`] beforehand.
///
/// See also: [`DeferGroup`], [`defer_scope_init!`], and [`defer!`].
#[cfg(doc)]
#[macro_export]
macro_rules! defer_scope { ($($tt:tt)*) => { ... } }

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
/// See also: [`DeferGroup`], [`defer_scope!`], and [`defer!`].
#[cfg(doc)]
#[macro_export]
macro_rules! defer_scope_init { () => { ... } }

#[cfg(test)]
#[allow(unused)]
mod tests {
    // use super::*;
    use super::{defer, defer_scope, defer_scope_init, Defer, DeferGroup};
    use std::cell::{Cell, RefCell};

    use std::io::Write;

    fn print(to_print: String) {
        println!("{to_print}");
    }

    fn add_to_buffer(to_add: String, buff: &RefCell<Vec<u8>>) {
        writeln!(buff.borrow_mut(), "{to_add}");
    }

    #[test]
    fn test_execution_order() {
        let buff = RefCell::new(Vec::new());
        let val = Cell::new(0);

        {
            defer_scope_init!();
            {
                // This to ensure that the deferred statments are executed in the correct order
                defer_scope!({
                    let res = b"This will be printed 1st, x is: 1\nThis will be printed 2nd\nThis will be printed 3rd\nThis will be printed 4th\nThis will be printed 5th\nThis will be printed 6th\nThis will be printed 7th\nThis will be printed 8th, x is: 3\nThis will be printed 9th\nThis will be printed 10th, x is: 0\nThis will be printed 11th\nThis will be printed 12th\nThis will be printed 13th/last\n";
                    assert_eq!(*buff.borrow(), res.to_vec());
                });
                // The macro deferrs execution of the code passed to it to the end of the scope of the nearest DeferGroup going up
                defer_scope!(
                    writeln!(buff.borrow_mut(), "This will be printed 13th/last");
                );

                defer_scope!(
                writeln!(buff.borrow_mut(), "This will be printed 12th");
                );

                {
                    let val = Cell::new(1);
                    // Define a new DeferGroup
                    defer_scope_init!();

                    // This will evaluate the arguments passed to the invoked at function at time of macro invocation (now), results in `0`
                    // Using `defer!` here instead of `defer_scope!` will result in identical behavior!
                    defer_scope!(add_to_buffer(
                        format!("This will be printed 1st, x is: {}", val.get()),
                        &buff
                    ));
                    val.set(3);
                }

                // `defer!` will delay the execution of code passed to it until the end of it's containg scope!
                defer! {
                    writeln!(buff.borrow_mut(), "This will be printed 3rd");
                    writeln!(buff.borrow_mut(), "This will be printed 4th");
                };

                defer_scope! {
                    writeln!(buff.borrow_mut(), "This will be printed 11th");
                };

                defer! {
                    writeln!(buff.borrow_mut(), "This will be printed 2nd");
                };
            }
            writeln!(buff.borrow_mut(), "This will be printed 5th");

            // This will evaluate the arguments passed to the invoked at function at time of macro invocation (now), results in `0`
            defer_scope!(add_to_buffer(
                format!("This will be printed 10th, x is: {}", val.get()),
                &buff
            ));

            // This will evaluate the arguments passed to the invoked at function at call time (deferred-to time, later), results in `3`
            defer!({
                add_to_buffer(
                    format!("This will be printed 8th, x is: {}", val.get()),
                    &buff,
                )
            });
            val.set(3);

            defer_scope! {
                writeln!(buff.borrow_mut(), "This will be printed 9th");
            };

            writeln!(buff.borrow_mut(), "This will be printed 6th");

            defer! {
                writeln!(buff.borrow_mut(), "This will be printed 7th");
            };
        }
    }

    #[test]
    fn test_defer_macro_execution() {
        let val = Cell::new(0);
        {
            defer!(val.set(1));
            assert_eq!(val.get(), 0);
        }
        assert_eq!(val.get(), 1);
    }

    #[test]
    fn test_defer_struct() {
        let val = Cell::new(0);
        {
            let _deferred = Defer::new(|| val.set(1));
            assert_eq!(val.get(), 0);
        }
        assert_eq!(val.get(), 1);
    }

    #[test]
    fn test_defer_scoped_macro_execution() {
        let val = Cell::new(0);
        {
            defer_scope_init!();
            {
                defer_scope!(val.set(1));
                assert_eq!(val.get(), 0);
            }
            assert_eq!(val.get(), 0);
        }
        assert_eq!(val.get(), 1)
    }

    #[test]
    fn test_defer_group() {
        let val = Cell::new(0);
        {
            let mut deferred = DeferGroup::new();
            {
                deferred.add(Box::new(|| val.set(1)));
                assert_eq!(val.get(), 0);
            }
            assert_eq!(val.get(), 0);
        }
        assert_eq!(val.get(), 1)
    }

    #[test]
    fn test_defer_macro_immediate_args_eval() {
        let buff = RefCell::new(Vec::new());
        let buff2 = RefCell::new(Vec::new());
        let val = Cell::new(0);

        defer! {
            let res = b"x is: 3\n";
            assert_eq!(*buff.borrow(), res.to_vec());

            let res = b"x is: 0\n";
            assert_eq!(*buff2.borrow(), res.to_vec());
        };

        // This will evaluate the arguments passed to the invoked at function at call time (deferred-to time, later), results in `3`
        defer!(
            add_to_buffer(
                format!("x is: {}", val.get()),
                &buff
            );
        );

        // This will evaluate the arguments passed to the invoked at function at time of macro invocation (now), results in `0`
        defer!(add_to_buffer(format!("x is: {}", val.get()), &buff2));
        val.set(3);
    }

    #[test]
    fn test_defer_scope_macro_immediate_args_eval() {
        let buff = RefCell::new(Vec::new());
        let buff2 = RefCell::new(Vec::new());
        let val = Cell::new(0);
        defer_scope_init!();
        defer_scope! {
            let res = b"x is: 3\n";
            assert_eq!(*buff.borrow(), res.to_vec());

            let res = b"x is: 0\n";
            assert_eq!(*buff2.borrow(), res.to_vec());
        };

        // This will evaluate the arguments passed to the invoked at function at call time (deferred-to time, later), results in `3`
        defer_scope!(
            add_to_buffer(
                format!("x is: {}", val.get()),
                &buff
            );
        );

        // This will evaluate the arguments passed to the invoked at function at time of macro invocation (now), results in `0`
        defer_scope!(add_to_buffer(format!("x is: {}", val.get()), &buff2));
        val.set(3);
    }
}
    