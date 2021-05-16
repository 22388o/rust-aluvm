use std::cmp::Ordering;

#[non_exhaustive]
pub enum Instruction {
    #[value = 0b00_000_000]
    ControlFlow(ControlFlowOp),

    #[value = 0b00_001_000]
    Register(RegisterOp),

    #[value = 0b00_010_000]
    Cmp(CmpOp),


    #[value = 0b00_100_000]
    Arithmetic(ArithmeticOp),

    #[value = 0b00_101_000]
    Bitwise(BitwiseOp),


    #[value = 0b01_000_000]
    Digest(DigestOp),

    #[value = 0b01_001_000]
    Secp256k1(SecpOp),

    #[value = 0b01_001_100]
    Ed25519(Ed25519Op),
}

pub enum ControlFlowOp {
    /// Completes program execution writing `false` to `st0` (indicating program failure)
    #[value = 0b000]
    Fail,

    /// Completes program execution writing `true` to `st0` (indicating program success)
    #[value = 0b001]
    Succ,

    /// Unconditionally jumps to an offset. Increments `cy0`.
    #[value = 0b010]
    Jmp(u16),

    /// Jumps to an offset if `st0` == true, otherwise does nothing. Increments `cy0`.
    #[value = 0b011]
    Jif(u16),

    /// Jumps to other location in the current code with ability to return
    /// back (calls a subroutine). Increments `cy0` and pushes offset of the
    /// instruction which follows current one to `cs0`.
    Routine(u16),

    /// Calls code from an external library identified by the hash of its code.
    /// Increments `cy0` and `cp0` and pushes offset of the instruction which
    /// follows current one to `cs0`.
    Call([u8; 32], u16),

    /// Passes execution to other library without an option to return.
    /// Does not increments `cy0` and `cp0` counters and does not add anything
    /// to the call stack `cs0`.
    Exec([u8; 32], u16),

    /// Returns execution flow to the previous location from the top of `cs0`.
    /// Does not change value in `cy0`. Decrements `cp0`.
    Ret,
}

pub enum RegisterOp {
    /// Swap operation. If the value does not fit destination bit dimensions
    /// truncates the most significant bits until they fit.
    Swp(Reg, Reg32, Reg, Reg32, bool /** Fill extra bits with highest bit for first value */, bool /** Fill extra bits with highest bit for second value */),
    /// Duplicates value from low 16 registers to high 16 registers
    Mov(Reg, Reg32, Reg, Reg32, bool /** Duplicate or move */, bool /** Fill extra bits with highest bit */),

    /// Sets register value to zero
    Zeroa(RegA, Reg32),
    Zeror(RegR, Reg32),

    /// Cleans a value of a register (sets it to undefined state)
    Cleana(RegA, Reg32),
    Cleanr(RegR, Reg32),

    Puta(RegA, Reg32, u16, Box<[u8]>),
    Putr(RegR, Reg32, u16, Box<[u8]>),
}

pub enum CmpOp {
    /// Compares value of two arithmetic (`A`) registers putting result into `cm0`
    #[value = 0b110] // 3 + 5 + 3 + 5 => 16 bits
    Cmpa(RegA, Reg32, RegA, Reg32),

    /// Compares value of two non-arithmetic (`R`) registers putting result into `cm0`
    #[value = 0b111]
    Cmpr(RegR, Reg32, RegR, Reg32),

    /// Checks equality of value in two arithmetic (`A`) registers putting result into `st0`
    #[value = 0b100]
    Eqa(RegA, Reg32, RegA, Reg32),

    /// Checks equality of value in two non-arithmetic (`R`) registers putting result into `st0`
    #[value = 0b101]
    Eqr(RegR, Reg32, RegR, Reg32),

    /// Measures bit length of a value in one fo the registers putting result to `a16[0]`
    Lena(RegA, Reg32, Reg32),
    Lenr(RegA, Reg32, Reg32),

    /// Counts number of `1` bits in register putting result to `a16[0]` register
    Cnta(RegA, Reg32, Reg32),
    Cntr(RegR, Reg32, Reg32),
}

pub enum ArithmeticOp {
    Neg(RegA, Reg32), // 3 + 5 = 8 bits
    Add(Arithmetics, RegA, Reg32, Reg32), // 3 + 3 + 5 + 5  => 16 bits
    Sub(Arithmetics, RegA, Reg32, Reg32),
    Mul(Arithmetics, RegA, Reg32, Reg32),
    Div(Arithmetics, RegA, Reg32, Reg32),
    Mod(RegA, Reg32), // 3 + 5 = 8 bits
    Abs(RegA, Reg32, RegA, Reg32), // 3 + 5 + 3 + 5 => 16 bits
}

pub enum BitwiseOp {
    And(RegA, Reg32, Reg32, Reg8 /** Operation destination, only first 8 registers */),
    Or(RegA, Reg32, Reg32, Reg8),
    Xor(RegA, Reg32, Reg32, Reg8),

    Not(RegA, Reg32),

    Shl(RegA, Reg32, Reg32 /** Always `a8` */, Reg8),
    Shr(RegA, Reg32, Reg32, Reg8),
    /// Shift-cycle left
    Scl(RegA, Reg32, Reg32, Reg8),
    /// Shift-cycle right
    Scr(RegA, Reg32, Reg32, Reg8),
}

#[non_exhaustive]
pub enum DigestOp {
    Ripemd(
        Reg32 /** Which of `a16` registers contain start offset */,
        Reg32 /** Index of string register */,
        Reg32 /** Index of `r160` register to save result to */,
        bool /** Clear string register after operation */
    ),
    Sha2(
        Reg32 /** Which of `a16` registers contain start offset */,
        Reg32 /** Index of string register */,
        Reg32 /** Index of `r160` register to save result to */,
        bool /** Clear string register after operation */
    ),
}

pub enum SecpOp {
    Gen(
        Reg32 /** Register containing scalar */,
        Reg8 /** Destination register to put G * scalar */
    ),
    Mul(
        bool /** Use `a` or `r` register as scalar source */,
        Reg32 /** Scalar register index */,
        Reg32 /** Source `r` register index containing EC point */,
        Reg32 /** Destination `r` register index */,
    ),
    Add(
        bool /** Allow overflows */,
        Reg32 /** Source 1 */,
        Reg32 /** Source 2 */,
        Reg32 /** Source 3 */,
    ),
    Neg(
        Reg32 /** Register hilding EC point to negate */,
        Reg8 /** Destination register */,
    ),
}

pub enum Ed25519Op {
    Gen(
        Reg32 /** Register containing scalar */,
        Reg8 /** Destination register to put G * scalar */
    ),
    Mul(
        bool /** Use `a` or `r` register as scalar source */,
        Reg32 /** Scalar register index */,
        Reg32 /** Source `r` register index containing EC point */,
        Reg32 /** Destination `r` register index */,
    ),
    Add(
        bool /** Allow overflows */,
        Reg32 /** Source 1 */,
        Reg32 /** Source 2 */,
        Reg32 /** Source 3 */,
    ),
    Neg(
        Reg32 /** Register hilding EC point to negate */,
        Reg8 /** Destination register */,
    ),
}

#[derive(Debug, Display)]
#[display(Debug)]
pub enum Reg32 {
    Reg1,
    Reg2,
    Reg3,
    Reg4,
    Reg5,
    Reg6,
    Reg7,
    Reg8,
    Reg9,
    Reg10,
    Reg11,
    Reg12,
    Reg13,
    Reg14,
    Reg15,
    Reg16,
    Reg17,
    Reg18,
    Reg19,
    Reg20,
    Reg21,
    Reg22,
    Reg23,
    Reg24,
    Reg25,
    Reg26,
    Reg27,
    Reg28,
    Reg29,
    Reg30,
    Reg31,
    Reg32,
}

#[derive(Debug, Display)]
#[display(Debug)]
pub enum Reg8 {
    Reg1,
    Reg2,
    Reg3,
    Reg4,
    Reg5,
    Reg6,
    Reg7,
    Reg8,
}

#[derive(Debug, Display)]
#[display(Debug)]
pub enum RegA {
    AP,
    A8,
    A16,
    A32,
    A64,
    A128,
    A256,
    A512,
}

#[derive(Debug, Display)]
#[display(Debug)]
pub enum RegR {
    R128,
    R160,
    R256,
    R512,
    R1024,
    R2048,
    R4096,
    R8192,
}

#[derive(Debug, Display)]
#[display(Debug)]
pub enum Reg {
    A(RegA),
    R(RegR),
}

pub enum Arithmetics {
    IntChecked(bool),
    IntUnchecked(bool),
    IntArbitraryPrecision(bool),
    Float,
    FloatArbitraryPrecision,
}

#[derive(Debug)]
struct Registers {
    // Arithmetic registers:
    a8: [Option<u8>; 32],
    a16: [Option<u16>; 32],
    a32: [Option<u32>; 32],
    a64: [Option<u64>; 32],
    a128: [Option<u128>; 32],
    a256: [Option<u256>; 32],
    a512: [Option<u512>; 32],

    /// Arbitrary-precision arithmetics registers
    ap: [Option<Box<[u8]>>; 32],

    // Non-arithmetic registers:
    r128: [Option<[u8; 16]>; 32],
    r160: [Option<[u8; 20]>; 32],
    r256: [Option<[u8; 32]>; 32],
    r512: [Option<[u8; 64]>; 32],
    r1024: [Option<[u8; 128]>; 32],
    r2048: [Option<[u8; 256]>; 32],
    r4096: [Option<[u8; 512]>; 32],
    r8192: [Option<[u8; 1024]>; 32],

    /// String and bytestring registers
    s16: [Option<[u8; u16::MAX as usize]>; 32],

    /// Control flow register which stores result of comparison operations. Initialized with `0`
    cm0: Ordering,

    /// Control flow register which stores result of equality and other types of boolean checks. Initialized with `true`
    st0: bool,

    /// Counts number of jumps (possible cycles). The number of jumps is limited by 2^16 per script.
    cy0: u16,

    /// Call stack. Maximal size is `u16::MAX` (limited by `cy0` mechanics and `cp0`)
    cs0: [(Option<[u8; 32]>, u16); u16::MAX as usize],

    /// Defines "top" of the call stack
    cp0: u16,
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            st0: true,
            cm0: Ordering::Equal,
            ..Default::default()
        }
    }
}

impl Registers {
    pub fn execute(&mut self, code: &[u8]) {

    }
}