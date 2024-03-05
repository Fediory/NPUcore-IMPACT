// 该寄存器记录普通例外处理完毕之后的返回地址。当触发例外时，如果例外类型既不是 TLB 重填例外
// 也不是机器错误例外，则触发例外的指令的 PC 将被记录在该寄存器中
impl_define_csr!(ERA, "Exception Return Address (ERA)\n\
                       Record the resulting PC in case of exceptions other than TLB Refill and Machine Error.");
impl_write_csr!(0x6, ERA);
impl_read_csr!(0x6, ERA);

impl ERA {
    pub fn next_ins(&mut self) -> &mut Self {
        self.bits += 4;
        self
    }
    pub fn set_pc(&mut self, pc: usize) -> &mut Self {
        self.bits = pc;
        self
    }
    pub fn get_pc(&self) -> usize {
        self.bits
    }
}
