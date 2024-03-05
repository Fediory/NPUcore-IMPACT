impl_define_csr!(MErrInfo1,"Machine Error Information 1\n\
                            When a machine error exception is triggered, \n\
                            the hardware will store more information related to that error into these two registers for system software diagnostic purposes.\n\
                            The specific format is implementation-dependent.");
impl_write_csr!(0x91, MErrInfo1);
impl_read_csr!(0x91, MErrInfo1);
impl_define_csr!(MErrInfo2,"Machine Error Information 1\n\
                            When a machine error exception is triggered, \n\
                            the hardware will store more information related to that error into these two registers for system software diagnostic purposes.\n\
                            The specific format is implementation-dependent.");
impl_write_csr!(0x92, MErrInfo2);
impl_read_csr!(0x92, MErrInfo2);
