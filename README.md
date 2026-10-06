# prompt-rust

Python's `input()` for Rust. Print a prompt, read a line from stdin, and parse it into any type that implements `FromStr`. No dependencies.

```toml
[dependencies]
prompt-rust = "0.2"
```

## Usage

```rust,no_run
use prompt_rust::input;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let name: String = input!("Name: ")?;
    let age: u8 = input!("Hi {name}, how old are you? ")?;
    println!("{name} is {age}.");
    Ok(())
}
```

The type you assign to decides what the line is parsed into: `String`, any integer or float, `bool`, `char`, `IpAddr`, or your own `FromStr` type. The prompt works like `print!`, inline `{name}` captures included, and a plain variable works too: `input!(question)`. With no arguments, `input!()` reads without a prompt.

Stdout is flushed before every read, so the prompt always shows. The trailing `\n` or `\r\n` is removed. Numbers tolerate surrounding spaces (`" 42 "` reads as `42`), while strings keep them.

## API

| Item | Returns | At end of input |
| --- | --- | --- |
| `input!(...)` | `Result<T, InputError<T::Err>>` | `Err(InputError::Eof)` |
| `inputln!(...)` | Same as `input!`, with the prompt on its own line | `Err(InputError::Eof)` |
| `try_input!(...)` | `Result<Option<T>, InputError<T::Err>>` | `Ok(None)` |
| `read_input()` | `input!()` as a function, for use after your own `print!` | `Err(InputError::Eof)` |
| `read_input_from(reader, writer, prompt)` | Same, with any `BufRead` and `Write` | `Err(InputError::Eof)` |

## Reading until end of input

```rust,no_run
use prompt_rust::try_input;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut numbers: Vec<i64> = Vec::new();
    while let Some(n) = try_input!()? {
        numbers.push(n);
    }
    println!("sum = {}", numbers.iter().sum::<i64>());
    Ok(())
}
```

## Retrying bad input

```rust,no_run
use prompt_rust::{input, InputError};

fn main() {
    let age: u8 = loop {
        match input!("Age: ") {
            Ok(age) => break age,
            Err(InputError::Parse(err)) => println!("That's not an age ({err}). Try again."),
            Err(err) => {
                eprintln!("{err}");
                return;
            }
        }
    };
    println!("You are {age}.");
}
```

## Testing code that reads input

`read_input_from` takes any reader and writer, so prompts and parsing can be tested without a terminal:

```rust
use prompt_rust::read_input_from;
use std::io::Cursor;

let mut input = Cursor::new("42\n");
let mut output = Vec::new();
let n: i32 = read_input_from(&mut input, &mut output, Some(format_args!("Number: "))).unwrap();
assert_eq!(n, 42);
assert_eq!(output, b"Number: ");
```

## Errors

`InputError<E>` has three variants. `Io` means reading stdin or writing the prompt failed, and the underlying `io::Error` is available through `source()`. `Parse(E)` means the line didn't parse into the requested type, and the parse error is part of the message. `Eof` means input ended before a line was read.

`InputError` implements `std::error::Error`, so `?` works with `Box<dyn Error>` and anyhow.

## Upgrading from 0.1

| 0.1 | 0.2 |
| --- | --- |
| `input!` returning `Result<Option<T>, _>` | `try_input!` |
| `input_no_eof!` | `input!` |
| `read_input_with_prompt(format_args!(...))` | `input!(...)` |
| `read_input_from(reader, prompt)` | `read_input_from(reader, writer, prompt)` |
| `use input_macro::...` | `use prompt_rust::...` |

## Background

The macros follow the design proposed in [RFC 3799](https://github.com/rust-lang/rfcs/pull/3799) for adding input macros to the standard library. This crate lets you use that API on stable Rust today.

Earlier discussions, oldest first:

- [rust-lang/rust#75435](https://github.com/rust-lang/rust/pull/75435) (2020): the first `std::io::input` PR, closed so the design could be worked out in an RFC.
- [RFC 3183](https://github.com/rust-lang/rfcs/pull/3183) (2021): Console Input Simplified, a draft that also proposed scan macros.
- [RFC 3196](https://github.com/rust-lang/rfcs/pull/3196) (2021): `std::io::inputln()`, the thread where this crate's API took shape.
- [libs-team#207](https://github.com/rust-lang/libs-team/issues/207) (2023): a `Stdin::parse_line` proposal that parses straight from stdin's buffer.
- [rust-lang/rust#117852](https://github.com/rust-lang/rust/issues/117852) (2023): the std feature request this crate grew out of.
- [libs-team#460](https://github.com/rust-lang/libs-team/issues/460) (2024): a `Stdin::next_line` proposal, closed in 2025 after the libs-api team said the proposed `inputln!` macro would likely cover it.
## License

MIT