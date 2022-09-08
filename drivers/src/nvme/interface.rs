use alloc::string::String;
use alloc::vec::Vec;
use core::mem::size_of;
use core::ptr::{read_volatile, write_volatile};

use crate::scheme::{BlockScheme, Scheme};
use crate::DeviceResult;

use super::dev::*;
use super::driver::*;
use super::queue::*;


pub struct NvmeInterface {
    name: String,

    dev: NvmeDev,

    driver: NvmeDriver,

    irq: usize,
}

impl NvmeInterface{

    pub fn new(irq: usize, bar:usize, len:usize) -> DeviceResult<NvmeInterface>{

        let dev = NvmeDev::new(bar);
        let driver = NvmeDriver::new();

        let mut interface = NvmeInterface{
            name: String::from("nvme"),
            dev,
            driver,
            irq,
        };

        interface.init(bar,len)?;

        Ok(interface)

    }


    // 第一，设置映射设备的bar空间到内核的虚拟地址空间当中，通过调用ioremap函数，将Controller的nvme寄存器映射到内核后，可以通过writel, readl这类函数直接读写寄存器。
    // 第二, 完成 DMA mask设置、pci总线中断分配、读取并配置 queue depth、stride 等参数
    // 第三，设置admin queue，admin queue设置之后，才能发送nvme admin Command。
    // 第四，添加nvme namespace设备，即/dev/nvme#n#，这样就可以对设备进行读写操作了。
    // 第五，添加nvme Controller设备，即/dev/nvme#，提供ioctl接口。这样userspace就可以通过ioctl系统调用发送nvme admin command。

    // 参考 linux 5.19  nvme_reset_work    nvme_pci_configure_admin_queue
    pub fn init(&mut self, bar: usize, len:usize) -> DeviceResult {
        
        //第一步在pci扫描到设备时已经完成

        //第二步 设置admin queue,包括其需要的CQ和SQ空间和DMA地址
        let nvme: Nvme<ProviderImpl> = super::Nvme::new(bar, len);


        // let q_db = dbs[qid * 2 * db_stride]


        // admin queue 队列深度 32
        // aqa寄存器高16bit存储cq深度，低16bit存储sq深度
        let aqa_low_16 = 32 as u16;
        let aqa_high_16 = 32 as u16;
        let aqa = (aqa_high_16 as u32) << 16 | aqa_low_16 as u32;
        let aqa_address = bar + NVME_REG_AQA;

        // 将admin queue配置信息写入nvme设备寄存器AQA, admin_queue_attributes
        unsafe{
            write_volatile(aqa_address as *mut u32, aqa);
        }

        // 将admin queue的sq dma物理地址写入nvme设备上的寄存器ASQ
        let sq_dma_pa = nvme.sq_dma_pa as u32;
        let asq_address = bar + NVME_REG_ASQ ;
        unsafe{
            write_volatile(asq_address as *mut u32, sq_dma_pa);
        }

        // 将admin queue的cq dma物理地址写入nvme设备上的寄存器ACQ
        let cq_dma_pa = nvme.cq_dma_pa as u32;
        let acq_address = bar + NVME_REG_ACQ;
        unsafe{
            write_volatile(acq_address as *mut u32, cq_dma_pa);
        }


        //&'static mut [Volatile<u32>]
        let dev_dbs = (bar + NVME_REG_DBS) as u32 as *mut u32;

    
        //db记录了sq和cq的头和尾指针，高16bit存储sq头指针，低16bit存储cq头指针

        /*
        Doorbell  Stride  (DSTRD):  Each  Submission  Queue  and  Completion  Queue  
        Doorbell  register  is  32-bits  in  size
        This  register  indicates  the  stride  between  
        doorbell registers. The stride is specified as (2 ^ (2 + DSTRD)) in bytes. A value 
        of 0h indicates a stride of 4 bytes, where the doorbell registers are packed without 
        reserved space between each register. 
        */
        

        // tell the doorbell register tail = 2
        // 写入了2个命令
        //至此 admin queue初始化完毕
        let admin_q_db = dev_dbs;
        unsafe{
            write_volatile(admin_q_db, 2)
        }


        //io queue db = dev_dbs[qid * 2 * dev->db_stride]
            
        // {BAR0，BAR1}+1000+Doorbell（寄存器内部偏移）={0XD1100000}+1000h+(2y * (4 <<CAP.DSTRD)
        // =0XD1101000+(2*4* (4 <<0)= 0XD1101000+32=0XD1101000+20h=0XD1101020


        //设置admin sq cq的参数, doorbell register寄存器地址等信息
        //设置head tail q_db 

        // let q_db = dbs[qid * 2 * db_stride];

        // let dev_cap_addr = (bar as u64 + NVME_REG_CAP as u64)   as *const u64;
        
        // let dev_cap = unsafe { read_volatile(dev_cap_addr) };
        
        // let db_stride = 1 << (dev_cap >> 32 & 0xfff);
        
        // let dev_dbs = bar + 4096;
        
        // let cap = dev_cap;
        // let q_head = 0;
        // let q_phase = 1;

        
        // self.nvme_alloc_queue(1, 32);
        // self.nvme_init_queue(queue_id, q_depth);
        

        // 分配一个nvme queue，包括其需要的CQ和SQ空间和DMA地址,注意这里第一个io queue使用的entry是0,也就是和admin queue共用
        // self.nvme_alloc_queue(0, 32);
        // /*
        // #define NVME_CAP_STRIDE(cap)	(((cap) >> 32) & 0xf)
        // 1 << NVME_CAP_STRIDE(dev->ctrl.cap);
        // dev->ctrl.cap = lo_hi_readq(dev->bar + NVME_REG_CAP);
        // cap = (dev->bar >> 32) & 0xf;
        // */
        // let db_bar_size = NvmeRegister::NvmeRegDbs as usize + (num_queues * 8 * db_stride);

        
        Ok(())
    }
}









impl Scheme for NvmeInterface {
    fn name(&self) -> &str {
        "nvme"
    }

    fn handle_irq(&self, irq: usize) {
        if irq != self.irq {
            // not ours, skip it
            return;
        }
        let data = self.driver.handle_interrupt();

    }
}


/*
 * Maximum Data Transfer Size (MDTS) field indicates the maximum
 * data transfer size between the host and the controller. The
 * host should not submit a command that exceeds this transfer
 * size. The value is in units of the minimum memory page size
 * and is reported as a power of two (2^n).
 *
 * The spec also says: a value of 0h indicates no restrictions
 * on transfer size. But in nvme_blk_read/write() below we have
 * the following algorithm for maximum number of logic blocks
 * per transfer:
 *
 * u16 lbas = 1 << (dev->max_transfer_shift - ns->lba_shift);
 *
 * In order for lbas not to overflow, the maximum number is 15
 * which means dev->max_transfer_shift = 15 + 9 (ns->lba_shift).
 * Let's use 20 which provides 1MB size.
 */

// dev->max_transfer_shift = 20;



impl BlockScheme for NvmeInterface {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> DeviceResult {

        assert_eq!(buf.len(), 512);

        //一次只读一块 512B
        let total_len = 512;
        let blkcnt = 1;
        let mut c = NvmeRWCommand::new_read_command();

        /* 
        每个NVMe命令中有两个域：PRP1和PRP2，Host就是通过这两个域告诉SSD数据在内存中的位置或者数据需要写入的地址
        首先对prp1进行读写，如果数据还没完，就看数据量是不是在一个page内，在的话，只需要读写prp2内存地址就可以了，数据量大于1个page，就需要读出prp list

        由于只读一块, 小于一页, 所以只需要prp1
        prp1 = dma_addr 
        prp2 = 0

        uboot中对应实现 nvme_setup_prps
        linux中对应实现 nvme_pci_setup_prps
        */
        let dma_addr = 0;
        let prp1 = dma_addr;
        let prp2 : u64 = 0;
        
        c.prp1 = prp1;
        c.prp2 = prp2;
        c.slba = blkcnt;

        // 把命令写入io queue的sq中
        // self.dev.io_queues[0].sq.write(c);

        // 修改door bell register, 通知SSD有新的命令
        
        // tail = xx
        // self.dev.io_queues[0].sq_db.write(1);

        // submit_queue[0].write(c);

        // c.slba = 0;
        // c.length = lbas -1;






        Ok(())
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) -> DeviceResult {


        assert_eq!(buf.len(), 512);

        //一次只读一块 512B
        let total_len = 512;
        let blkcnt = 1;
        let mut c = NvmeCommand::NvmeRWCommand;

        /* 
        每个NVMe命令中有两个域：PRP1和PRP2，Host就是通过这两个域告诉SSD数据在内存中的位置或者数据需要写入的地址
        首先对prp1进行读写，如果数据还没完，就看数据量是不是在一个page内，在的话，只需要读写prp2内存地址就可以了，数据量大于1个page，就需要读出prp list

        由于只读一块, 小于一页, 所以只需要prp1
        prp1 = dma_addr 
        prp2 = 0

        uboot中对应实现 nvme_setup_prps
        linux中对应实现 nvme_pci_setup_prps
        */
        let dma_addr = 0;
        let prp1 = dma_addr;
        let prp2 : u64 = 0;
        let src_ptr = buf.as_ptr() as u64;

        
        
        //riscv是小端模式, 故这里不做转换
        // c.slba = block_id as u64;
        // c.length = 1;
        // c.prp1 = src_ptr;
        // c.prp2 = prp2;

        // self.nvme_submit_sync_cmd(c);



        Ok(())
    }

    fn flush(&self) -> DeviceResult {
        Ok(())
    }
}



impl NvmeInterface {
    pub fn nvme_alloc_queue(&self, queue_id: usize, q_depth: usize) -> DeviceResult {
        Ok(())
    }


    pub fn nvme_init_queue(&self, queue_id: usize, q_depth: usize) -> DeviceResult {
        Ok(())
    }




    pub fn nvme_submit_sync_cmd(&mut self, cmd: NvmeCommand) -> DeviceResult {
        // match cmd {
        //     NvmeCommand::NvmeRWCommand => {

        //     }

        //     NvmeCommand::NvmeCreateCq => {

        //     }

        //     NvmeCommand::NvmeCreateSq => {

        //     }
            
        //     _ => {
        //         info!("wrong command");
        //     }
            
        // }
        // let io_queue = &mut self.dev.io_queues[0];

        // // copy a command into a queue and ring the doorbell
        // self.nvme_submit_cmd(io_queue, cmd);

        // // wait for the command to complete
        // self.nvme_read_completion_status(io_queue);
        Ok(())
    }


    pub fn nvme_read_completion_status(&mut self, nvmeq: &mut NvmeQueue) -> Option<usize>{

        Some(0)
    }

    pub fn nvme_submit_cmd(&mut self, nvmeq: &mut NvmeQueue, cmd:NvmeCommand){


        let cmdsize = size_of::<NvmeRWCommand>();

        nvmeq.sq_push(cmd);


        // ring the doorbell

    }
}



#[derive(Copy, Clone, Debug)]
pub enum NvmeCommand {
    NvmeRWCommand,
    NvmeCreateSq,
	NvmeCreateCq,
}

// impl NvmeCommand{
//     pub fn from_create_sq(x: NvmeCreateSq) -> NvmeCommand {
//         x
//     }

//     // pub fn to_create_sq(&self) -> NvmeCreateSq{
//     //     match self {
//     //         NvmeCommand::NvmeCreateSq => ,
//     //         _ => None,
//     //     }
//     // }

// }

// 1+1+2+4*5+8+8+2+2+2+2+4*4 = 64B
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NvmeCreateCq{
    pub opcode: u8,
    pub flags: u8,
    pub command_id: u16,
    pub rsvd1: [u32;5],
    pub prp1: u64,
    pub rsvd8: u64,
    pub sqid: u16,
    pub qsize: u16,
    pub cq_flags: u16,
    pub irq_vector: u16,
    pub rsvd12: [u32;4],
}

impl NvmeCreateCq{
    pub fn new_create_cq_command() -> Self{
        NvmeCreateCq{
            opcode: 0x04,
            flags: 0,
            command_id: 0,
            rsvd1: [0 as u32; 5],
            prp1: 0,
            rsvd8: 0,
            sqid: 0,
            qsize: 0,
            cq_flags: 0,
            irq_vector: 0,
            rsvd12: [0 as u32; 4],
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NvmeCreateSq{
    pub opcode: u8,
    pub flags: u8,
    pub command_id: u16,
    pub rsvd1: [u32;5],
    pub prp1: u64,
    pub rsvd8: u64,
    pub sqid: u16,
    pub qsize: u16,
    pub sq_flags: u16,
    pub cqid: u16,
    pub rsvd12: [u32;4],
}
impl NvmeCreateSq{
    pub fn new_create_sq_command() -> Self{
        NvmeCreateSq{
            opcode: 0x05,
            flags: 0,
            command_id: 0,
            rsvd1: [0 as u32; 5],
            prp1: 0,
            rsvd8: 0,
            sqid: 0,
            qsize: 0,
            sq_flags: 0,
            cqid: 0,
            rsvd12: [0 as u32; 4],
        }
    }
}

pub struct NvmeRWCommand {
    pub opcode: u8,
    pub flags: u8,
    pub command_id: u16,
    pub nsid: u32,
    pub rsvd2: u64,
    pub metadata: u64,
    pub prp1: u64,
    pub prp2: u64,
    pub slba: u64,
    pub length: u16,
    pub control: u16,
    pub dsmgmt: u32,
    pub reftag: u32,
    pub apptag: u16,
    pub appmask: u16,
}

impl NvmeRWCommand{
    pub fn new_read_command() -> Self{
        Self{
            opcode: 0x02,
            flags: 0,
            command_id: 0,
            nsid: 0,
            rsvd2: 0,
            metadata: 0,
            prp1: 0,
            prp2: 0,
            slba: 0,
            length: 0,
            control: 0,
            dsmgmt: 0,
            reftag: 0,
            apptag: 0,
            appmask: 0,
        }
    }

    pub fn new_write_command() -> Self{
        Self{
            opcode: 0x01,
            flags: 0,
            command_id: 0,
            nsid: 0,
            rsvd2: 0,
            metadata: 0,
            prp1: 0,
            prp2: 0,
            slba: 0,
            length: 0,
            control: 0,
            dsmgmt: 0,
            reftag: 0,
            apptag: 0,
            appmask: 0,
        }
    }
}


#[deny(non_camel_case_types)]
pub enum NvmeRegister{
	NvmeRegCap	= 0x0000,	/* Controller Capabilities */
    NvmeRegVs	= 0x0008,	/* Version */
    NvmeRegIntms	= 0x000c,	/* Interrupt Mask Set */
    NvmeRegIntmc	= 0x0010,	/* Interrupt Mask Clear */
    NvmeRegCc	= 0x0014,	/* Controller Configuration */
    NvmeRegCsts	= 0x001c,	/* Controller Status */
    NvmeRegNssr	= 0x0020,	/* NVM Subsystem Reset */
    NvmeRegAqa	= 0x0024,	/* Admin Queue Attributes */
    NvmeRegAsq	= 0x0028,	/* Admin Submission Queue Base Address */
    NvmeRegAcq	= 0x0030,	/* Admin Completion Queue Base Address */
    NvmeRegCmbloc	= 0x0038,	/* Controller Memory Buffer Location */
    NvmeRegCmbsz	= 0x003c,	/* Controller Memory Buffer Size */
    NvmeRegBpinfo	= 0x0040,	/* Boot Partition Information */
 	NvmeRegBprsel	= 0x0044,	/* Boot Partition Read Select */
	NvmeRegBpmbl	= 0x0048,	/* Boot Partition Memory Buffer
					 * Location
					 */
	NvmeRegCmbmsc = 0x0050,	/* Controller Memory Buffer Memory
					 * Space Control
					 */
	NvmeRegCrto	= 0x0068,	/* Controller Ready Timeouts */
	NvmeRegPmrcap	= 0x0e00,	/* Persistent Memory Capabilities */
    NvmeRegPmrctl	= 0x0e04,	/* Persistent Memory Control */
    NvmeRegPmrsts	= 0x0e08,	/* Persistent Memory Status */
    NvmeRegPmrebs	= 0x0e0c,	/* Persistent Memory Region Elasticity
					 * Buffer Size
					 */
	NvmeRegPmrswtp = 0x0e10,	/* Persistent Memory Region Sustained
					 * Write Throughput
					 */
	NvmeRegDbs	= 0x1000,	/* SQ 0 Tail Doorbell */
}



//NvmeRegister
pub const NVME_REG_CAP:usize	= 0x0000;	/* Controller Capabilities */
pub const NVME_REG_VS:usize	    = 0x0008;	/* Version */
pub const NVME_REG_INTMS:usize	= 0x000c;	/* Interrupt Mask Set */
pub const NVME_REG_INTMC:usize	= 0x0010;	/* Interrupt Mask Clear */
pub const NVME_REG_CC:usize	    = 0x0014;	/* Controller Configuration */
pub const NVME_REG_CSTS:usize	= 0x001c;	/* Controller Status */
pub const NVME_REG_NSSR:usize	= 0x0020;	/* NVM Subsystem Reset */
pub const NVME_REG_AQA:usize    = 0x0024;	/* Admin Queue Attributes */
pub const NVME_REG_ASQ:usize    = 0x0028;	/* Admin SQ Base Address */
pub const NVME_REG_ACQ:usize    = 0x0030;	/* Admin CQ Base Address */
pub const NVME_REG_CMBLOC:usize	= 0x0038;	/* Controller Memory Buffer Location */
pub const NVME_REG_CMBSZ:usize	= 0x003c;	/* Controller Memory Buffer Size */
pub const NVME_REG_BPINFO:usize	= 0x0040;	/* Boot Partition Information */
pub const NVME_REG_BPRSEL:usize	= 0x0044;	/* Boot Partition Read Select */
pub const NVME_REG_BPMBL:usize	= 0x0048;	/* Boot Partition Memory Buffer
         				 * Location
         				 */
pub const NVME_REG_CMBMSC:usize = 0x0050;	/* Controller Memory Buffer Memory
         				 * Space Control
         				 */
pub const NVME_REG_CRTO:usize	= 0x0068;	/* Controller Ready Timeouts */
pub const NVME_REG_PMRCAP:usize	= 0x0e00;	/* Persistent Memory Capabilities */
pub const NVME_REG_PMRCTL:usize	= 0x0e04;	/* Persistent Memory Region Control */
pub const NVME_REG_PMRSTS:usize	= 0x0e08;	/* Persistent Memory Region Status */
pub const NVME_REG_PMREBS:usize	= 0x0e0c;	/* Persistent Memory Region Elasticity
         				 * Buffer Size
         				 */
pub const NVME_REG_PMRSWTP:usize = 0x0e10;	/* Persistent Memory Region Sustained
         				 * Write Throughput
         				 */
pub const NVME_REG_DBS:usize	= 0x1000;	/* SQ 0 Tail Doorbell */


// 16 bytes
pub struct NvmeCompleteQueue{
    pub byte8_1: u64,
    pub byte8_2: u64,
}
