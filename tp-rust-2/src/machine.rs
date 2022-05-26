use std::io::{self, Write};

const MEMORY_SIZE: usize = 4096;
const NREGS: usize = 16;

const IP: usize = 0;

pub struct Machine {
    memory : [u8; MEMORY_SIZE],
    registers : [u32; NREGS]
}

#[derive(Debug)]
pub enum MachineError {
    OutOfMemory,
    InexistantInstruction,
    InexistantRegister,
    IoError(std::io::Error), //Error for out instructions
}

impl Machine {
    /// Create a new machine in its reset state. The `memory` parameter will
    /// be copied at the beginning of the machine memory.
    ///
    /// # Panics
    /// This function panics when `memory` is larger than the machine memory.
    pub fn new(memory: &[u8]) -> Self {
        let mut machine = Self {
            memory: [0; MEMORY_SIZE],
            registers: [0; NREGS]
        };
        machine.memory[..memory.len()].copy_from_slice(memory);
        machine
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on `fd`.
    pub fn run_on<T: Write>(&mut self, fd: &mut T) -> Result<(), MachineError> {
        while !self.step_on(fd)? {}
        Ok(())
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on standard output.
    pub fn run(&mut self) -> Result<(), MachineError> {
        self.run_on(&mut io::stdout().lock())
    }

    /// Execute the next instruction by doing the following steps:
    ///   - decode the instruction located at IP (register 0)
    ///   - increment the IP by the size of the instruction
    ///   - execute the decoded instruction
    ///
    /// If output instructions are run, they print on `fd`.
    /// If an error happens at either of those steps, an error is
    /// returned.
    ///
    /// In case of success, `true` is returned if the program is
    /// terminated (upon encountering an exit instruction), or
    /// `false` if the execution must continue.
    pub fn step_on<T: Write>(&mut self, fd: &mut T) -> Result<bool, MachineError> {
        let adr : u32 = self.registers[IP];
        if adr > 4095 {return Err(MachineError::OutOfMemory);} //check if instruction pointer does not overflow memory
        let inst: u8 = self.memory[adr as usize];
        match inst {
            1 => self.mov_if(adr,4),
            2 => self.store(adr,3),
            3 => self.load(adr,3),
            4 => self.loadimm(adr,4),
            5 => self.sub(adr,4),
            6 => self.out(adr,2, fd),
            7 => self.exit(adr,1),
            8 => self.out_number(adr,2, fd),
            _ => Err(MachineError::InexistantInstruction)          
        }
    }

    /// Check if index of registers does not exceed 15
    /// If this is the case a MachineError is returned
    pub fn check_registers (&mut self, indice: u8) -> Result<(),MachineError> {
        if indice>15 {
            Err(MachineError::InexistantRegister)
        } else {
            Ok(())
        }
    }

    /// Update instruction pointer with set_reg call
    pub fn update_ip(&mut self, adr: u32, inc_adr: u8) -> Result<(),MachineError> {
         
        self.set_reg(IP, adr+inc_adr as u32) 
        
    }

    /// Similar to [step_on](Machine::step_on).
    /// If output instructions are run, they print on standard output.
    pub fn step(&mut self) -> Result<bool, MachineError> {
        self.step_on(&mut io::stdout().lock())
    }

    /// Reference onto the machine current set of registers.
    pub fn regs(&self) -> &[u32] {
        &self.registers
    }

    /// Sets a register to the given value
    /// Returns error if register index out of bounds
    pub fn set_reg(&mut self, reg: usize, value: u32) -> Result<(),MachineError> {
            self.check_registers(reg as u8)?;
            self.registers[reg] = value;
            Ok(())
            
    }

    /// Reference onto the machine current memory.
    /// Returns false if execution was complete or a MachineError
    pub fn memory(&self) -> &[u8] {
        &self.memory
    }

    /// Move value of register B in register A only if register C contains 0
    /// Returns false if execution was complete or a MachineError
    pub fn mov_if(&mut self, adr: u32, inc: u8 ) -> Result<bool,MachineError> {

        self.update_ip(adr,inc)?;         

        let reg_c = self.memory[(adr+3) as usize]; self.check_registers(reg_c)?;

        if self.registers[reg_c as usize] != 0 {
            let reg_b = self.memory[(adr+2) as usize]; self.check_registers(reg_b)?;
            self.set_reg(self.memory[(adr+1) as usize] as usize, self.registers[reg_b as usize])?;
            Ok(false)
        } else {
            Ok(false) 
        }
    }

    /// Store content of register B into memory at register A pointing adress
    /// Returns false if execution was complete or a MachineError
    pub fn store(&mut self, adr: u32, inc: u8) -> Result<bool,MachineError> {

        self.update_ip(adr,inc)?;

        let reg_a = self.memory[(adr+1) as usize]; self.check_registers(reg_a)?;
        let reg_b = self.memory[(adr+2) as usize]; self.check_registers(reg_b)?;
        
        let addr = self.registers[reg_a as usize];

        if addr >= 4093 {return Err(MachineError::OutOfMemory);}

        let val = self.registers[reg_b as usize];
        let mut i = 0;
        for word in val.to_ne_bytes() {
            self.memory[(addr + i) as usize] = word;
            i = i+1;
        }
        Ok(false)
    }

    /// Load memory content pointed by register B in register A
    /// Returns false if execution was complete or a MachineError
    pub fn load(&mut self, adr: u32, inc: u8) -> Result<bool,MachineError> {

        self.update_ip(adr,inc)?;

        let reg_a = self.memory[(adr+1) as usize]; self.check_registers(reg_a)?;
        let reg_b = self.memory[(adr+2) as usize]; self.check_registers(reg_b)?;

        let adr_pointed = self.registers[reg_b as usize];
        if adr_pointed >= 4093 {return Err(MachineError::OutOfMemory);}

        let val = [self.memory[adr_pointed as usize],self.memory[(adr_pointed+1) as usize],self.memory[(adr_pointed+2) as usize],self.memory[(adr_pointed+3) as usize]];
        let concat = u32::from_le_bytes(val);
        self.set_reg(reg_a as usize,concat)?;
        Ok(false)
    }

    /// Load from memory i16 and store extended value into register A
    /// Returns false if execution was complete or a MachineError
    pub fn loadimm(&mut self, adr: u32, inc: u8) -> Result<bool,MachineError> {

        self.update_ip(adr,inc)?;

        let reg_a = self.memory[(adr+1) as usize]; self.check_registers(reg_a)?;
        let l = self.memory[(adr+2) as usize]; 
        let h = self.memory[(adr+3) as usize];

        let val = i16::from_le_bytes([l,h]);
        self.set_reg(reg_a as usize, val as u32)?;
        Ok(false)
    }

    /// Sub content of register B to register C and wrap result in case of overflow
    /// Returns false if execution was complete or a MachineError
    pub fn sub(&mut self, adr:u32, inc:u8) -> Result<bool,MachineError> {


        self.update_ip(adr,inc)?;

        let reg_a = self.memory[(adr+1) as usize]; self.check_registers(reg_a)?;
        let reg_b = self.memory[(adr+2) as usize]; self.check_registers(reg_b)?;
        let reg_c = self.memory[(adr+3) as usize]; self.check_registers(reg_c)?;

        self.set_reg(reg_a as usize,  u32::wrapping_sub(self.registers[reg_b as usize],  self.registers[reg_c as usize] as u32))?;
        Ok(false)
    }

    /// Write unicode character to fd from last byte of register A
    /// Returns false if execution was complete or a MachineError
    pub fn out<T : Write>(&mut self, adr: u32, inc:u8, fd: &mut T) -> Result<bool,MachineError> {

        self.update_ip(adr,inc)?;

        let reg_a = self.memory[(adr+1) as usize]; self.check_registers(reg_a)?;

        let unicode = self.registers[reg_a as usize] as u8 as char;
        let unicode = format!("{unicode}");
        
        if let Err(e) = fd.write(unicode.as_bytes()) {
            return Err(MachineError::IoError(e));
        }
        Ok(false)
    }

    /// Exit program by returning true
    /// Returns false if execution was complete or a MachineError
    pub fn exit(&mut self, adr: u32, inc: u8) -> Result<bool,MachineError> {

        self.update_ip(adr,inc)?;

        Ok(true)
    }

    /// Write in fd value from register A in decimal form
    /// Returns false if execution was complete or a MachineError
    pub fn out_number<T: Write>(&mut self, adr: u32,inc: u8, fd: &mut T) -> Result<bool,MachineError> {

        self.update_ip(adr,inc)?;

        let reg_a = self.memory[(adr+1) as usize]; 

        let val = self.registers[reg_a as usize] as i32;
        let val = format!("{val}");
        if let Err(e) = fd.write(val.as_bytes()) {
            return Err(MachineError::IoError(e));
        }
        Ok(false)
    }
}
