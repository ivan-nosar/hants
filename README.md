# HANTS

**HAN**dy **T**ool**S**et - A lightweight command-line interface utility that consolidates several small tools to streamline everyday development tasks.

## Usage

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

Use `password` command to generate new secret string that can be used as a secure password.

```sh
$ hants.exe password -h
Generate a secure password

Usage: hants.exe password [OPTIONS]

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
          - c / console:      Print output of the command to the standard console output
          - cb / clipboard:   Write output of the command to the system clipboard
          - <file path>:      Write output  of the command to the file with specified path.
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

Usage: hants.exe base64 <COMMAND>

Commands:
  encode    Encode input sequence to Base64 format
  decode    Decode input Base64 sequence
  validate  Check if input string is a valid Base64 sequence
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

<!-- TODO: Options:

```sh
# Input options. These are exclusive options and cannot be used simultaneously.
-fi <filePath>, --file-input <filePath>     Read input sequence from file with specified path.
-cbi, --clipboard-input                     Read input sequence from clipboard.
-ci <string>, --console-input <string>      Specify input sequence directly in parameters list.

# Output options. These are exclusive options and cannot be used simultaneously.
-fo <filePath>, --file-output <filePath>    Write output to the file with specified path.
                                            File must not exist prior to command execution.
-cbo, --clipboard-output                    Write output to the clipboard.
-co, --console-output                       Print output in the console. Default option.

# Alphabet options.
-ps <symbol>, --padding-symbol <symbol>             Use symbol provided as padding character.
                                                    Default: '='
-cs <symbols>, --complementary-symbols <symbols>    Use symbols provided as a replacement for default
                                                    complementary symbols (63th and 64th character in
                                                    alphabet). Default: '+/'. Can not be used along with
                                                    -a/--alphabet option.
-a <alphabet>, --alphabet <alphabet>                Use custom alphabet. Must be a string consisting
                                                    of exactly 64 unique symbols. Default:
                                                    'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'.
                                                    Can not be used along with -cs/--complementary-symbols option.
``` -->

### JSON

**TBD**

### JWT

**TBD**

## External dependencies

Built with Rust, this tool relies on a minimal set of external dependencies and avoids direct use of platform-specific APIs, ensuring maximum portability.

- [`arboard`](https://crates.io/crates/arboard): Cross-platform library for getting and setting the contents of the OS-level clipboard.
- [`clap`](https://crates.io/crates/clap): A simple to use, efficient, and full-featured Command Line Argument Parser