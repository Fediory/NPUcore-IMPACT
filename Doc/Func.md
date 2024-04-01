# This markdown file tells the basic call and usage of each function in the main folder (./os)

## os

## arch

### la64

#### board

##### [2k500.rs](os/src/arch/la64/board/2k500.rs)

used call:
HIGH_BASE_EIGHT(const)-[config.rs](os/src/arch/la64/config.rs).
ls_nand(super)-[ls_nand](os/src/arch/la64/ls_nand).

func:
defines NAND type/base/dma_addr/base_addr/block_size to init the starting session.

#### ls_nand

##### [dma.rs](os/src/arch/la64/ls_nand/dma.rs)

used call:.
core(from rust).
DMA_ADDR(const)-[2k500.rs](os/src/arch/la64/board/2k500.rs).
task(mod)-[task](os/src/task).

func:.
defines **const** for DMA to init(10-30).
defines **status** of DMA(74-119).
defines **impl** DMAorder.
defines **impl** DMACmd/DMAOrderAddrLow and their defaults/Debugs.
defines **struct** DMADesc and its Debug.

