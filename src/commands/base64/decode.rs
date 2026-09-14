use crate::base64::alphabet::{build_decoding_alphabet_mapping, validate_alphabet, validate_padding_symbol};
use crate::base64::engine::decode_with_alphabet;
use crate::commands::base64::Args;
use crate::io::{read_input_bytes, write_output_bytes};

pub fn run(args: Args) -> Result<(), String> {
    let alphabet = validate_alphabet(args.alphabet, args.complementary_symbols)?;

    let padding_symbol = validate_padding_symbol(args.padding_symbol, &alphabet)?;

    let alphabet_mapping = build_decoding_alphabet_mapping(&alphabet);

    let input_data = read_input_bytes(args.input)?;

    let decoded_data = decode_with_alphabet(&input_data, alphabet_mapping, padding_symbol)?;

    write_output_bytes(args.output, &decoded_data)
}
