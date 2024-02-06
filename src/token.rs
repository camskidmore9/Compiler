//Token Class
struct Token;
impl Token {
    enum tokenType {
        PLUS,
        MINUS,
        IF_RW,
        LOOP_RW,
        END_RW,
        L_PAREN,
        R_PAREN,
        L_BRACKET,
        R_BRACKET,
        NUMBER,
        IDENTIFIER,
        EOF
    }

    fn bark(&self) {
        println!("baf!");

    }
}