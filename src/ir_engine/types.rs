use indexmap::IndexMap;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub u32);

impl Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FunctionId(pub u32);

impl Display for FunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "F{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IrValue(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
    Nil,
    Bool,
    Number,
    String,
    Table,
    Function,
    Thread,
    Userdata,
    Any,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrConst {
    Nil,
    Bool(bool),
    Number(f64),
    String(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrOp {
    LoadConst(IrValue, IrConst),
    Move(IrValue, IrValue),
    GetUpvalue(IrValue, u8),
    SetUpvalue(u8, IrValue),
    GetGlobal(IrValue, String),
    SetGlobal(String, IrValue),
    GetTable(IrValue, IrValue, IrValue),
    SetTable(IrValue, IrValue, IrValue),
    NewTable(IrValue),
    SelfOp(IrValue, IrValue, IrValue),
    Add(IrValue, IrValue, IrValue),
    Sub(IrValue, IrValue, IrValue),
    Mul(IrValue, IrValue, IrValue),
    Div(IrValue, IrValue, IrValue),
    Mod(IrValue, IrValue, IrValue),
    Pow(IrValue, IrValue, IrValue),
    Unm(IrValue, IrValue),
    Not(IrValue, IrValue),
    Len(IrValue, IrValue),
    Concat(IrValue, Vec<IrValue>),
    Eq(IrValue, IrValue, IrValue),
    Lt(IrValue, IrValue, IrValue),
    Le(IrValue, IrValue, IrValue),
    Test(IrValue, IrValue),
    TestSet(IrValue, IrValue, IrValue),
    Call(Vec<IrValue>, IrValue, Vec<IrValue>),
    TailCall(IrValue, Vec<IrValue>),
    Return(Vec<IrValue>),
    ForLoop(IrValue, IrValue),
    ForPrep(IrValue, IrValue),
    TForLoop(IrValue, IrValue, IrValue),
    SetList(IrValue, u8, u8),
    Close(IrValue),
    Closure(IrValue, FunctionId),
    VarArg(Vec<IrValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    Branch(BlockId),
    CondBranch(IrValue, BlockId, BlockId),
    Return(Vec<IrValue>),
    TailCall(IrValue, Vec<IrValue>),
    Unreachable,
}

#[derive(Debug, Clone)]
pub struct PhiNode {
    pub dest: IrValue,
    pub incoming: Vec<(BlockId, IrValue)>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub phis: Vec<PhiNode>,
    pub ops: Vec<IrOp>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, Default)]
pub struct ControlFlowGraph {
    pub successors: IndexMap<BlockId, Vec<BlockId>>,
    pub predecessors: IndexMap<BlockId, Vec<BlockId>>,
    pub dominators: IndexMap<BlockId, BlockId>,
    pub dominance_frontiers: IndexMap<BlockId, Vec<BlockId>>,
}

#[derive(Debug, Clone)]
pub struct UpvalueDesc {
    pub name: String,
    pub in_stack: bool,
    pub index: u8,
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub id: FunctionId,
    pub blocks: IndexMap<BlockId, BasicBlock>,
    pub cfg: ControlFlowGraph,
    pub params: Vec<IrValue>,
    pub upvalues: Vec<UpvalueDesc>,
    pub is_vararg: bool,
    pub max_stack_size: u8,
}

#[derive(Debug, Clone)]
pub struct IrModule {
    pub functions: Vec<IrFunction>,
    pub entry: FunctionId,
}
