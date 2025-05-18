use std::{collections::HashMap, fmt::Display, rc::Rc};

use bincode::{decode_from_reader, encode_into_writer, error::DecodeError, Decode, Encode};

use crate::utils::lang::AstPos;

use super::object::UniLValue;

#[derive(Clone, Debug, Encode, Decode)]
pub enum Instruction {
    LoadName(u64),
    DefineName(u64),
    StoreName(u64),
    DropName(u64),
    SetField(u64),
    GetField(u64),
    SetIndex,
    GetIndex,

    BeginScope,
    EndScope,

    LoadConst(u64),
    Pop,
    Clone,
    Clone2,
    Insert(u8),
    Null,
    One,
    Zero,
    Object,
    List(u64),
    NextTask,

    Jmp(u64),
    IfJmp(u64),
    IfNotJmp(u64),

    Call(u8),
    Return,

    Catch(u64),
    PopCatch,
    Throw,
    ThrowReturn,

    Neg,
    Not,
    Inv,
    Mul,
    Add,
    Div,
    Mod,
    Sub,
    BitAnd,
    Xor,
    BitOr,
    Shl,
    Shr,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,

    FunctionDecl {
        address: u64,
        name_idx: u64,
        parameters: Vec<u64>,
        algorithm_type: Option<u64>
    },

    End
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoadName(_) => write!(f, "LoadName"),
            Self::DefineName(_) => write!(f, "DefineName"),
            Self::StoreName(_) => write!(f, "StoreName"),
            Self::DropName(_) => write!(f, "DropName"),
            Self::SetField(_) => write!(f, "SetField"),
            Self::GetField(_) => write!(f, "GetField"),
            Self::SetIndex => write!(f, "SetIndex"),
            Self::GetIndex => write!(f, "GetIndex"),
            Self::BeginScope => write!(f, "BeginScope"),
            Self::EndScope => write!(f, "EndScope"),
            Self::LoadConst(_) => write!(f, "LoadConst"),
            Self::Pop => write!(f, "Pop"),
            Self::Clone => write!(f, "Clone"),
            Self::Clone2 => write!(f, "Clone2"),
            Self::Insert(_) => write!(f, "Insert"),
            Self::Null => write!(f, "Null"),
            Self::One => write!(f, "One"),
            Self::Zero => write!(f, "Zero"),
            Self::Object => write!(f, "Object"),
            Self::List(_) => write!(f, "List"),
            Self::NextTask => write!(f, "NextTask"),
            Self::Jmp(_) => write!(f, "Jmp"),
            Self::IfJmp(_) => write!(f, "IfJmp"),
            Self::IfNotJmp(_) => write!(f, "IfNotJmp"),
            Self::Call(_) => write!(f, "Call"),
            Self::Return => write!(f, "Return"),
            Self::Catch(_) => write!(f, "Catch"),
            Self::PopCatch => write!(f, "PopCatch"),
            Self::Throw => write!(f, "Throw"),
            Self::ThrowReturn => write!(f, "ThrowReturn"),
            Self::Neg => write!(f, "Neg"),
            Self::Not => write!(f, "Not"),
            Self::Inv => write!(f, "Inv"),
            Self::Mul => write!(f, "Mul"),
            Self::Add => write!(f, "Add"),
            Self::Div => write!(f, "Div"),
            Self::Mod => write!(f, "Mod"),
            Self::Sub => write!(f, "Sub"),
            Self::BitAnd => write!(f, "BitAnd"),
            Self::Xor => write!(f, "Xor"),
            Self::BitOr => write!(f, "BitOr"),
            Self::Shl => write!(f, "Shl"),
            Self::Shr => write!(f, "Shr"),
            Self::Lt => write!(f, "Lt"),
            Self::Le => write!(f, "Le"),
            Self::Gt => write!(f, "Gt"),
            Self::Ge => write!(f, "Ge"),
            Self::Eq => write!(f, "Eq"),
            Self::Ne => write!(f, "Ne"),
            Self::FunctionDecl { .. } => write!(f, "FunctionDecl"),
            Self::End => write!(f, "End"),
        }
    }
}

#[derive(Debug)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub constants:    Vec<UniLValue>,
    pub positions:    Vec<AstPos>,
    pub names:        Vec<Rc<str>>,
}

impl Bytecode {
    pub fn new() -> Self {
        Bytecode { 
            instructions: Vec::new(), 
            constants: Vec::new(), 
            positions: Vec::new(), 
            names: Vec::new() 
        }
    }

    fn disassemble_internal(&self, from: usize, to: usize) -> String {
        let mut result = String::new();

        for i in from .. to {
            let instruction = &self.instructions[i];

            result.push_str(&format!("{:#018x}: {instruction}", i));

            match instruction {
                Instruction::LoadName(name) |
                Instruction::DefineName(name) |
                Instruction::StoreName(name) |
                Instruction::DropName(name) |
                Instruction::SetField(name) |
                Instruction::GetField(name) => {
                    result.push_str(&format!(" ({})", self.names[*name as usize]));
                }
                Instruction::LoadConst(constant) => {
                    result.push_str(&format!(" ({})", self.constants[*constant as usize].stringify()));
                }
                Instruction::Insert(x) |
                Instruction::Call(x) => {
                    result.push_str(&format!(" ({})", *x));
                }
                Instruction::List(x) |
                Instruction::Jmp(x) |
                Instruction::IfJmp(x) |
                Instruction::IfNotJmp(x) |
                Instruction::Catch(x) => {
                    result.push_str(&format!(" ({:#018x})", *x));
                }
                _ => ()
            }

            result.push('\n');
        }

        result
    }

    #[allow(dead_code)]
    pub fn disassemble_function(&self, address: u64) -> String {
        let mut end = address as usize;
        while end < self.instructions.len() {
            if matches!(self.instructions[end], Instruction::FunctionDecl { .. }) {
                break;
            }

            end += 1;
        }

        self.disassemble_internal(address as usize, end)
    }
    
    #[allow(dead_code)]
    pub fn disassemble(&self) -> String {
        self.disassemble_internal(0, self.instructions.len())
    }

    #[allow(dead_code)]
    pub fn disassemble_one(&self, address: u64) -> String {
        self.disassemble_internal(address as usize, address as usize + 1)
    }
}

impl Encode for Bytecode {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        let mut writer = encoder.writer();
        encode_into_writer(&self.instructions, &mut writer, bincode::config::standard())?;
        encode_into_writer(&self.constants, &mut writer, bincode::config::standard())?;

        encode_into_writer(self.positions.len() as u64, &mut writer, bincode::config::standard())?;

        // store each source only once, mapped to its respective filename
        let mut sources: HashMap<&str, &str> = HashMap::new();
        let mut last_file = None;
        for pos in &self.positions {
            // TODO: instead of appending 0 or 1 (1 extra byte) for every position, 
            // we could instead insert the amount of positions after which a new file change is present,
            // after we specify a file, something like: [ ... "foo.txt" 2 <pos 1> <pos 2> "bar.txt" ...]

            if last_file.is_none_or(|last_file| last_file != pos.filename.as_ref()) {
                if !sources.contains_key(pos.filename.as_ref()) {
                    sources.insert(&pos.filename, &pos.source);
                }

                last_file.replace(pos.filename.as_ref()); // instead of specifying filename for every position, only specify it when changed
                encode_into_writer(1u8, &mut writer, bincode::config::standard())?;
                encode_into_writer(last_file.unwrap(), &mut writer, bincode::config::standard())?;
            } else {
                encode_into_writer(0u8, &mut writer, bincode::config::standard())?;
            }

            encode_into_writer(pos.line, &mut writer, bincode::config::standard())?;
            encode_into_writer(pos.start, &mut writer, bincode::config::standard())?;
            encode_into_writer(pos.end, &mut writer, bincode::config::standard())?;
        }

        encode_into_writer(sources, &mut writer, bincode::config::standard())?;
        encode_into_writer(&self.names, &mut writer, bincode::config::standard())?;
        Ok(())
    }
}

impl<C> Decode<C> for Bytecode {
    fn decode<D: bincode::de::Decoder<Context = C>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let mut reader = decoder.reader();
        let instructions = decode_from_reader(&mut reader, bincode::config::standard())?;
        let constants = decode_from_reader(&mut reader, bincode::config::standard())?;
        
        let n_positions: u64 = decode_from_reader(&mut reader, bincode::config::standard())?;
        
        let mut positions = Vec::with_capacity(n_positions as usize);
        let mut last_file: Option<Rc<str>> = None;
        for _ in 0 .. n_positions {
            let type_: u8 = decode_from_reader(&mut reader, bincode::config::standard())?;

            if type_ == 1 {
                last_file.replace(decode_from_reader(&mut reader, bincode::config::standard())?);
            }

            let line = decode_from_reader(&mut reader, bincode::config::standard())?;
            let start = decode_from_reader(&mut reader, bincode::config::standard())?;
            let end = decode_from_reader(&mut reader, bincode::config::standard())?;

            if last_file.is_none() {
                return Err(DecodeError::Other("Malformed bytecode"));
            }

            positions.push(AstPos {
                source: Rc::from(""),
                filename: Rc::clone(last_file.as_ref().unwrap()),
                start, end, line,
            });
        }

        let sources: HashMap<Rc<str>, Rc<str>> = decode_from_reader(&mut reader, bincode::config::standard())?;
        for pos in positions.iter_mut() {
            if let Some(source) = sources.get(&pos.filename) {
                pos.source = Rc::clone(source);
            } else {
                return Err(DecodeError::Other("Malformed bytecode"));
            }
        }

        let names = decode_from_reader(&mut reader, bincode::config::standard())?;
        Ok(Bytecode { instructions, constants, positions, names })
    }
}