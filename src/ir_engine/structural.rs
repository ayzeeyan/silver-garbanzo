use crate::error::IrError;
use super::types::*;

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

pub fn analyze(_func: &IrFunction) -> Result<StructuralNode, IrError> {
    // Scaffold for pattern-matching over dominator tree for structures
    Ok(StructuralNode::Sequence(Vec::new()))
}
