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

Usage: hants.exe <COMMAND>

Commands:
  password  Generate a secure password
  base64    Encode/decode/validate Base64 content
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
Encode/decode/validate Base64 content

Usage: hants base64 <COMMAND>

Commands:
  encode    Encode input sequence to Base64 format
  decode    Decode input Base64 sequence
  validate  Check if input sequence is a valid Base64 payload
  length    Calculate the length of the Base64 encoded sequence for a given input. No encoding is performed.
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

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

### JSON

**TBD**

### JWT

**TBD**

## External dependencies

Built with Rust, this tool relies on a minimal set of external dependencies and avoids direct use of platform-specific APIs, ensuring maximum portability.

- [`arboard`](https://crates.io/crates/arboard): Cross-platform library for getting and setting the contents of the OS-level clipboard.
- [`clap`](https://crates.io/crates/clap): A simple to use, efficient, and full-featured Command Line Argument Parser