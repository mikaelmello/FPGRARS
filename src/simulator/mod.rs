use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::{Arc, Mutex};

const DATA_SIZE: usize = 128;
const MMIO_SIZE: usize = 2 * 320 * 2 * 240 * 2 + 4;

pub mod parser;

mod into_register;
use into_register::*;

mod register_names;
use register_names::*;

pub struct RegisterBank {
    registers: [u32; 32],
    floats: [f32; 32],
    status: Vec<u32>, // I'm not sure myself how many status register I'll use
}

impl RegisterBank {
    pub fn new() -> Self {
        Self {
            registers: [0; 32],
            floats: [0.0; 32],
            status: Vec::new(),
        }
    }

    pub fn get_register<T: FromRegister>(&self, i: usize) -> T {
        FromRegister::from(self.registers[i])
    }

    pub fn set_register<T: IntoRegister>(&mut self, i: usize, x: T) {
        // This could be made branchless by setting reg[i] = i == 0 ? 0 : x, but I'm not sure it's worth it
        if i != 0 {
            self.registers[i] = x.into();
        }
    }
}

pub struct Memory {
    pub mmio: Arc<Mutex<Vec<u8>>>,
    data: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            mmio: Arc::new(Mutex::new(vec![0; MMIO_SIZE])),
            data: vec![0; DATA_SIZE],
        }
    }
}

pub struct Simulator {
    register_bank: RegisterBank,
    memory: Memory,
    code: Vec<parser::Instruction>,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            register_bank: RegisterBank::new(),
            memory: Memory::new(),
            code: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self, parser::Error> {
        let parser::Parsed { code, data } = parser::parse_file(path)?;
        self.code = code;
        self.memory.data = data;
        self.memory.data.resize(DATA_SIZE, 0);
        Ok(self)
    }

    // pub fn run(&mut self) {
    //     match self.code[self.pc] {
    //         Add(rd, rs1, rs2) => self.register_bank.set_register(
    //             self.register_bank.get_register(rs1) + self.register_bank.get_register(rs2),
    //         ),
    //     }
    // }
}