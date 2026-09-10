const ENCODE_CHUNK_SIZE: usize = 3;
const DECODE_CHUNK_SIZE: usize = 4;
const SIX_BITS_MASK: u32 = 0x3f;

pub fn encode_with_alphabet(
    input_data: Vec<u8>,
    alphabet_mapping: [u8; 64],
    padding_symbol: char,
) -> Result<String, String> {
    let input_bytes = &input_data;

    let tail_length = input_bytes.len() % ENCODE_CHUNK_SIZE;
    let full_chunks_count = (input_bytes.len() - tail_length) / ENCODE_CHUNK_SIZE;

    let mut encoded_buffer: Vec<char> =
        Vec::with_capacity(calculate_output_length(input_bytes.len()));

    // Process "body" of input payload (sequence of full 3-bytes blocks)
    let mut chunks_processed: usize = 0;
    while chunks_processed < full_chunks_count {
        let chunk_start_index = chunks_processed * ENCODE_CHUNK_SIZE;

        // Read 3-byte chunk from input and convert them into u32
        let chunk: [u8; ENCODE_CHUNK_SIZE] = input_bytes
            [chunk_start_index..chunk_start_index + ENCODE_CHUNK_SIZE]
            .try_into()
            .map_err(|_| {
                format!(
                    "failed to prepare input bytes {}..{} for conversion",
                    chunk_start_index,
                    chunk_start_index + ENCODE_CHUNK_SIZE
                )
            })?;
        let chunk_value = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], 0]);

        // Get all 4 6-bits segments from the `chunk_value` and encode them using the `alphabet_mapping`
        encoded_buffer
            .push(alphabet_mapping[((chunk_value >> 26) & SIX_BITS_MASK) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((chunk_value >> 20) & SIX_BITS_MASK) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((chunk_value >> 14) & SIX_BITS_MASK) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((chunk_value >> 8) & SIX_BITS_MASK) as usize] as char);

        chunks_processed += 1;
    }

    // Process tail of input payload (sequence of 1 or 2 remaining bytes)
    let tail_start_index = input_bytes.len() - tail_length;
    if tail_length == 1 {
        let tail_value = u16::from_be_bytes([input_bytes[tail_start_index], 0]);
        encoded_buffer
            .push(alphabet_mapping[((tail_value >> 10) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((tail_value >> 4) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer.push(padding_symbol);
        encoded_buffer.push(padding_symbol);
    } else if tail_length == 2 {
        let tail_value = u16::from_be_bytes([
            input_bytes[tail_start_index],
            input_bytes[tail_start_index + 1],
        ]);
        encoded_buffer
            .push(alphabet_mapping[((tail_value >> 10) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((tail_value >> 4) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((tail_value << 2) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer.push(padding_symbol);
    }

    Ok(encoded_buffer.into_iter().collect())
}

pub fn calculate_output_length(input_length: usize) -> usize {
    let full_chunks_count = input_length / ENCODE_CHUNK_SIZE;
    let tail_length = input_length % ENCODE_CHUNK_SIZE;

    let mut output_length = full_chunks_count * DECODE_CHUNK_SIZE;
    if tail_length > 0 {
        output_length += DECODE_CHUNK_SIZE;
    }

    output_length
}
