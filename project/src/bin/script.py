strings = [
    "if",
    "else",
    "procedure",
    "is",
    "global",
    "variable",
    "begin",
    "then",
    "end",
    "program"
]

for word in strings:
    print('("' + word + '", Token::new(tokenTypeEnum::' + word.upper() + ', "' + word + '".to_string())),')