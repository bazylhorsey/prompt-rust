use std::io::{self, BufRead, Write};
use std::str::FromStr;

/// A custom error type for input operations that can represent I/O or parse errors.
///
/// This type provides a convenient way to distinguish whether a failure
/// occurred due to an I/O problem (like a closed or unreadable stdin),
/// or because the input could not be parsed into the desired type.
#[derive(Debug)]
pub enum InputError<E: std::fmt::Debug> {
    /// An error related to reading from standard input (I/O error).
    Io(io::Error),
    /// An error related to parsing the input string into the desired type.
    Parse(E),
}

impl<E: std::fmt::Display + std::fmt::Debug> std::fmt::Display for InputError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::Io(e) => write!(f, "I/O error: {}", e),
            InputError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for InputError<E> {}

/// Reads a line from a generic buffered reader, optionally prints a prompt,
/// then trims and parses the input into the desired type.
///
/// This function is generic over any type implementing `BufRead` and allows
/// injecting custom input sources (e.g., for testing) instead of always using
/// stdin. If a `prompt` is provided, it is printed before reading.
///
/// # Errors
///
/// Returns `InputError::Io` if reading fails and `InputError::Parse` if parsing the
/// trimmed line into type `T` fails.
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
///
/// let input_data = b"42\n";
/// let mut reader = Cursor::new(input_data);
/// let value: i32 = read_and_parse_from(&mut reader, None).unwrap();
/// assert_eq!(value, 42);
/// ```
fn read_and_parse_from<R, T>(reader: &mut R, prompt: Option<&str>) -> Result<T, InputError<T::Err>>
where
    R: BufRead,
    T: FromStr,
    T::Err: std::fmt::Debug + std::fmt::Display,
{
    if let Some(p) = prompt {
        print!("{}", p);
        io::stdout().flush().map_err(InputError::Io)?;
    }

    let mut input = String::new();
    reader.read_line(&mut input).map_err(InputError::Io)?;
    let trimmed_input = input.trim_end_matches(|c| c == '\r' || c == '\n');

    trimmed_input.parse::<T>().map_err(InputError::Parse)
}

/// Reads a line from a generic buffered reader, optionally prints a prompt, and attempts to
/// parse it into the desired type. Returns `Ok(None)` if EOF is reached.
///
/// This function is similar to `read_and_parse_from` but is designed for cases where
/// encountering EOF (no input available) should return `None` instead of an error.
///
/// # Errors
///
/// Returns `InputError::Io` if there is an I/O error.  
/// Returns `InputError::Parse` if the input cannot be parsed.  
/// Returns `Ok(None)` on EOF, so no error for that scenario.
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
///
/// // Simulate EOF by providing an empty input.
/// let input_data = b"";
/// let mut reader = Cursor::new(input_data);
/// let value: Option<i32> = read_and_parse_with_eof_from(&mut reader, None).unwrap();
/// assert!(value.is_none());
/// ```
fn read_and_parse_with_eof_from<R, T>(
    reader: &mut R,
    prompt: Option<&str>,
) -> Result<Option<T>, InputError<T::Err>>
where
    R: BufRead,
    T: FromStr,
    T::Err: std::fmt::Debug + std::fmt::Display,
{
    if let Some(p) = prompt {
        print!("{}", p);
        io::stdout().flush().map_err(InputError::Io)?;
    }

    let mut input = String::new();
    match reader.read_line(&mut input) {
        Ok(0) => Ok(None), // EOF encountered
        Ok(_) => {
            let trimmed_input = input.trim_end_matches('\r').trim_end_matches('\n');
            trimmed_input
                .parse::<T>()
                .map(Some)
                .map_err(InputError::Parse)
        }
        Err(e) => Err(InputError::Io(e)),
    }
}

/// Reads and parses a single line from stdin into a desired type `T`.
///
/// This function locks stdin, reads one line, trims it, and attempts to parse it into `T`.
/// If an error occurs during reading or parsing, an `InputError` is returned.
///
/// # Errors
///
/// Returns `InputError::Io` on read errors.  
/// Returns `InputError::Parse` if the input doesn't parse into the desired type.
///
/// # Examples
///
/// ```
/// // Not testable directly in a doc test since it reads from stdin, but usage is straightforward:
/// // let value: i32 = read_and_parse().unwrap();
/// ```
pub fn read_and_parse<T: FromStr>() -> Result<T, InputError<T::Err>>
where
    T::Err: std::fmt::Debug + std::fmt::Display,
{
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    read_and_parse_from(&mut reader, None)
}

/// Like `read_and_parse`, but prints a prompt before reading.
///
/// This function is useful for interactive programs. It prints the provided `prompt`,
/// then waits for user input. The input is then parsed into `T` or yields an error.
///
/// # Errors
///
/// Returns `InputError::Io` on read errors.  
/// Returns `InputError::Parse` if the input cannot be parsed into `T`.
///
/// # Examples
///
/// ```no_run
/// let value: i32 = read_and_parse_with_prompt("Enter a number: ").unwrap();
/// println!("You entered: {}", value);
/// ```
pub fn read_and_parse_with_prompt<T: FromStr>(prompt: &str) -> Result<T, InputError<T::Err>>
where
    T::Err: std::fmt::Debug + std::fmt::Display,
{
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    read_and_parse_from(&mut reader, Some(prompt))
}

/// Reads one line from stdin and attempts to parse it into `T`. Returns `Ok(None)` if EOF is reached.
///
/// This function is similar to `read_and_parse`, but handles EOF gracefully by returning `None`
/// instead of an error. Useful for input loops where EOF indicates no more input.
///
/// # Errors
///
/// Returns `InputError::Io` on read errors.  
/// Returns `InputError::Parse` if parsing fails.
///
/// # Examples
///
/// ```no_run
/// // If the user presses Ctrl-D (on Unix) or Ctrl-Z (on Windows) to signal EOF:
/// let value: Option<i32> = read_and_parse_with_eof().unwrap();
/// match value {
///     Some(v) => println!("Got: {}", v),
///     None => println!("No more input"),
/// }
/// ```
pub fn read_and_parse_with_eof<T: FromStr>() -> Result<Option<T>, InputError<T::Err>>
where
    T::Err: std::fmt::Debug + std::fmt::Display,
{
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    read_and_parse_with_eof_from(&mut reader, None)
}

/// Like `read_and_parse_with_eof`, but prints a prompt before reading input.
///
/// This is useful for interactive use cases where you want to prompt the user and then read
/// an optional value, returning `None` if EOF is encountered.
///
/// # Errors
///
/// Returns `InputError::Io` on read errors.  
/// Returns `InputError::Parse` if parsing fails.
///
/// # Examples
///
/// ```no_run
/// let value: Option<i32> = read_and_parse_with_prompt_eof("Enter a number (or EOF): ").unwrap();
/// match value {
///     Some(v) => println!("Got: {}", v),
///     None => println!("Reached EOF or empty input."),
/// }
/// ```
pub fn read_and_parse_with_prompt_eof<T: FromStr>(
    prompt: &str,
) -> Result<Option<T>, InputError<T::Err>>
where
    T::Err: std::fmt::Debug + std::fmt::Display,
{
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    read_and_parse_with_eof_from(&mut reader, Some(prompt))
}

/// A macro for reading and parsing input from stdin, returning `Option<T>`.
///
/// - `input!()` reads a line from stdin and attempts to parse it into a `String`.
///   If EOF is encountered, it returns `Ok(None)`.
///
/// - `input!("prompt")` prints the given prompt before reading and parsing the input.
///   If EOF is encountered, `Ok(None)` is returned.
///
/// This macro uses `read_and_parse_with_eof`, allowing for graceful EOF handling.
///
/// # Examples
///
/// ```no_run
/// // With no prompt
/// let name: Option<String> = input!().unwrap();
///
/// // With a prompt
/// let age: Option<i32> = input!("Enter your age: ").unwrap();
/// ```
#[macro_export]
macro_rules! input {
    () => {
        $crate::read_and_parse_with_eof::<String>()
    };
    ($prompt:expr) => {
        $crate::read_and_parse_with_eof_from(&mut ::std::io::stdin().lock(), Some($prompt))
    };
}

/// A macro for reading and parsing input from stdin that does not handle EOF gracefully.
///
/// - `input_no_eof!()` reads a line and tries to parse it into a `String`.
///   On EOF or parsing error, returns an `Err`.
///
/// - `input_no_eof!("prompt")` prints a prompt before reading.
///
/// This macro uses `read_and_parse`, which returns an error on EOF rather than `None`.
///
/// # Examples
///
/// ```no_run
/// // Without a prompt
/// let name: String = input_no_eof!().unwrap();
///
/// // With a prompt
/// let age: i32 = input_no_eof!("Enter your age: ").unwrap();
/// ```
#[macro_export]
macro_rules! input_no_eof {
    () => {
        $crate::read_and_parse::<String>()
    };
    ($prompt:expr) => {
        $crate::read_and_parse_from(&mut ::std::io::stdin().lock(), Some($prompt))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_and_parse_float() {
        let input_data = b"3.14\n";
        let mut reader = Cursor::new(input_data);
        let result: f64 = read_and_parse_from(&mut reader, None).unwrap();
        assert_eq!(result, 3.14);
    }

    #[test]
    fn test_read_and_parse_integer() {
        let input_data = b"42\n";
        let mut reader = Cursor::new(input_data);
        let result: i32 = read_and_parse_from(&mut reader, None).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_read_and_parse_string() {
        let input_data = b"hello world\n";
        let mut reader = Cursor::new(input_data);
        let result: String = read_and_parse_from(&mut reader, None).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_read_and_parse_with_prompt() {
        let input_data = b"100\n";
        let mut reader = Cursor::new(input_data);
        let result: i32 = read_and_parse_from(&mut reader, Some("Enter a number: ")).unwrap();
        assert_eq!(result, 100);
    }

    #[test]
    fn test_unix_line_ending() {
        let input_data = b"123\n";
        let mut reader = Cursor::new(input_data);
        let result: i32 = read_and_parse_from(&mut reader, None).unwrap();
        assert_eq!(result, 123);
    }

    #[test]
    fn test_windows_line_ending() {
        let input_data = b"456\r\n";
        let mut reader = Cursor::new(input_data);
        let result: i32 = read_and_parse_from(&mut reader, None).unwrap();
        assert_eq!(result, 456);
    }

    #[test]
    fn test_read_and_parse_with_eof() {
        let input_data = b"";
        let mut reader = Cursor::new(input_data);
        let result: Option<i32> = read_and_parse_with_eof_from(&mut reader, None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_and_parse_with_eof_nonempty() {
        let input_data = b"789\n";
        let mut reader = Cursor::new(input_data);
        let result: Option<i32> = read_and_parse_with_eof_from(&mut reader, None).unwrap();
        assert_eq!(result, Some(789));
    }
}