Defer macros and utils for Rust
===============================

Deferred execution Rust utilities.

This crate provides utility structs and macros for deferred execution of code in Rust. It allows you to specify actions that should be executed when a scope is exited, such as when a function ends or a block of code completes execution.

## Features

- **Defer Statements**: The library introduces the concept of `Defer`ed statements, providing similar functionality to those found in other programming languages. The `Defer` struct allows you to register functions or closures to be executed when the active scope is exited.

- **DeferGroup**: The `DeferGroup` struct allows you to group multiple deferred actions together. When the `DeferGroup` goes out of scope, all registered actions are executed in reverse order.

- **Macro Support**: The `defer!` and `defer_scope!` macros simplify the process of creating deferred actions. They can be used to defer function calls, closures, or arbitrary code blocks.

## Installation

To use the `defer` library, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
defer-rs = "0.1"
```


## Usage

### 1. Using `defer!` Macro

The `defer!` macro simplifies the creation of deferred actions. It supports both function calls and arbitrary code blocks.

Example:

```rust
use defer_rs::defer;

defer!({
    println!("Deferred action using macro");
});

// ... other code ...

// The deferred action will be executed when it goes out of scope
```

### 2. Using `defer_scope!` Macro

The `defer_scope!` macro can be used in conjunction with the `defer_scope_init!` macro to precisely manage the scope in which a collection of deferred statements will execute at the end of.

Example:

```rust
use defer_rs::{defer_scope, defer_scope_init};
use std::cell::Cell;

let val = Cell::new(0);
{
    defer_scope_init!(); // Initiate scoped deferred collection
    {
        defer_scope!(val.set(1)); // Add to it in a nested scope
        assert_eq!(val.get(), 0);
    }
    // `defer_scope!(val.set(1))` is still to run, even though the scope containing it has ended!
    assert_eq!(val.get(), 0);
    // ... other code ...

    // The deferred collection will execute the contained closures when it goes out of scope on the following line
}
assert_eq!(val.get(), 1)

```

### 3. Using `Defer` Struct

The `Defer` struct is used to create individual deferred actions. You can create a new deferred action by providing a closure or function that takes no arguments. When the `Defer` instance goes out of scope, the provided closure or function will be executed.

Note: The `defer!` macro is syntactic sugar for this struct.

Example:

```rust
use defer_rs::Defer;

let deferred_action = Defer::new(|| {
    println!("Deferred action executed!");
});

// ... other code ...

// deferred_action will be executed when it goes out of scope
```

### 4. Grouping Deferred Actions

The `DeferGroup` struct allows you to group multiple deferred actions together. Actions are executed in reverse order when the `DeferGroup` goes out of scope.

Note: The `defer_scope!` and `defer_scope_init!` macros are syntactic sugar for this struct and it's `::add()` method.

Example:

```rust
use defer_rs::DeferGroup;

let mut defer_group = DeferGroup::new();

defer_group.add(Box::new(|| {
    println!("First deferred action");
}));

defer_group.push(Box::new(|| {
    println!("Second deferred action");
}));

// ... other code ...

// Both deferred actions will be executed in reverse order
```

### 5. Macros Special Case: Capturing closure’s environment by value

Prefixing the statement(s) in a 'defer!' or 'defer_scope!' macro with the `move` keyword will capture the closure’s environment by value.

Note: This is the same as adding `move` to the closure passed to `Defer::new()` or `DeferGroup::add()`.

Example:

```rust
use defer_rs::{defer_scope, defer_scope_init};

let val = "Variable that must be passed by value!";
defer_scope_init!(); // Initiate scoped deferred collection
{
    defer_scope!(move {
        println!("`val` is captured by value!");
        println!("{}", val);
    });
}
```

### 6. Macros Special Case: Immediate evaluation of passed arguments

It's sometimes desireable to have the arguments passed to a deferred function call be evaluated at time of deferment, rather than the deferred time of execution (similar to how the `defer` keyword works in `Go` and other programming languages), this behavior is mimicked when the 'defer!' (or 'defer_scope!') macro is used on a solitary function call.

Note: This behavior can be disbled by simply postfixing the function call passed to the macro by a `;`.

Example:

```rust
use defer_rs::{defer_scope, defer_scope_init};
use std::cell::{Cell, RefCell};
use std::io::Write;

fn add_to_buffer(to_add: String, buff: &RefCell<Vec<u8>>) {
    writeln!(buff.borrow_mut(), "{to_add}");
}

let buff = RefCell::new(Vec::new());
let buff2 = RefCell::new(Vec::new());
let val = Cell::new(0);
defer_scope_init!();
defer_scope ! {
    let res = b"x is: 3\n";
    assert_eq!(*buff.borrow(), res.to_vec());

    let res = b"x is: 0\n";
    assert_eq!(*buff2.borrow(), res.to_vec());
};

// This will evaluate the arguments passed to the invoked at function at call time (deferred-to time / later), `val.get()` results in `3`
defer_scope!(
    add_to_buffer(
        format!("x is: {}", val.get()),
        &buff
    ); // Notice the `;` at the end of the function call, disables the immediate evaluation of passed arguments.
);

// This will evaluate the arguments passed to the invoked function at time of macro invocation (now), `val.get()` results in `0`
defer_scope!(
    add_to_buffer(
        format!("x is: {}", val.get()),
        &buff2
    )  // No `;`. Arguments are immediately evaluated!
);
val.set(3);
```

## Lengthy Example

```rust
use defer_rs::{defer, defer_scope, defer_scope_init};
use std::cell::Cell;

fn print(to_print: String) {
    println!("{to_print}");
}

fn main() {
    let x = Cell::new(0);
    defer_scope_init!();

    {
        // The macro deferrs execution of the code passed to it to the end of the scope of the nearest `DeferGroup` (or `defer_scope_init!` invocation) going up
        defer_scope!({
            println!("This will be printed 13th/last");
        });

        defer_scope!(
        println!("This will be printed 12th");
        );

        {
            let x = Cell::new(1);
            // Define a new DeferGroup
            defer_scope_init!();

            // This will evaluate the arguments passed to the invoked at function at time of macro invocation (now), results in `0`
            // Using `defer!` here instead of `defer_scope!` will result in identical behavior!
            defer_scope!(print(format!("This will be printed 1st, x is: {}", x.get())));
            x.set(3);
        }

        // `defer!` will delay the execution of code passed to it until the end of it's containg scope!
        defer! { move
            println!("This will be printed 3rd");
            println!("This will be printed 4th");
        };

        defer_scope! {
            println!("This will be printed 11th");
        };

        defer! { move
            println!("This will be printed 2nd");
        };
    }
    println!("This will be printed 5th");

    // This will evaluate the arguments passed to the invoked at function at time of macro invocation (now), `x.get()` results in `0`
    defer_scope!(print(format!("This will be printed 10th, x is: {}", x.get())));

    // This will evaluate the arguments passed to the invoked at function at call time (deferred-to time, later), `x.get()` results in `3`
    defer!({ print(format!("This will be printed 8th, x is: {}", x.get())) });
    x.set(3);

    defer_scope! {
        println!("This will be printed 9th");
    };

    println!("This will be printed 6th");

    defer! {
        println!("This will be printed 7th");
    };
}
```
This will result in the following lines being printed to `stdout`:
```text  
This will be printed 1st, x is: 1
This will be printed 2nd
This will be printed 3rd    
This will be printed 4th
This will be printed 5th
This will be printed 6th
This will be printed 7th
This will be printed 8th, x is: 3
This will be printed 9th
This will be printed 10th, x is: 0
This will be printed 11th
This will be printed 12th
This will be printed 13th/last
```

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>
<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
