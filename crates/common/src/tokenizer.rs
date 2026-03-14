pub struct Tokenizer;

impl Tokenizer {
  pub fn count_string_tokens(input: &str) -> u32 {
    let tokenizer = tiktoken_rs::o200k_base_singleton();
    let tokens = tokenizer.encode_ordinary(input);

    tokens.len() as u32
  }

  pub fn truncate_output(output: &str, max_tokens: usize) -> String {
    let tokenizer = tiktoken_rs::o200k_base_singleton();
    let mut tokens = tokenizer.encode_ordinary(output);

    if tokens.len() > max_tokens {
      tokens.reverse();
      tokens.truncate(max_tokens);
      tokens.reverse();

      tokenizer.decode(tokens).unwrap_or_default()
    } else {
      output.to_string()
    }
  }

  pub fn tokenize_string(output: &str) -> Vec<String> {
    let tokenizer = tiktoken_rs::o200k_base_singleton();
    let ranks = tokenizer.encode_ordinary(output);

    ranks.iter().map(|rank| tokenizer.decode(vec![*rank]).unwrap_or_default()).collect()
  }
}
