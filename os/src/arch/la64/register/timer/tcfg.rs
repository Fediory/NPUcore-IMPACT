use core::fmt::Debug;

use bit_field::BitField;
impl_define_csr!(TCfg, "Timer Configuration\n\
                        This register is the interface to the software configuration timer.\n\
                        The number of valid bits of the timer is determined by the implementation,\n\
                        so the length of the TimeVal field in this register will change accordingly.");
impl_read_csr!(0x41, TCfg);
impl_write_csr!(0x41, TCfg);
impl Debug for TCfg {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TCfg")
            .field("is_enabled", &self.is_enabled())
            .field("is_periodic", &self.is_periodic())
            .field("InitVal of (dec) timer", &self.get_init_val())
            .finish()
    }
}
impl TCfg {
    /// Timer enable bit.
    /// Only when this bit is 1,
    /// the timer will perform countdown self decrement and set up the timing interrupt signal when it decrements to 0 value.
    pub fn is_enabled(&self) -> bool {
        //第0位
        !self.bits.get_bit(0)
    }
    /// Timer cycle mode control bit.
    /// If this bit is 1, when the timer decreases to 0,
    /// the timer will set up the timer interrupt signal and reload the timer to the initial value configured in the TimeVal field,
    /// and then continue to decrement itself in the next clock cycle.
    /// If this bit is 0, the timer will stop counting until the software configures the timer again when the countdown reaches 0.
    pub fn is_periodic(&self) -> bool {
        //第1位
        self.bits.get_bit(1)
    }
    /// The initial value of the timer countdown self decrement count.
    /// This initial value must be an integer multiple of 4.
    /// The hardware will automatically fill in the lowest bit of the field value.
    /// Two bits of 0 are added before it is used.
    pub fn get_init_val(&self) -> usize {
        //第2位开始
        (self.bits >> 2) << 2
    }
    pub fn bits(&self) -> usize {
        self.bits
    }
    pub fn set_bits(&mut self, val: usize) -> &mut Self {
        self.bits = val;
        self
    }
    /// Only when this bit is 1,
    /// the timer will perform countdown self decrement and set up the timing interrupt signal when it decrements to 0 value.
    pub fn set_enable(&mut self, enable: bool) -> &mut Self {
        self.bits.set_bit(0, enable);
        self
    }
    /// If this bit is 1, when the timer decreases to 0,
    /// the timer will set up the timer interrupt signal and reload the timer to the initial value configured in the TimeVal field,
    /// and then continue to decrement itself in the next clock cycle.
    /// If this bit is 0, the timer will stop counting until the software configures the timer again when the countdown reaches 0.
    pub fn set_periodic(&mut self, loop_: bool) -> &mut Self {
        self.bits.set_bit(1, loop_);
        self
    }
    /// Set the initial value of the timer countdown self decrement count.
    /// The hardware will automatically fill in the lowest bit of the field value.
    /// Two bits of 0 are added before it is used.
    /// # Warning!
    /// This initial value *MUST* be an integer multiple of 4.
    pub fn set_init_val(&mut self, val: usize) -> &mut Self {
        // 设置计数值, 只能是4的整数倍
        // 在数值末尾会补上2bit0
        self.bits.set_bits(2.., val >> 2);
        self
    }
}
