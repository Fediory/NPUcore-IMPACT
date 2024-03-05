impl_define_csr!(BadV, "Bad Virtual Address (BADV)
This register is used to record the bad address when a bad address exception is triggered. Such exceptions include:
* ADdress error Exception for Fetching instructions (ADEF), at this time the PC of the instruction is recorded
* ADdress error Exception for Memory access instructions (ADEM)
* Address aLignment fault Exception (ALE)
* Bound Check Exception (BCE)
* Page Invalid exception for Load operation (PIL)
* Page Invalid exception for Store operation (PIS)
* Page Invalid exception for Fetch operation (PIF)
* Page Modification Exception (PME)
* Page Non-Readable exception (PNR)
* Page Non-eXecutable exception (PNX)
* Page Privilege level Illegal exception (PPI)");
impl_write_csr!(0x7, BadV);
impl_read_csr!(0x7, BadV);

impl BadV {
    pub fn get_vaddr(&self) -> usize {
        self.bits
    }
}
