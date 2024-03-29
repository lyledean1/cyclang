use llvm_sys::target::{LLVMInitializeWebAssemblyAsmPrinter, LLVMInitializeWebAssemblyTarget};

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Target {
    wasm,
    arm32,
    arm64,
    x86_32,
    x86_64,
}

impl Target {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "wasm" => Some(Target::wasm),
            "arm32" => Some(Target::arm32),
            "arm64" => Some(Target::arm64),
            "x86_32" => Some(Target::x86_32),
            "x86_64" => Some(Target::x86_64),
            _ => None,
        }
    }

    pub fn get_llvm_target_name(&self) -> String {
        match self {
            Target::wasm => "wasm32-unknown-unknown-wasm".to_string(),
            Target::arm32 => "arm-unknown-linux-gnueabihf".to_string(),
            Target::arm64 => "aarch64-unknown-linux-gnu".to_string(),
            Target::x86_32 => "i386-unknown-unknown-elf".to_string(),
            Target::x86_64 => "x86_64-unknown-unknown-elf".to_string(),
        }
    }

    pub fn initialize(&self) {
        unsafe {
            match self {
                Target::wasm => {
                    LLVMInitializeWebAssemblyTarget();
                    LLVMInitializeWebAssemblyAsmPrinter();
                }
                Target::arm32 => {
                    unimplemented!("arm32 not implemented yet ")
                }
                Target::arm64 => {
                    unimplemented!("arm64 not implemented yet ")
                }
                Target::x86_32 => {
                    unimplemented!("x86_32 not implemented yet ")
                }
                Target::x86_64 => {
                    unimplemented!("x86_64 not implemented yet ")
                }
            }
        }
    }
}
