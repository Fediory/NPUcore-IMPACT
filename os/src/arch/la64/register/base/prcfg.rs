use core::fmt::Debug;

use bit_field::BitField;
impl_define_csr!(PRCfg1, "Privileged Resource Configuration 1");
impl_read_csr!(0x21, PRCfg1);

impl Debug for PRCfg1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PRCfg1")
            .field("SAVE reg. number", &self.get_save_num())
            .field("Timer bits", &self.get_timer_bits())
            .field(
                "max vector entry spacing",
                &self.get_max_vector_entry_spacing(),
            )
            .finish()
    }
}
impl PRCfg1 {
    /// The number of SAVE control and status registers.
    pub fn get_save_num(&self) -> usize {
        self.bits.get_bits(0..4)
    }
    /// The number of valid bit width of the timer.
    pub fn get_timer_bits(&self) -> usize {
        // 返回定时器的位数
        self.bits.get_bits(4..12) + 1
    }
    /// The maximum value that can be set for the exception and interrupt vector entry spacing (CSR.ECTL.VS).
    pub fn get_max_vector_entry_spacing(&self) -> usize {
        self.bits.get_bits(12..15)
    }
}
impl_define_csr!(PRCfg2, "Privileged Resource Configuration 1");
impl_read_csr!(0x22, PRCfg2);

impl PRCfg2 {
    /// Return a bit vector of page sizes supported by the TLB.
    pub fn psval(&self) -> usize {
        self.bits
    }
    /// Test whether a page size is supported.
    /// # Argument
    /// * `page_size_log`: `log(page_size)/log(2)`
    pub fn is_supported(&self, page_size_log: usize) -> bool {
        (self.bits & (1 << page_size_log)) != 0
    }
}
impl_define_csr!(PRCfg3, "Privileged Resource Configuration 1");
impl_read_csr!(0x23, PRCfg3);
impl Debug for PRCfg2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PRCfg2")
            .field("FP supported", &self.float_point_support())
            .field("FP version", &self.fp_ver())
            .finish()
    }
}
macro_rules! def_field_rd {
    ($num:literal,$func_name:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $func_name(&self) -> bool {
            self.get_bit($num)
        }
    };
    ($beg:literal,$last:literal,$func_name:ident,$doc:literal) => {
        #[doc = $doc]
        pub fn $func_name(&self) -> usize {
            self.bits.get_bits($beg..=$last)
        }
    };
}
impl PRCfg2 {
    def_field_rd!(
        0,
        float_point_support,
        "True indicates support for basic floating-point instructions"
    );
    def_field_rd!(
        1,
        float_point_single_precision_support,
        "True indicates support for single-precision floating-point numbers"
    );
    def_field_rd!(
        2,
        float_point_double_precision_support,
        "  True indicates support for double-precision floating-point numbers"
    );
    def_field_rd!(
        6,
        lsx_support,
        "1 indicates support for 128-bit vector extension"
    );
    def_field_rd!(
        7,
        lasx_support,
        "1 indicates support for 256-bit vector expansion"
    );
    def_field_rd!(
        8,
        complex_support,
        "True indicates support for complex vector operation instructions"
    );
    def_field_rd!(
        9,
        crypto_support,
        "True indicates support for encryption and decryption vector instructions"
    );
    def_field_rd!(
        10,
        virt_support,
        "True indicates support for virtualization expansion"
    );
    def_field_rd!(13,11,  lvz_ver ,"The version number of the virtualization hardware acceleration specification. 1 is the initial version number");
    def_field_rd!(
        22,
        atomic_support,
        "1 indicates support AM* atomic memory access instruction"
    );
    def_field_rd!(
        21,
        lspw,
        "1 indicates support for the software page table walking instruction"
    );
    def_field_rd!(
        17,
        15,
        fp_ver,
        "
The version number of the floating-point arithmetic standard.
1 is the initial version number to indicate compatibility with the IEEE 754-2008 standard"
    );
    def_field_rd!(
        17,
        15,
        llftp_ver,
        "Constant frequency counter and timer version number. 1 is the initial version "
    );
    def_field_rd!(
        14,
        llftp,
        "1 indicates support for constant frequency counter and timer"
    );
    def_field_rd!(
        18,
        lbt_x86,
        "1 indicates support for X86 binary translation extension"
    );
    def_field_rd!(
        19,
        lbt_arm,
        "1 indicates support for ARM binary translation extension"
    );
    def_field_rd!(
        20,
        lbt_mips,
        "1 indicates support for MIPS binary translation extension"
    );
}
impl PRCfg3 {
    /// 指示 TLB 组织方式：
    /// # Return Values:
    /// * 0：No TLB
    /// * 1：一个全相联的多重页大小 TLB（MTLB）
    /// * 2：一个全相联的多重页大小 TLB（MTLB）+一个组相联的单个页大小 TLB（STLB）；
    /// * Others: Reserved.
    pub fn get_tlb_type(&self) -> usize {
        self.bits.get_bits(0..=3)
    }
    /// 当 TLBType=1 或 2 时，该域的值是全相联多重页大小 TLB 的项数减 1
    pub fn get_mtlb_entries(&self) -> usize {
        self.bits.get_bits(4..=11)
    }

    /// STLBWays
    pub fn get_stlb_ways(&self) -> usize {
        self.bits.get_bits(12..=19) + 1
    }

    /// 当 TLBType=2 时，该域的值是组相联单个页大小 TLB 的每一路项数的幂指数，即每一
    /// 路有 2 ^ STLBSets项。
    pub fn get_sltb_sets(&self) -> usize {
        self.bits.get_bits(20..=25)
    }
}
