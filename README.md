# HANTS

**HAN**dy **T**ool**S**et - A lightweight command-line interface utility that consolidates several small tools to streamline everyday development tasks.

## Usage

> [!IMPORTANT]
> When `hants` is invoked with the `-i c` option, it reads the `STDIN` stream until an `EOF` is
> received. Two consequences of this behavior are worth noting:
>
> 1. A piped `STDIN` stream is finite only if the producing process terminates. For example,
>    `cat input.txt | hants ...` receives an `EOF` as soon as `cat` exits, whereas a process that
>    stays open indefinitely never produces one (e.g. `yes | hants ...`). This causes `hants` to
>    hang until manually interrupted by the user
> 2. Without piping, `hants` reads input from the console interactively. Pressing `Enter`/`Return`
>    only appends a new-line symbol to the input stream and does not send an `EOF` to `STDIN`. To
>    stop the input collection and start processing the data entered so far, send an `EOF`
>    explicitly: `Ctrl + D` on Linux/macOS or `Ctrl + Z` on Windows (the latter may require two
>    key presses).

<!-- TODO: Generate it automatically based on docs in source code. -->

```sh
$ hants -h
HANdy ToolSet - A lightweight command-line interface utility that consolidates several small tools to streamline everyday development tasks.

Usage: hants <COMMAND>

Commands:
  password  Generate a secure password
  base64    Encode/decode Base64 content
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Password

Use the `password` command to generate new secret string that can be used as a secure password.

```sh
$ hants password -h
Generate a secure password

Usage: hants password [OPTIONS]

Options:
  -l, --length <LENGTH>
          The length of the password [default: 12]
  -a, --symbol_classes <SYMBOL_CLASSES>
          The symbol classes for the password construction. Supported values:
          - a: Lower-case alphabetic latin symbols
          - A: Upper-case alphabetic latin symbols
          - n: Digits
          - b: Braces: ()<>[]{}
          - q: Quotes: '"`
          - p: Punctuation: !?.,;:
          - m: Math operations: +-*/=
          - w: Whitespace symbols: space, \t\n\r
          - s: Special symbols: \^~@$&%_
           [default: aAnbqpms]
  -s, --seed <SEED>
          The seed for the random values generator
  -o, --output <OUTPUT>
          The output location for the command result. Supported values:
          - c / console:      Print output of the command to the standard console output;
          - cb / clipboard:   Write output of the command to the system clipboard;
          - <file path>:      Write output of the command to the file with specified path.
                              File must not exist prior to command execution
           [default: clipboard]
  -h, --help
          Print help
```

### Base64

Use `base64` command to encode or decode [Base64](https://en.wikipedia.org/wiki/Base64) content.

```sh
$ hants base64 -h
Encode/decode Base64 content

Usage: hants base64 <COMMAND>

Commands:
  encode  Encode input sequence to Base64 format
  decode  Decode input Base64 sequence
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

> [!IMPORTANT]
> The `-i c` option may produce results that look unexpected in PowerShell (both Windows PowerShell
> and cross-platform PowerShell Core).
>
> PowerShell transfers text objects over the pipeline as discrete, line-terminated strings rather
> than as a raw byte stream. As a result, an implicit trailing newline (CRLF on Windows, LF on
> Unix-like systems) is appended whenever a string is piped to an external native program. That
> newline is treated as part of the input sequence and is encoded together with the main payload.
>
> For example, the command above produces the following output in Windows PowerShell:
> ```powershell
> > "foobar" | hants.exe base64 encode -i c -o c
> Zm9vYmFyDQo=
> # `DQo=` stands for the `\r\n` new-line sequence
> ```
>
> Using `-i <file path>` or `-i cb` (the default option) avoids this behavior in PowerShell
> environments.

> [!IMPORTANT]
> A similar behavior can be observed on Linux/macOS systems, although for a different reason: many
> built-in or commonly used CLI tools append a trailing newline to the produced output:
>
> - The `echo` command appends a newline character to the end of the provided text by default. The
>   `-n` option suppresses it: `echo -n "foobar"`.
> - The `cat` command writes the content of a file to the console as-is, but many standard text
>   editors - such as `vim` or `nano` - automatically add a newline character to the end of a file
>   on every save.
>
> Please take this behavior into account when using the `-i c` input mode on Linux/macOS systems.

#### Encode

Use the `base64 encode` command to encode data into [Base64](https://en.wikipedia.org/wiki/Base64)
format.

```sh
$ hants base64 encode -h
Encode input sequence to Base64 format

Usage: hants base64 encode [OPTIONS]

Options:
  -o, --output <OUTPUT>
          The output location for the command result. Supported values:
          - c / console:      Print output of the command to the standard console output;
          - cb / clipboard:   Write output of the command to the system clipboard;
          - <file path>:      Write output of the command to the file with specified path.
                              File must not exist prior to command execution
           [default: clipboard]
  -i, --input <INPUT>
          The target location for command to consume input from. Supported values:
          - c / console:      Read input for the command from the stdin. Most suitable for using with pipes;
          - cb / clipboard:   Read input for the command from the system clipboard;
          - <file path>:      Read input for the command from the file with specified path.
                              File must exist prior to command execution
           [default: clipboard]
  -a, --alphabet <ALPHABET>
          Use custom alphabet. Must be a string consisting of exactly
          64 unique symbols. If not provided - default alphabet is used:
          ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/
  -c, --complementary-symbols <COMPLEMENTARY_SYMBOLS>
          Use symbols provided as a replacement for default complementary symbols
          (63th and 64th character in alphabet: +/).
  -p, --padding-symbol <PADDING_SYMBOL>
          Use symbol provided as padding character. [default: =]
  -n, --no-pad
          Disable padding. When encoding, no trailing padding symbols
          are emitted; when decoding, the input is expected to carry none.
  -h, --help
          Print help
```

**Usage examples**

1. Read input from the clipboard and print the result to the console:
```sh
# Assuming the clipboard contains the "foobar" text
$ hants base64 encode -i cb -o c
Zm9vYmFy
```

2. Read input from a file and print the result to the console:
```sh
# Assuming './input.txt' exists and contains the "foobar" text
$ hants base64 encode -i ./input.txt -o c
Zm9vYmFy
```

3. Read input from `STDIN` and print the result to the console:
```sh
$ "foobar" | hants base64 encode -i c -o c
Zm9vYmFy
```

4. Use a custom alphabet; the input is implicitly read from the clipboard:
```sh
# Assuming the clipboard contains the "foobar" text
$ hants base64 encode -a 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+/ -o c
PczlOc5o
```

5. Replace the default complementary symbols; the input is implicitly read from the clipboard:
```sh
# Assuming the clipboard contains the "ÿÿÿ" text
$ hants base64 encode -c -_ -o c
w7_Dv8O_
```

6. Encode with a custom padding symbol; the input is implicitly read from the clipboard:
```sh
# Assuming the clipboard contains the "foob" text
$ hants base64 encode -p _ -o c
Zm9vYg__
```

7. Encode without padding; the input is implicitly read from the clipboard:
```sh
# Assuming the clipboard contains the "foob" text
$ hants base64 encode -n -o c
Zm9vYg
```

#### Decode

Use the `base64 decode` command to decode [Base64](https://en.wikipedia.org/wiki/Base64) encoded
data to the original (potentially non-text) state.

```sh
$ hants base64 decode -h
Decode input Base64 sequence

Usage: hants base64 decode [OPTIONS]

Options:
  -o, --output <OUTPUT>
          The output location for the command result. Supported values:
          - c / console:      Print output of the command to the standard console output;
          - cb / clipboard:   Write output of the command to the system clipboard;
          - <file path>:      Write output of the command to the file with specified path.
                              File must not exist prior to command execution
           [default: clipboard]
  -i, --input <INPUT>
          The target location for command to consume input from. Supported values:
          - c / console:      Read input for the command from the stdin. Most suitable for using with pipes;
          - cb / clipboard:   Read input for the command from the system clipboard;
          - <file path>:      Read input for the command from the file with specified path.
                              File must exist prior to command execution
           [default: clipboard]
  -a, --alphabet <ALPHABET>
          Use custom alphabet. Must be a string consisting of exactly
          64 unique symbols. If not provided - default alphabet is used:
          ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/
  -c, --complementary-symbols <COMPLEMENTARY_SYMBOLS>
          Use symbols provided as a replacement for default complementary symbols
          (63th and 64th character in alphabet: +/).
  -p, --padding-symbol <PADDING_SYMBOL>
          Use symbol provided as padding character. [default: =]
  -n, --no-pad
          Disable padding. When encoding, no trailing padding symbols
          are emitted; when decoding, the input is expected to carry none.
  -h, --help
          Print help
```

> [!TIP]
> Decoding parameters must match the ones the payload was encoded with: `hants` relies entirely on
> what is passed on the command line and never tries to infer them from the payload itself.
>
> If the input was produced with a custom alphabet or a custom padding symbol, pass the same `-a`,
> `-c` or `-p` value to `base64 decode`. If the input carries no padding at all - a common case for
> data transferred over a network - `-n` is mandatory. Mismatched parameters lead either to an
> explicit error or, in some cases, to a silently asymmetric result.
>
> For instance, `base64 encode -p ' '` produces a payload padded with spaces, which is then
> ambiguous to decode: both `base64 decode -p ' '` and `base64 decode -n` accept it, because the
> decoder trims surrounding whitespace before processing. Omitting the padding-related arguments
> altogether, on the other hand, makes `hants` reject the payload with the block size alignment
> error described below.

> [!IMPORTANT]
> Without the `-n` flag, the `base64 decode` command does not accept unaligned or non-padded
> payloads. If the length of the input sequence is not aligned to the decoding block size - 4
> bytes - `hants` cannot process such a payload and reports an error:
> ```sh
> # Assuming the clipboard contains the "Zm9" text - an incomplete Base64 sequence.
> $ hants base64 decode -i cb -o c
> Error: input payload is malformed: its length must be aligned to the decoding block size of 4 bytes, but the actual length is 3 bytes.
> ```

> [!TIP]
> Respecting the newline-related behavior of PowerShell and of standard CLI tools on Linux/macOS,
> `hants` makes a best effort to avoid failures caused by whitespace surrounding the input payload.
> All ASCII whitespace symbols are trimmed from the beginning and the end of the input sequence
> before the actual decoding. The trim process removes the following symbols:
>
> - `U+0020` Space (` `)
> - `U+0009` Horizontal Tab (`\t`)
> - `U+000A` Line Feed (`\n`)
> - `U+000C` Form Feed (`\f`)
> - `U+000D` Carriage Return (`\r`)
>
> The only exception is when `--padding-symbol` is set to the `U+0020` Space symbol - the only
> *printable* ASCII whitespace symbol. In that case `hants` still trims every other whitespace
> symbol from both ends until the padding symbol or any other non-whitespace symbol is encountered.
> The remaining untrimmed part is treated as raw input, and the decoding attempt is performed.

**Usage examples**

1. Read input from the clipboard and print the result to the console:
```sh
# Assuming the clipboard contains the "Zm9vYmFy" text
$ hants base64 decode -i cb -o c
foobar
```

2. Read input from a file and print the result to the console:
```sh
# Assuming './input.txt' exists and contains the "Zm9vYmFy" text
$ hants base64 decode -i ./input.txt -o c
foobar
```

3. Read input from `STDIN` and print the result to the console:
```sh
$ "Zm9vYmFy" | hants base64 decode -i c -o c
foobar

echo " \tZm9vYmFy\n \r" | hants base64 decode -i c -o c
foobar
```

4. Use a custom alphabet; the input is implicitly read from the clipboard:
```sh
# Assuming the clipboard contains the "PczlOc5o" text
$ hants base64 decode -a 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+/ -o c
foobar
```

5. Replace the default complementary symbols; the input is implicitly read from the clipboard:
```sh
# Assuming the clipboard contains the "w7_Dv8O_" text
$ hants base64 decode -c -_ -o c
ÿÿÿ
```

6. Decode a payload padded with a custom symbol; the input is implicitly read from the clipboard:
```sh
# Assuming the clipboard contains the "Zm9vYg__" text
$ hants base64 decode -p _ -o c
foob
```

7. Decode a payload without padding; the input is implicitly read from the clipboard:
```sh
# Assuming the clipboard contains the "Zm9vYg" text
$ hants base64 decode -n -o c
foob
```

> [!IMPORTANT]
> By nature, the output of the `base64 decode` command can hold non-printable, binary data.
>
> This is not an issue when the output is directed to a file with the `-o <file path>` option; in
> that case the resulting stream of bytes is written verbatim, with no changes. However, the
> `console` and `clipboard` output streams accept nothing but a valid UTF-8 text sequence.
>
> Taking this into account, `hants` converts every non-printable byte sequence into a valid UTF-8
> symbol before sending it to the output stream. For instance, decoding deliberately non-printable
> data replaces all invalid UTF-8 characters with readable alternatives:
> ```sh
> # Assuming the clipboard contains the "lYlRMvG3" text
> $ hants base64 decode -i cb -o c
> Note: binary data detected; the visible representation may not reflect the actual content.
> ��Q2�
> ```
>
> On the other hand, if the output sequence already consists of valid UTF-8 characters only, it is
> printed verbatim:
> ```sh
> # Assuming the clipboard contains the "SSDimaUgaGFudHM=" text
> $ hants base64 decode -i cb -o c
> I ♥ hants
> ```

### JSON

**TBD**

### JWT

**TBD**

### GUID

**TBD**

## External dependencies

Built with Rust, this tool relies on a minimal set of external dependencies and avoids direct use of platform-specific APIs, ensuring maximum portability.

- [`arboard`](https://crates.io/crates/arboard): Cross-platform library for getting and setting the contents of the OS-level clipboard.
- [`clap`](https://crates.io/crates/clap): A simple to use, efficient, and full-featured Command Line Argument Parser
