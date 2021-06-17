use crate::sub_system::process::elf::ElfIdent::{EiMag0, EiMag1, EiMag2, EiMag3};

/// half int
type Elf32Half = u16;
/// offset
type Elf32Off = u32;
/// address
type Elf32Addr = u32;
type Elf32Word = u32;
type Elf32Sword = i32;

const ELFNIDENTIFY: usize = 16;

#[repr(C)]
pub struct ELFHeader {
    identify: [u8; ELFNIDENTIFY],
    typ: Elf32Half,
    machine: Elf32Half,
    version: Elf32Word,
    entry: Elf32Addr,
    phoff: Elf32Off,
    shoff: Elf32Off,
    flags: Elf32Word,
    ehsize: Elf32Half,
    phent_size: Elf32Half,
    phnum: Elf32Half,
    shentsize: Elf32Half,
    shnum: Elf32Half,
    shstrndx: Elf32Half,
}

impl ELFHeader {
    fn verify(&self) -> bool {
        return self.identify[EiMag0] == 0x7F
            && self.identify[EiMag1] == 'E'
            && self.identify[EiMag2] == 'L'
            && self.identify[EiMag3] == 'F'
    }
}

enum ElfIdent {
	EiMag0 = 0, // 0x7F
    EiMag1 = 1, // 'E'
    EiMag2 = 2, // 'L'
    EiMag3 = 3, // 'F'
    EiClass = 4, // Architecture (32/64)
    EiData = 5, // Byte Order
    EiVersion = 6, // ELF Version
    EiOsabi = 7, // OS Specific
    EiAbiversion = 8, // OS Specific
    EiPad = 9  // Padding
}

enum ElfType {
	EtNone = 0, // Unknown Type
    EtRel = 1, // Relocatable File
    EtExec = 2  // Executable File
}
