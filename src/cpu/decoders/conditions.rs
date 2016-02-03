use cpu::registers::Condition;

pub struct ConditionDecoder;
impl ConditionDecoder {
    /// Return condition type based on code. Code must be lower than `0b1000`.
    pub fn condition(code: u8) -> Condition {
        assert!(code <= 0b111, " Invalid condition index: {:#b}");
        match code {
            0b000 => Condition::NonZero,
            0b001 => Condition::Zero,
            0b010 => Condition::NonCary,
            0b011 => Condition::Cary,
            0b100 => Condition::ParityOdd,
            0b110 => Condition::ParityEven,
            0b101 => Condition::SignPositive,
            0b111 => Condition::SignNegative,
            _ => unreachable!("Unreachable code!"),
        }
    }
}
