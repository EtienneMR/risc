use crate::{
    parser::{ast::Block, cursor::Cursor, error::ParseError, expr::infix::parse_infix},
    tokenizer::TokenKind,
};

pub fn parse_block(cursor: &mut Cursor, terminators: &[TokenKind]) -> Result<Block, ParseError> {
    let mut block = Block::new();

    loop {
        let kind = &cursor.peek()?.kind;
        if terminators.iter().any(|t| t == kind) {
            break;
        }
        block.exprs.push(parse_infix(cursor)?);
    }

    Ok(block)
}
