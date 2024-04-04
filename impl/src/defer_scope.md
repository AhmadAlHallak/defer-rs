A macro for deferring execution of code until the closest scope containg a previousely invoked [`defer_scope_init!`] macro ends.
///
`defer_scope!` is to be used when you want to defer not to the end of the current active scope,
but to the end of a larger parent scope, that scope is specified by invoking `defer_scope_init!`, 
[`defer_scope_init!`] macro **must** be invoked before `defer_scope!` is used, it must also share a scope with it!
`defer_scope!` macro can be invoked unlimited times for a given `defer_scope_init!` invocation
`defer_scope!` is otherwise identical to [`defer!`].
///
# Examples

## Basic usage:
///
```rust
use defer_rs::defer_scope;
defer_scope! {
    println!("This will be executed when the current scope exits.");
}
```
### Expands to:
```rust
let ___deferred_code = ::defer_rs::Defer::new( || { println!("This will be executed when the current scope exits."); });
```
///
## Multiple statements:
Multiple statements also work (enclosed in a block or not).
///
```rust
use defer_rs::defer;
defer! {
    println!("1st statement.");
    println!("2nd statement.");
}
```
### Expands to:

```rust
let ___deferred_code = ::defer_rs::Defer::new( || {
    println!("1st statement.");
    println!("2nd statement.");
});
```
///
## Move capturred values:
Sometimes it's necessary to `move` the values capturred from the environment into the generated closure,
in this case `move` can be added before the deferred statements and it will be added to the closure.
///
```rust
use defer_rs::defer;
let val = "Variable that must be passed by value!";
defer!( move {
    println!("`val` is captured by value!");
    
    println!("{}", val);
});
```
### Expands to:

```rust
let val = "Variable that must be passed by value!";
let ___deferred_code = ::defer_rs::Defer::new( move || {
    println!("`val` is captured by value!");
    println!("{}", val);
});
```
///
## Immmediately evaluating passed arguments:
A common pattern in languages with a `defer` keyword (i.e., `Go`), 
is to defer a single function/method call,
in that case, arguments passed to the call are usually evaluated immediately,
this can be mimicked using the `defer!` macro on a single call expression,
This behavior can easily be disabled by postfixing the  call expression.
Note: `move` cannot be used when the macro is used this way, 
as it's implied.
///
```rust
use defer_rs::defer;
use std::cell::Cell;

fn print(to_print: String) {
    println!("{to_print}"); 
}

let x = Cell::new(0);   
// This will evaluate the arguments passed to the invoked at function at time of macro invocation (now), results in `0`
defer!(print(format!("Var x now is: {}", x.get())));
// This will evaluate the arguments passed to the invoked at function at call time (deferred-to time, later), results in `3`
defer!(print(format!("Var x later is: {}", x.get())););
x.set(3);
```
### Expands to:

```rust
use std::cell::Cell;

fn print(to_print: String) {
    println!("{to_print}"); 
}

let x = Cell::new(0);
let ___deferred_code_captured_args = (format!("Var x now is: {}", x.get()), );
let ___deferred_code = ::defer_rs::Defer::new( move || {
                print(___deferred_code_captured_args.0);
});
let ___deferred_code = ::defer_rs::Defer::new(|| {
    print(format!("Var x later is: {}", x.get()))
});
x.set(3);
```