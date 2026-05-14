use super::ast::{Expr, Stmt, Block};
use super::instruction::Instruction;
use super::opcodes::Opcode;
use super::constants::LuaConstant;
use super::proto::FunctionProto;
use std::borrow::Cow;
use std::collections::HashMap;

#[allow(missing_docs)]
pub struct Compiler<'a> {
    instructions: Vec<Instruction>,
    constants: Vec<LuaConstant<'a>>,
    locals: HashMap<String, u8>,
    next_reg: u8,
}

impl<'a> Default for Compiler<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Compiler<'a> {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            locals: HashMap::new(),
            next_reg: 0,
        }
    }

    fn get_reg(&mut self) -> u8 {
        let r = self.next_reg;
        self.next_reg += 1;
        r
    }

    fn add_const(&mut self, c: LuaConstant<'a>) -> u16 {
        if let Some(pos) = self.constants.iter().position(|x| x == &c) {
            return pos as u16;
        }
        let pos = self.constants.len() as u16;
        self.constants.push(c);
        pos
    }

    #[allow(missing_docs)]
    pub fn compile_block(&mut self, block: &Block) {
        for stmt in &block.0 {
            self.compile_stmt(stmt);
        }
        self.instructions.push(Instruction {
            opcode: Opcode::Return,
            a: 0, b: 1, c: 0, bx: 0, sbx: 0, raw: 0,
        });
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::LocalDecl(names, exprs) => {
                for (name, expr) in names.iter().zip(exprs.iter()) {
                    let r = self.compile_expr(expr);
                    self.locals.insert(name.clone(), r);
                }
            }
            Stmt::Assign(vars, exprs) => {
                if let (Some(Expr::Ident(name)), Some(expr)) = (vars.first(), exprs.first()) {
                    let r = self.compile_expr(expr);
                    if let Some(&local_reg) = self.locals.get(name) {
                        self.instructions.push(Instruction {
                            opcode: Opcode::Move,
                            a: local_reg, b: r as u16, c: 0, bx: 0, sbx: 0, raw: 0,
                        });
                    } else {
                        let k = self.add_const(LuaConstant::LuaString(Cow::Owned(name.as_bytes().to_vec())));
                        self.instructions.push(Instruction {
                            opcode: Opcode::SetGlobal,
                            a: r, b: 0, c: 0, bx: k as u32, sbx: 0, raw: 0,
                        });
                    }
                }
            }
            Stmt::CallStmt(expr) => {
                let r = self.compile_expr(expr);
                self.instructions.push(Instruction {
                    opcode: Opcode::Call,
                    a: r, b: 1, c: 1, bx: 0, sbx: 0, raw: 0,
                });
            }
            _ => {}
        }
    }

    fn compile_expr(&mut self, expr: &Expr) -> u8 {
        let r = self.get_reg();
        match expr {
            Expr::Number(n) => {
                let k = self.add_const(LuaConstant::Number(*n));
                self.instructions.push(Instruction {
                    opcode: Opcode::LoadK,
                    a: r, b: 0, c: 0, bx: k as u32, sbx: 0, raw: 0,
                });
            }
            Expr::String(s) => {
                let k = self.add_const(LuaConstant::LuaString(Cow::Owned(s.clone())));
                self.instructions.push(Instruction {
                    opcode: Opcode::LoadK,
                    a: r, b: 0, c: 0, bx: k as u32, sbx: 0, raw: 0,
                });
            }
            Expr::Ident(name) => {
                if let Some(&local_reg) = self.locals.get(name) {
                    self.instructions.push(Instruction {
                        opcode: Opcode::Move,
                        a: r, b: local_reg as u16, c: 0, bx: 0, sbx: 0, raw: 0,
                    });
                } else {
                    let k = self.add_const(LuaConstant::LuaString(Cow::Owned(name.as_bytes().to_vec())));
                    self.instructions.push(Instruction {
                        opcode: Opcode::GetGlobal,
                        a: r, b: 0, c: 0, bx: k as u32, sbx: 0, raw: 0,
                    });
                }
            }
            _ => {}
        }
        r
    }

    #[allow(missing_docs)]
    pub fn finish(self) -> FunctionProto<'a> {
        FunctionProto {
            source_name: Some(b"@obfuscated"),
            line_defined: 0,
            last_line_defined: 0,
            num_upvalues: 0,
            num_params: 0,
            is_vararg: 1,
            max_stack_size: self.next_reg,
            instructions: self.instructions,
            constants: self.constants,
            protos: Vec::new(),
            line_info: Vec::new(),
            local_vars: Vec::new(),
            upvalue_names: Vec::new(),
        }
    }
}
