use std::collections::HashMap;

// Define your Token struct
#[derive(Debug, PartialEq, Eq, Hash)]
enum TokenTypeEnum {
    // Define token types here
    // For example:
    PLUS,
    MINUS,
    NUMBER,
    IDENTIFIER,
    // Add other token types as needed
}
#[derive(Debug)]
struct Token {
    // Define fields of your Token struct
    // For example:
    token_string: String,
    tokenType: TokenTypeEnum,
}

fn main() {
    // Create a new empty hashmap with token strings as keys and Token structs as values
    let mut token_map: HashMap<String, Token> = HashMap::new();

    // Insert token-value pairs into the hashmap
    let token1 = Token { token_string: "+".to_string(), tokenType: TokenTypeEnum::PLUS };
    token_map.insert(token1.token_string.clone(), token1);

    let token2 = Token { token_string: "5".to_string(), tokenType: TokenTypeEnum::NUMBER };
    token_map.insert(token2.token_string.clone(), token2);

    // Look up a token by its string representation
    let token_to_lookup = "+";
    if let Some(token) = token_map.get(token_to_lookup) {
        println!("Token for '{}': {:?}", token_to_lookup, token);
    } else {
        println!("Token not found for '{}'", token_to_lookup);
    }

    // Iterate over token-value pairs
    for (token_string, token) in &token_map {
        println!("Token '{}': {:?}", token_string, token);
    }
}
