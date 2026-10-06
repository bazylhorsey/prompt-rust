use std::error::Error;
use std::fmt;
use std::io::{self, BufRead, Write};
use std::str::FromStr;

/// Why reading input failed.
#[derive(Debug)]
pub enum InputError<E> {
    /// Reading stdin or writing the prompt failed.
    Io(io::Error),
    /// The line could not be parsed into the requested type.
    Parse(E),
    /// Input ended before a line was read (where Python raises `EOFError`).
    Eof,
}

impl<E: fmt::Display> fmt::Display for InputError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // The io::Error is exposed through `source()`, so it isn't repeated here.
            InputError::Io(_) => f.write_str("I/O error while getting input"),
            InputError::Parse(err) => write!(f, "failed to parse input: {err}"),
            InputError::Eof => f.write_str("unexpected end of input"),
        }
    }
}

impl<E: fmt::Display + fmt::Debug> Error for InputError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InputError::Io(err) => Some(err),
            InputError::Parse(_) | InputError::Eof => None,
        }
    }
}

/// Writes `prompt` (if any) to `writer`, then reads one line from `reader` and parses it.
///
/// The writer is always flushed before reading. The trailing `\n` or `\r\n` is removed.
/// If the line doesn't parse as-is, it is trimmed and parsed once more, so `" 42 "`
/// reads as `42` while a `String` keeps its spaces.
pub fn read_input_from<R, W, T>(
    reader: &mut R,
    writer: &mut W,
    prompt: Option<fmt::Arguments<'_>>,
) -> Result<T, InputError<T::Err>>
where
    R: BufRead + ?Sized,
    W: Write + ?Sized,
    T: FromStr,
{
    write_prompt(writer, prompt)?;
    read_line_parsed(reader)
}

/// Reads one line from stdin and parses it, without a prompt.
///
/// Stdout is flushed first, so text from `print!` shows up before the read.
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// print!("Guess a number: ");
/// let guess: u32 = input_macro::read_input()?;
/// # Ok(())
/// # }
/// ```
pub fn read_input<T: FromStr>() -> Result<T, InputError<T::Err>> {
    __read_stdin(None)
}

#[doc(hidden)]
pub fn __read_stdin<T: FromStr>(
    prompt: Option<fmt::Arguments<'_>>,
) -> Result<T, InputError<T::Err>> {
    // Write the prompt before locking stdin, so we never hold both locks at once.
    write_prompt(&mut io::stdout(), prompt)?;
    read_line_parsed(&mut io::stdin().lock())
}

fn write_prompt<W: Write + ?Sized, E>(
    writer: &mut W,
    prompt: Option<fmt::Arguments<'_>>,
) -> Result<(), InputError<E>> {
    if let Some(prompt) = prompt {
        writer.write_fmt(prompt).map_err(InputError::Io)?;
    }
    writer.flush().map_err(InputError::Io)
}

fn read_line_parsed<R: BufRead + ?Sized, T: FromStr>(
    reader: &mut R,
) -> Result<T, InputError<T::Err>> {
    let mut line = String::new();
    if reader.read_line(&mut line).map_err(InputError::Io)? == 0 {
        return Err(InputError::Eof);
    }
    let line = line.strip_suffix('\n').unwrap_or(&line);
    let line = line.strip_suffix('\r').unwrap_or(line);
    line.parse()
        .or_else(|err| match line.trim() {
            trimmed if trimmed.len() < line.len() => trimmed.parse(),
            _ => Err(err),
        })
        .map_err(InputError::Parse)
}

/// Reads one line from stdin and parses it into the type you ask for,
/// printing an optional prompt first.
///
/// End of input is an error (`InputError::Eof`), like Python's `EOFError`.
/// The prompt can be a format string (inline `{name}` captures work) or any
/// value that implements `Display`.
///
/// ```no_run
/// use input_macro::input;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let name: String = input!("Name: ")?;
/// let age: u8 = input!("Hi {name}, how old are you? ")?;
/// let question = format!("{name}, pick a number: ");
/// let pick: i64 = input!(question)?;
/// let line: String = input!()?; // no prompt
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! input {
    () => {
        $crate::read_input()
    };
    ($($prompt:tt)+) => {
        $crate::__read_stdin(::core::option::Option::Some($crate::__prompt!($($prompt)+)))
    };
}

/// Like [`input!`], but prints the prompt on its own line.
///
/// ```no_run
/// use input_macro::inputln;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let color: String = inputln!("What's your favorite color?")?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! inputln {
    () => {
        $crate::read_input()
    };
    ($($prompt:tt)+) => {
        $crate::__read_stdin(::core::option::Option::Some(::core::format_args!(
            "{}\n",
            $crate::__prompt!($($prompt)+)
        )))
    };
}

/// Like [`input!`], but returns `Ok(None)` at end of input instead of an error.
///
/// ```no_run
/// use input_macro::try_input;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut numbers: Vec<i64> = Vec::new();
/// while let Some(n) = try_input!()? {
///     numbers.push(n);
/// }
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! try_input {
    ($($prompt:tt)*) => {
        match $crate::input!($($prompt)*) {
            ::core::result::Result::Ok(value) => {
                ::core::result::Result::Ok(::core::option::Option::Some(value))
            }
            ::core::result::Result::Err($crate::InputError::Eof) => {
                ::core::result::Result::Ok(::core::option::Option::None)
            }
            ::core::result::Result::Err(err) => ::core::result::Result::Err(err),
        }
    };
}

/// Turns macro arguments into a prompt: a format string with arguments, or any `Display` value.
#[doc(hidden)]
#[macro_export]
macro_rules! __prompt {
    ($fmt:literal $($args:tt)*) => {
        ::core::format_args!($fmt $($args)*)
    };
    ($prompt:expr $(,)?) => {
        ::core::format_args!("{}", $prompt)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Error as IoError, ErrorKind};

    /// Reads one line with no prompt, discarding prompt output.
    fn read<T: FromStr>(reader: &mut impl BufRead) -> Result<T, InputError<T::Err>> {
        read_input_from(reader, &mut io::sink(), None)
    }

    /// Basic test reading an integer
    #[test]
    fn test_read_input_integer() {
        let mut reader = Cursor::new("42\n");
        let res: Result<i32, _> = read(&mut reader);
        assert_eq!(res.unwrap(), 42);
    }

    /// Test reading a floating-point number
    #[test]
    fn test_read_input_float() {
        let mut reader = Cursor::new("1.25\n");
        let res: Result<f64, _> = read(&mut reader);
        assert!((res.unwrap() - 1.25).abs() < f64::EPSILON);
    }

    /// Test reading an unsigned integer
    #[test]
    fn test_read_input_unsigned() {
        let mut reader = Cursor::new("255\n");
        let res: Result<u32, _> = read(&mut reader);
        assert_eq!(res.unwrap(), 255);
    }

    /// EOF immediately (0 bytes read)
    #[test]
    fn test_read_input_eof() {
        let mut reader = Cursor::new("");
        let res: Result<i32, _> = read(&mut reader);
        assert!(matches!(res, Err(InputError::Eof)));
    }

    /// Parse error: not an integer
    #[test]
    fn test_read_input_parse_error() {
        let mut reader = Cursor::new("not an int\n");
        let res: Result<i32, _> = read(&mut reader);
        assert!(matches!(res, Err(InputError::Parse(_))));
    }

    /// Reading a standard string
    #[test]
    fn test_read_input_string() {
        let mut reader = Cursor::new("hello world\r\n");
        let res: Result<String, _> = read(&mut reader);
        assert_eq!(res.unwrap(), "hello world");
    }

    /// The prompt goes to the writer, not straight to stdout
    #[test]
    fn test_with_prompt() {
        let mut reader = Cursor::new("100\n");
        let mut out = Vec::new();
        let res: Result<i32, _> =
            read_input_from(&mut reader, &mut out, Some(format_args!("Enter: ")));
        assert_eq!(res.unwrap(), 100);
        assert_eq!(out, b"Enter: ");
    }

    /// Multiple lines: read first line (valid), then second line (valid)
    #[test]
    fn test_multiple_lines_valid() {
        let mut reader = Cursor::new("123\n456\n");
        // Read first line
        let first: i32 = read(&mut reader).unwrap();
        assert_eq!(first, 123);

        // Read second line
        let second: i32 = read(&mut reader).unwrap();
        assert_eq!(second, 456);
    }

    /// Multiple lines: read first line (valid), second line (invalid parse), third line (EOF)
    #[test]
    fn test_multiple_lines_parse_error_then_eof() {
        let mut reader = Cursor::new("42\nnotanint\n");
        // Read first line
        let first: i32 = read(&mut reader).unwrap();
        assert_eq!(first, 42);

        // Read second line: parse error
        let second = read::<i32>(&mut reader);
        assert!(matches!(second, Err(InputError::Parse(_))));

        // Next read is EOF (because we've consumed all input)
        let third = read::<i32>(&mut reader);
        assert!(matches!(third, Err(InputError::Eof)));
    }

    /// Test what happens with an empty line (just "\n")
    /// - By default, we interpret empty line as an empty string -> parse error for integer
    #[test]
    fn test_empty_line_behavior() {
        let mut reader = Cursor::new("\n");
        let res: Result<i32, _> = read(&mut reader);
        // Typically this is a parse error, because "" can't parse into i32
        assert!(matches!(res, Err(InputError::Parse(_))));
    }

    /// Prompts accept format strings, inline captures, and plain `Display` values
    #[test]
    fn test_prompt_forms() {
        let name = "Ada";
        let question = String::from("Ready? ");
        assert_eq!(crate::__prompt!("Hi {name}: ").to_string(), "Hi Ada: ");
        assert_eq!(crate::__prompt!("{} + {} = ", 1, 2).to_string(), "1 + 2 = ");
        assert_eq!(crate::__prompt!(question).to_string(), "Ready? ");
        assert_eq!(crate::__prompt!(&question).to_string(), "Ready? ");
    }

    /// Numbers forgive surrounding spaces; strings keep them
    #[test]
    fn test_whitespace_around_numbers() {
        let mut reader = Cursor::new(" 42 \r\n  keep  \n");
        assert_eq!(read::<i32>(&mut reader).unwrap(), 42);
        assert_eq!(read::<String>(&mut reader).unwrap(), "  keep  ");
    }

    /// A failing prompt write is returned as `InputError::Io` instead of panicking
    #[test]
    fn test_prompt_write_error() {
        struct BrokenPipe;
        impl Write for BrokenPipe {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(IoError::new(ErrorKind::BrokenPipe, "closed"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let mut reader = Cursor::new("1\n");
        let res: Result<i32, _> =
            read_input_from(&mut reader, &mut BrokenPipe, Some(format_args!("x")));
        assert!(matches!(res, Err(InputError::Io(_))));
    }

    /// Each message appears once in an error chain (Display or source, never both)
    #[test]
    fn test_error_display_and_source() {
        let parse = read::<i32>(&mut Cursor::new("x\n")).unwrap_err();
        assert_eq!(
            parse.to_string(),
            "failed to parse input: invalid digit found in string"
        );
        assert!(parse.source().is_none());

        let io = InputError::<std::num::ParseIntError>::Io(IoError::other("boom"));
        assert_eq!(io.to_string(), "I/O error while getting input");
        assert_eq!(io.source().unwrap().to_string(), "boom");
    }

    /// Check that a custom `Read` implementation that returns an error triggers `InputError::Io`.
    #[test]
    fn test_io_error() {
        struct ErrorReader;

        impl BufRead for ErrorReader {
            fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
                // Force an I/O error
                Err(IoError::other("Simulated I/O failure"))
            }
            fn consume(&mut self, _amt: usize) {}
        }

        // We only need `read_line` to fail:
        impl std::io::Read for ErrorReader {
            fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
                Err(IoError::other("Simulated I/O failure"))
            }
        }

        let mut reader = ErrorReader;
        let res: Result<String, _> = read(&mut reader);
        assert!(matches!(res, Err(InputError::Io(_))));
    }
}
