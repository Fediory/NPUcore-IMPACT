# This markdown file tells the basic call and usage of each function in the main folder (./os)

## os

## arch

### la64

#### board

##### [2k500.rs](os/src/arch/la64/board/2k500.rs)

used call:
HIGH_BASE_EIGHT **const** -[config.rs](os/src/arch/la64/config.rs)

ls_nand **super** - [ls_nand](os/src/arch/la64/ls_nand)

func:
defines NAND type/base/dma_addr/base_addr/block_size to init the starting session

#### ls_nand

##### [dma.rs](os/src/arch/la64/ls_nand/dma.rs)

used call:
core(from rust)

DMA_ADDR **const** - [2k500.rs](os/src/arch/la64/board/2k500.rs)

task **mod** - [task](os/src/task)

func:
defines **const** for DMA to init(10-30)

defines **status** of DMA(74-119)

defines **impl** DMAorder

defines **impl** DMACmd/DMAOrderAddrLow and their defaults/Debugs

defines **struct** DMADesc and its Debug

##### [mmio_macro.rs](os/src/arch/la64/ls_nand/mmio_macro.rs)

func:
defines the macro of memories' option and its registers

##### [mod.rs](os/src/arch/la64/ls_nand/mod.rs)

used call:
spin(from rust::std)

DMAOrder **impl** - [dma.rs](os/src/arch/la64/ls_nand/dma.rs)

board **super** - [board](os/src/arch/la64/board) *contains the const of DMA*

dma **mod** - [dma.rs](os/src/arch/la64/ls_nand/dma.rs)

nand **mod** - [nand.rs](os/src/arch/la64/ls_nand/nand.rs)

func:
defines **struct** LoongsonNand and its **impl** LoongsonNand

defines **impl** BlockDevice for LoongsonNand

#### register

##### base

###### [badi.rs](os/src/arch/la64/register/base/badi.rs)

defines **impl** the Bad Instruction register which *used to record the instruction code of the instruction that triggers the synchronous-related exception*

###### [badv.rs](os/src/arch/la64/register/base/badv.rs)

defines **impl** Bad Virtual Address

flags below:

- ADdress error Exception for Fetching instructions (ADEF), at this time the PC of the instruction is recorded
- ADdress error Exception for Memory access instructions (ADEM)
- Address aLignment fault Exception (ALE)
- Bound Check Exception (BCE)
- Page Invalid exception for Load operation (PIL)
- Page Invalid exception for Store operation (PIS)
- Page Invalid exception for Fetch operation (PIF)
- Page Modification Exception (PME)
- Page Non-Readable exception (PNR)
- Page Non-eXecutable exception (PNX)
- Page Privilege level Illegal exception (PPI)

###### [cpuid.rs](os/src/arch/la64/register/base/cpuid.rs)

defines **impl** of register contains the processor core number information

###### [crmd.rs](os/src/arch/la64/register/base/crmd.rs)

defines **impl** CrMd which *determine the the processor core’s privilege level,global interrupt enable bit, watchpoint enable bit, and address translation mode at that time*

used call:
MemoryAccessType **crate** - [register](os/src/arch/la64/register)

func:
defines **fn** for CrMd below:

bits/set_bits:changes bit's value

get_plv:returns the privilege level *from0~2*

set_plv:call set_bits to set prv_level

set_ie:Set the interrupt enabling status to 'status'

is_interrupt_enabled:returns 2 if the machine is globally interrupt enabled

is_paging:returns 1 if paging mode on

set_paging:set the paging mode

-*below are seting memo_acc_type*

get_datf/set_datf:Set memory access type in direct access mode

get_datm/set_datm:Set Memory Access Type (MAT) for load and store operations when in direct address translation mode.

is_watchpoint_enabled/set_watchpoint_enabled:Instruction and data watchpoints enable bit, which is active high.

da/pg:modifying the 3rd & 4th bit

---

now starting making code_tree about mm(*memory_managements*) and syscall(*system_call*) first while reconstructing

here gives a method of reconstructing(*temp*):

1. by moving each **struct** and **impl** together to reform its libs

2. I haven't find the triggers and calls of each lib file, so I just ignore them

>marked at 4.1 - starting reconstruct memo-management

## mm

contructure as below

- main
    - memory allocation
        - page -> frame -> single unit
    - using heap as container

### [address.rs](os\src\mm\address.rs)

this .rs contains defination and function of address(including *Physicaladdr&Virtualaddr*),now reconstucted into [/address](os\src\mm\mm_reconstructed\address).

here's the list of **func**,**struct**,**impl** contains in the addr(which are the options toward address)

- physical addr.
    - **structs**
        - PhysAddr (the structure of physical address)
        - PhysPageNum 
    - **impl**
        - From\<usize\> (getting its usize)
        - From\<PhysPageNum\> (getting its Page Num by lefting its addr.)
        - PhysAddr (options of Physical Address)
            - get_ref ***unsafe!***(unwraping itself into ref?)
            - get_mut ***unsafe!***(unwraping itself into mut_var)
            - get_bytes_ref ***unsafe!***
            - get_bytes_mut ***unsafe!***
        - PhysPage
            - start_addr (having the start addr. of the page)
            - offset
            - get_bytes_array
            - get_dwords_array
            - get_mut
- virtual addr. (*familiar to physical address,**only** differences are given below*)
    - **structs**
        - VirtAddr
        - VirtPageNum
    - **impl** (***only differences are given***)  
        - from [VirtAddr.rs](os\src\mm\mm_reconstructed\address\Virt\VirtAddr.rs)   
            - floor
            - ceil  
            - page_offset
            - aligned
        - from [VirtPageNum.rs](os\src\mm\mm_reconstructed\address\Virt\VirtPageNum.rs)
            - indexes

*basic functions* - giving the basic method of processing address, include calculating offset/page index/etc.






