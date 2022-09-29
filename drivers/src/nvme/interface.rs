use alloc::string::String;
use alloc::vec::Vec;
use core::mem::size_of;
use core::ptr::{read_volatile, write_volatile};
use alloc::sync::Arc;

use crate::scheme::{BlockScheme, Scheme};
use crate::DeviceResult;

use lock::Mutex;

use super::dev::*;
use super::driver::*;
use super::queue::*;

pub struct NvmeRes(Arc<Mutex<Nvme<ProviderImpl>>>);

pub struct NvmeInterface {
    name: String,

    dev: NvmeDev,

    nvme: Arc<Mutex<Nvme<ProviderImpl>>>,

    irq: usize,
}

impl NvmeInterface{

    pub fn new(irq: usize, bar:usize, len:usize) -> DeviceResult<NvmeInterface>{

        let dev = NvmeDev::new(bar);
        
        let nvme: Arc<Mutex<Nvme<ProviderImpl>>> = Arc::new(Mutex::new(super::Nvme::new()));

        let mut interface = NvmeInterface{
            name: String::from("real_nvme"),
            dev,
            nvme: nvme,
            irq,
        };

        interface.init();

        warn!("new nvme dev init done");
        Ok(interface)

    }

    pub fn new_early() -> DeviceResult<NvmeInterface>{

        let dev = NvmeDev::new(0);
        let nvme: Arc<Mutex<Nvme<ProviderImpl>>> = Arc::new(Mutex::new(super::Nvme::new()));

        let interface = NvmeInterface{
            name: String::from("fake_nvme"),
            dev,
            nvme:nvme,
            irq:0,
        };
        Ok(interface)
    }


    // 第一，设置映射设备的bar空间到内核的虚拟地址空间当中，通过调用ioremap函数，将Controller的nvme寄存器映射到内核后，可以通过writel, readl这类函数直接读写寄存器。
    // 第二, 完成 DMA mask设置、pci总线中断分配、配置 queue depth、stride 等参数
    // 第三，设置admin queue，admin queue设置之后，才能发送nvme admin Command。
    // 第四，添加nvme namespace设备，即/dev/nvme#n#，这样就可以对设备进行读写操作了。
    // 第五，添加nvme Controller设备，即/dev/nvme#，提供ioctl接口。这样userspace就可以通过ioctl系统调用发送nvme admin command。
    // 参考 linux 5.19  nvme_reset_work    nvme_pci_configure_admin_queue
    pub fn init(&mut self) {


        self.nvme_configure_admin_queue();

        self.nvme_alloc_io_queue();


    }
}

impl NvmeInterface {

    pub fn nvme_configure_admin_queue(&self){
        
        let bar = self.dev.bar;
        let sq_dma_pa = self.nvme.as_ref().lock().sq_dma_pa as u32;
        let cq_dma_pa = self.nvme.as_ref().lock().cq_dma_pa as u32;
        let data_dma_pa = self.nvme.as_ref().lock().data_dma_pa as u64;



        let aqa_low_16 = 31 as u16;
        let aqa_high_16 = 31 as u16;
        let aqa = (aqa_high_16 as u32) << 16 | aqa_low_16 as u32;
        let aqa_address = bar + NVME_REG_AQA;

        // 将admin queue配置信息写入nvme设备寄存器AQA, admin_queue_attributes
        unsafe{
            write_volatile(aqa_address as *mut u32, aqa);
        }
        info!("nvme aqa {:#x?} aqa_address {:#x?}", aqa, aqa_address);



        
        // 将admin queue的sq dma物理地址写入nvme设备上的寄存器ASQ
        let asq_address = bar + NVME_REG_ASQ ;
        info!("nvme asq_address {:#x?} sq_dma_pa {:#x?}", asq_address, sq_dma_pa);
        unsafe{
            write_volatile(asq_address as *mut u32, sq_dma_pa);
        }

        // 将admin queue的cq dma物理地址写入nvme设备上的寄存器ACQ
        let acq_address = bar + NVME_REG_ACQ;
        info!("nvme acq_address {:#x?} cq_dma_pa {:#x?}", acq_address, cq_dma_pa);
        unsafe{
            write_volatile(acq_address as *mut u32, cq_dma_pa);
        }
        let dev_dbs = bar + NVME_REG_DBS;

        // enable ctrl
        let enable_ctrl = 0x460061;        
        unsafe{
            write_volatile((bar + NVME_REG_CC) as *mut u32, enable_ctrl)
        }
        
        let dev_status = unsafe {
            read_volatile((bar + NVME_REG_CSTS) as *mut u32)
        };

        info!("nvme dev_status {:#x?}", dev_status);


        // config identify
        let mut cmd = NvmeIdentify::new();

        cmd.prp1 = data_dma_pa;
        
        cmd.command_id = 0x1018;
        cmd.nsid = 1;
        let mut common_cmd = unsafe {
            core::mem::transmute(cmd)
        };
        // info!("cmd :{:#x?}", common_cmd);
        self.nvme.as_ref().lock().sq[0].write(common_cmd);


        let admin_q_db = dev_dbs;
        warn!("admin_q_db {:#x?}", admin_q_db);
        unsafe{
            write_volatile(admin_q_db as *mut u32, 1)
        }

        loop {
            let status = self.nvme.as_ref().lock().cq[0].read();
            if status.status != 0 {
                info!("nvme cq :{:#x?}", status);

                unsafe{
                    write_volatile((admin_q_db + 0x4) as *mut u32, 1)
                }
                break;
            }
        }

    }

    pub fn nvme_alloc_io_queue(&self){
        let bar = self.dev.bar;
        let sq_dma_pa = self.nvme.as_ref().lock().sq_dma_pa as u32;
        let cq_dma_pa = self.nvme.as_ref().lock().cq_dma_pa as u32;
        let data_dma_pa = self.nvme.as_ref().lock().data_dma_pa as u64;

        let dev_dbs = bar + NVME_REG_DBS;
        let admin_q_db = dev_dbs;

        //nvme_set_queue_count
        let mut cmd = NvmeCommonCommand::new();
        cmd.opcode = 0x09;
        cmd.command_id = 0x101a;
        cmd.nsid = 1;
        cmd.cdw10 = 0x7;
        let common_cmd = unsafe {
            core::mem::transmute(cmd)
        };

        // info!("cmd :{:#x?}", common_cmd);
        self.nvme.as_ref().lock().sq[2].write(common_cmd);

        unsafe{
            write_volatile(admin_q_db as *mut u32, 3)
        }

        loop {
            let status = self.nvme.as_ref().lock().cq[2].read();
            if status.status != 0 {
                info!("nvme cq :{:#x?}", status);
                unsafe{
                    write_volatile((admin_q_db + 0x4) as *mut u32, 3)
                }
                break;
            }
        }

        //nvme create cq
        let mut cmd = NvmeCommonCommand::new();
        cmd.opcode = 0x05;
        cmd.command_id = 0x101b;
        cmd.nsid = 1;
        cmd.prp1 = cq_dma_pa as u64;
        cmd.cdw10 = 0x3ff0001;
        cmd.cdw11 = 0x3;
        // cmd.flags = 3;
        let common_cmd = unsafe {
            core::mem::transmute(cmd)
        };
        // info!("cmd :{:#x?}", common_cmd);
        self.nvme.as_ref().lock().sq[3].write(common_cmd);
                unsafe{
            write_volatile(admin_q_db as *mut u32, 4)
        }
        loop {
            let status = self.nvme.as_ref().lock().cq[3].read();
            if status.status != 0 {
                info!("nvme cq :{:#x?}", status);
                unsafe{
                    write_volatile((admin_q_db + 0x4) as *mut u32, 4)
                }
                break;
            }
        }


        //nvme create sq
        let mut cmd = NvmeCommonCommand::new();
        cmd.opcode = 0x01;
        cmd.command_id = 0x2018;
        cmd.nsid = 1;
        cmd.prp1 = self.nvme.as_ref().lock().sq_dma_pa as u64;
        cmd.cdw10 = 0x3ff0001;
        cmd.cdw11 = 0x10001;
        // cmd.flags = (NVME_QUEUE_PHYS_CONTIG | NVME_SQ_PRIO_MEDIUM)  as u8;

        let common_cmd = unsafe {
            core::mem::transmute(cmd)
        };
        // info!("cmd :{:#x?}", z);
        self.nvme.as_ref().lock().sq[4].write(common_cmd);
                unsafe{
            write_volatile(admin_q_db as *mut u32, 5)
        }
        loop {
            let status = self.nvme.as_ref().lock().cq[4].read();
            if status.status != 0 {
                info!("nvme cq :{:#x?}", status);
                unsafe{
                    write_volatile((admin_q_db + 0x4) as *mut u32, 5)
                }
                break;
            }
        }    
    }
}

impl BlockScheme for NvmeInterface {
    
    // 每个NVMe命令中有两个域：PRP1和PRP2，Host就是通过这两个域告诉SSD数据在内存中的位置或者数据需要写入的地址
    // 首先对prp1进行读写，如果数据还没完，就看数据量是不是在一个page内，在的话，只需要读写prp2内存地址就可以了，数据量大于1个page，就需要读出prp list
    
    // 由于只读一块, 小于一页, 所以只需要prp1
    // prp1 = dma_addr 
    // prp2 = 0
    
    // uboot中对应实现 nvme_setup_prps
    // linux中对应实现 nvme_pci_setup_prps
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> DeviceResult {
        info!("-----------read data----------");

        let bar = self.dev.bar;
        let sq_dma_pa = self.nvme.as_ref().lock().sq_dma_pa as u32;
        let cq_dma_pa = self.nvme.as_ref().lock().cq_dma_pa as u32;
        let data_dma_pa = self.nvme.as_ref().lock().data_dma_pa as u64;

        let dev_dbs = bar + NVME_REG_DBS;
        let admin_q_db = dev_dbs;



        // let mut read_buf = [0 as u8;512];
        
        // 这里dma addr 就是buffer的地址
        let ptr = buf.as_mut_ptr();

        let addr = virt_to_phys(ptr as usize);

        let mut cmd = NvmeRWCommand::new_read_command();
        cmd.nsid = 1;
        cmd.prp1 = addr as u64;
        cmd.command_id = 101;
        cmd.length = 0x7;
        cmd.slba = (block_id as u64 / 8) * 8;

        let common_cmd = unsafe {
            core::mem::transmute(cmd)
        };


        self.nvme.as_ref().lock().sq[6].write(common_cmd);

        unsafe{
            write_volatile((admin_q_db + 8)as *mut u32, 7)
        }
        loop {
            let status = self.nvme.as_ref().lock().cq[6].read();
            if status.status != 0 {
                info!("nvme cq :{:#x?}", status);
                unsafe{
                    write_volatile((admin_q_db + 0xc) as *mut u32, 7)
                }
                break;
            }
        }

        Ok(())
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) -> DeviceResult {
        info!("-----------write data----------");
        
        // 1 SLBA = 512B
        // 1 sector = 4KB
        // SLBA = (block_id / 8) * 8

        // Starting LBA (SLBA): 
        // This field indicates the 64-bit address of the first logical block to be read as part of the operation

        let bar = self.dev.bar;
        let sq_dma_pa = self.nvme.as_ref().lock().sq_dma_pa as u32;
        let cq_dma_pa = self.nvme.as_ref().lock().cq_dma_pa as u32;
        let data_dma_pa = self.nvme.as_ref().lock().data_dma_pa as u64;

        let dev_dbs = bar + NVME_REG_DBS;
        let admin_q_db = dev_dbs;


        let ptr = buf.as_ptr();

        let addr = virt_to_phys(ptr as usize);

        let mut cmd = NvmeRWCommand::new_write_command();
        cmd.nsid = 1;
        cmd.prp1 = addr as u64;
        cmd.length = 0x7;
        cmd.command_id = 100;
        cmd.slba = (block_id as u64 / 8) * 8;


        let common_cmd = unsafe {
            core::mem::transmute(cmd)
        };
        self.nvme.as_ref().lock().sq[5].write(common_cmd);
                unsafe{
            write_volatile((admin_q_db + 8)as *mut u32, 6)
        }

        // loop {
        //     let status = self.nvme.as_ref().lock().cq[5].read();
        //     if status.status != 0 {
        //         info!("nvme cq :{:#x?}", status);
        //         unsafe{
        //             write_volatile((admin_q_db + 0xc) as *mut u32, 6)
        //         }
        //         break;
        //     }
        // }
        Ok(())
    }

    fn flush(&self) -> DeviceResult {
        Ok(())
    }
}

impl Scheme for NvmeInterface {
    fn name(&self) -> &str {
        "real_nvme"
    }

    fn handle_irq(&self, irq: usize) {

        warn!("nvme device irq");
    }
}

pub fn nvme_init_early() -> DeviceResult<NvmeInterface> {
    let nvme_interface = NvmeInterface::new_early()?;
    Ok(nvme_interface)
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
//1+1+2+4+4*2+8+8+8+4*6 64B
pub struct NvmeCommonCommand {
	opcode: u8,
	flags: u8,
    command_id: u16,
    nsid: u32,
    cdw2: [u32;2],
    metadata: u64,
    prp1: u64,
    prp2: u64,
    cdw10: u32,
    cdw11: u32,
    cdw12: u32,
    cdw13: u32,
    cdw14: u32,
    cdw15: u32,
}

impl NvmeCommonCommand {
    pub fn new() -> Self{
        Self{
            opcode: 0,
            flags: 0,
            command_id: 0,
            nsid: 0,
            cdw2: [0;2],
            metadata: 0,
            prp1: 0,
            prp2: 0,
            cdw10: 0,
            cdw11: 0,
            cdw12: 0,
            cdw13: 0,
            cdw14: 0,
            cdw15: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum NvmeCommand {
    NvmeRWCommand,
    NvmeCreateSq,
	NvmeCreateCq,
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
// 4+4+2+2+2+2 16B
pub struct NvmeCompletion{
    pub result: u64,
    // pub rsvd: u32,
    pub sq_head: u16,
    pub sq_id: u16,
    pub command_id: u16,
    pub status: u16,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NvmeCreateCq{
    pub opcode: u8,
    pub flags: u8,
    pub command_id: u16,
    pub rsvd1: [u32;5],
    pub prp1: u64,
    pub rsvd8: u64,
    pub cqid: u16,
    pub qsize: u16,
    pub cq_flags: u16,
    pub irq_vector: u16,
    pub rsvd12: [u32;4],
}

pub const NVME_CQ_IRQ_ENABLED: u16 = 1 << 1;

impl NvmeCreateCq{
    pub fn new_create_cq_command() -> Self{
        let flags = NVME_QUEUE_PHYS_CONTIG | NVME_CQ_IRQ_ENABLED;
        NvmeCreateCq{
            opcode: 0x05,
            flags: 0,
            command_id: 2,
            rsvd1: [0 as u32; 5],
            prp1: 0,
            rsvd8: 0,
            cqid: 1,
            qsize: 31,
            cq_flags: 0x3ff,
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


pub const NVME_QUEUE_PHYS_CONTIG: u16 = 1 << 0;
pub const NVME_SQ_PRIO_MEDIUM: u16 = 2 << 1;

impl NvmeCreateSq{
    pub fn new_create_sq_command() -> Self{
        let flags = NVME_QUEUE_PHYS_CONTIG | NVME_SQ_PRIO_MEDIUM;
        NvmeCreateSq{
            opcode: 0x01,
            flags: 0,
            command_id: 3,
            rsvd1: [0 as u32; 5],
            prp1: 0,
            rsvd8: 0,
            sqid: 1,
            qsize: 32,
            sq_flags: 0x3ff,
            cqid: 0,
            rsvd12: [0 as u32; 4],
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
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
}

pub struct NvmeFeatures{
    opcode: u8,
    flags: u8,
    command_id: u16,
    nsid: u32,
    rsvd2: [u64;2],
    prp1: u64,
    prp2: u64,
    fid: u32,
    dword11: u32,
    rsvd12: [u32;4],
}

impl NvmeFeatures{
    pub fn new_admin_set_features() -> Self{
        Self{
            opcode: 0x09,
            flags: 0,
            command_id: 0,
            nsid: 0,
            rsvd2: [0;2],
            prp1: 0,
            prp2: 0,
            fid: 0,
            dword11: 0,
            rsvd12: [0;4],
        }
    }

    pub fn nvme_set_queue_count(count: u32) -> Self{

        Self{
            opcode: 0x09,
            flags: 0,
            command_id: 3,
            nsid: 0,
            rsvd2: [0;2],
            prp1: 0,
            prp2: 0,
            fid: 0x07,
            dword11: count,
            rsvd12: [0;4],
        }
    }
}
// 16 bytes
pub struct NvmeCompleteQueue{
    pub byte8_1: u64,
    pub byte8_2: u64,
}

pub const NVME_FEAT_NUM_QUEUES: u32 = 0x7;
// 1+1+2+4+8+8+8+8+1+1+2+1+1+1+1+4+4+4+4 = 64 bytes
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NvmeIdentify{
    opcode: u8,
    flags: u8,
    command_id: u16,
    nsid: u32,
    rsvd2: [u64;2],
    prp1:u64,
    prp2:u64,
    cns: u8,
    rsvd3: u8,
    ctrlid: u16,
    rsvd11: [u8;3],
    csi: u8,
    rsvd12: [u32;4],
}

impl NvmeIdentify{
    pub fn new() -> Self{
        Self{
            opcode: 0x06,
            flags: 0,
            command_id: 0x1,
            nsid: 1,
            rsvd2: [0;2],
            prp1: 0,
            prp2: 0,
            cns: 1,
            rsvd3: 0,
            ctrlid: 0,
            rsvd11: [0;3],
            csi: 0,
            rsvd12: [0;4],
        }
    }

    pub fn nvme_init_non_mdts_limits() -> Self{
        Self{
            opcode: 0x06,
            flags: 0,
            command_id: 2,
            nsid: 0,
            rsvd2: [0;2],
            prp1: 0,
            prp2: 0,
            cns: 0,
            rsvd3: 0,
            ctrlid: 0,
            rsvd11: [0;3],
            csi: 0,
            rsvd12: [0;4],
        }
    }
}

pub struct NvmeFeatAutoPst{
    pub entry: [u64;32],
}

impl NvmeFeatAutoPst {

    pub fn new() -> Self{
        Self{
            entry: [0;32],
        }
    }
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
