pub const HANTS_HELP_TEXT: &str = "\
Usage: hants [command] [options...]

Commands:
    help            Use `help` command to get the usage instructions.
                    Example: hants help

    base64          Use `base64` command to encode or decode Base64 content.
                    Example: hants base64 [action] [options...]

        Supported actions:
            encode      Encode input sequence to Base64 format.
            decode      Decode input Base64 sequence.
            encode      Check if input string is a valid Base64 sequence.

        Options:
            Input options. These are exclusive options and cannot be used simultaneously.
                -fi <filePath>, --file-input <filePath>     Read input sequence from file with specified path.
                -cbi, --clipboard-input                     Read input sequence from clipboard.
                -ci <string>, --console-input <string>      Specify input sequence directly in parameters list.

            Output options. These are exclusive options and cannot be used simultaneously.
                -fo <filePath>, --file-output <filePath>    Write output to the file with specified path.
                                                            File must not exist prior to command execution.
                -cbo, --clipboard-output                    Write output to the clipboard.
                -co, --console-output                       Print output in the console. Default option.

            Alphabet options.
                -ps <symbol>, --padding-symbol <symbol>             Use symbol provided as padding character.
                                                                    Default: '='
                -cs <symbols>, --complementary-symbols <symbols>    Use symbols provided as a replacement for default
                                                                    complementary symbols (63th and 64th character in
                                                                    alphabet). Default: '+/'.
                -a <alphabet>, --alphabet <alphabet>                Use custom alphabet. Must be a string consisting
                                                                    of exactly 64 unique symbols. Default:
                                                                    'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'";