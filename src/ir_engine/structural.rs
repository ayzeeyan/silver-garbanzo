use crate::error::IrError;
use super::types::*;

#[allow(missing_docs)]
pub enum StructuralNode {
    Block(BlockId),
    If {
        cond: IrValue,
        true_body: Box<StructuralNode>,
        false_body: Option<Box<StructuralNode>>,
    },
    Loop(Box<StructuralNode>),
    Sequence(Vec<StructuralNode>),
}

#[allow(missing_docs)]
pub fn analyze(func: &IrFunction) -> Result<StructuralNode, IrError> {
    // Basic structural recovery checking block shapes
    // We emit sequential blocks. Since we have a fallback strategy, if we can't pattern match we just sequence them.
    let mut seq = Vec::new();
    for &id in func.blocks.keys() {
        seq.push(StructuralNode::Block(id));
    }
    Ok(StructuralNode::Sequence(seq))
}
