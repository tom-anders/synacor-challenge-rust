#[derive(Debug, Clone)]
struct Vm {
    memory: [u16; 2_usize.pow(15)],
    registers: [u16; 8],
    stack: Vec<u16>,
}
