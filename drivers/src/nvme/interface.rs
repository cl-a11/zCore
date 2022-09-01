use alloc::string::String;
use alloc::vec::Vec;
use core::mem::size_of;


use crate::scheme::{BlockScheme, Scheme};
use crate::{DeviceResult};

use super::dev::NvmeDev;
use super::driver::NvmeDriver;
use super::queue::NvmeQueue;


pub struct NvmeInterface {
    name: String,

    dev: NvmeDev,

    driver: NvmeDriver,

    irq: usize,
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
        //一次只读一块
        //512B
        let total_len = 512;
        let blkcnt = 1;
        let c = NvmeRWCommand::new_read_command();

        //每个NVMe命令中有两个域：PRP1和PRP2，Host就是通过这两个域告诉SSD数据在内存中的位置或者数据需要写入的地址

        // 首先对prp1进行读写，如果数据还没完，就看数据量是不是在一个page内，在的话，只需要读写prp2内存地址就可以了，数据量大于1个page，就需要读出prp list

        // 由于只读一块, 小于一页, 所以只需要prp1
        // prp1=dma_addr 
        // prp2=0

        //uboot中对应实现 nvme_setup_prps
        //linux中对应实现 nvme_pci_setup_prps
        let dma_addr = 0;
        let prp1 = dma_addr;

        let prp2 : u64 = 0;



        // self.driver.0.lock().read_block(block_id, buf)?;
        Ok(())
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) -> DeviceResult {


        assert_eq!(buf.len(), 512);
        //一次只读一块
        //512B
        let total_len = 512;
        let blkcnt = 1;
        let mut c = NvmeRWCommand::new_write_command();

        //每个NVMe命令中有两个域：PRP1和PRP2，Host就是通过这两个域告诉SSD数据在内存中的位置或者数据需要写入的地址

        // 首先对prp1进行读写，如果数据还没完，就看数据量是不是在一个page内，在的话，只需要读写prp2内存地址就可以了，数据量大于1个page，就需要读出prp list

        // 由于只读一块, 小于一页, 所以只需要prp1
        // prp1=dma_addr 
        // prp2=0

        //uboot中对应实现 nvme_setup_prps
        //linux中对应实现 nvme_pci_setup_prps
        let dma_addr = 0;
        let prp1 = dma_addr;
        let prp2 : u64 = 0;

        let src_ptr = buf.as_ptr() as u64;
        
        //riscv是小端模式, 故这里不做转换
        c.slba = block_id as u64;
        c.length = 1;
        c.prp1 = src_ptr;
        c.prp2 = prp2;

        self.nvme_submit_sync_cmd(c);







        Ok(())
    }

    fn flush(&self) -> DeviceResult {
        Ok(())
    }
}



impl NvmeInterface {
    pub fn nvme_submit_sync_cmd(&self, cmd:NvmeCommand) -> DeviceResult {

        match NvmeCommand{
            NvmeCommand::NvmeCreateCq => {

            }

            NvmeCommand::NvmeCreateSq => {

            }

            NvmeCommand::NvmeRWCommand => {

            }
            
            _ => {
                info!("wrong command");
            }
            
        }
        let mut io_queue = &mut self.dev.io_queues[0];

        // copy a command into a queue and ring the doorbell
        self.nvme_submit_cmd(io_queue, cmd);

    
        // wait for the command to complete
        self.nvme_read_completion_status(io_queue);



        Ok(())
    }


    pub fn nvme_read_completion_status(&self, nvmeq: &mut NvmeQueue) -> Option<usize>{




        Some(0)
    }

    pub fn nvme_submit_cmd(&self, nvmeq: &mut NvmeQueue, cmd:NvmeCommand){


        let cmdsize = size_of::<NvmeRWCommand>();

        nvmeq.sq_push(cmd);


        // ring the doorbell
        
    }
}





// /*
//  * Returns 0 on success.  If the result is negative, it's a Linux error code;
//  * if the result is positive, it's an NVM Express status code
//  */
// int __nvme_submit_sync_cmd(struct request_queue *q, struct nvme_command *cmd,
// 		union nvme_result *result, void *buffer, unsigned bufflen,
// 		unsigned timeout, int qid, int at_head,
// 		blk_mq_req_flags_t flags)
// {
// 	struct request *req;
// 	int ret;

// 	if (qid == NVME_QID_ANY)
// 		req = blk_mq_alloc_request(q, nvme_req_op(cmd), flags);
// 	else
// 		req = blk_mq_alloc_request_hctx(q, nvme_req_op(cmd), flags,
// 						qid ? qid - 1 : 0);

// 	if (IS_ERR(req))
// 		return PTR_ERR(req);
// 	nvme_init_request(req, cmd);

// 	if (timeout)
// 		req->timeout = timeout;

// 	if (buffer && bufflen) {
// 		ret = blk_rq_map_kern(q, req, buffer, bufflen, GFP_KERNEL);
// 		if (ret)
// 			goto out;
// 	}

// 	req->rq_flags |= RQF_QUIET;
// 	ret = nvme_execute_rq(req, at_head);
// 	if (result && ret >= 0)
// 		*result = nvme_req(req)->result;
//  out:
// 	blk_mq_free_request(req);
// 	return ret;
// }





enum NvmeCommand {
    NvmeRWCommand,
	NvmeCreateCq,
	NvmeCreateSq,
}


pub struct NvmeCreateCq{
    pub opcode: u8,
    pub flags: u8,
    pub command_id: u16,
    pub rsvd1: Vec<u32>,
    pub prp1: u64,
    pub rsvd8: u64,
    pub sqid: u16,
    pub qsize: u16,
    pub cq_flags: u16,
    pub irq_vector: u16,
    pub rsvd12: Vec<u32>,
}

impl NvmeCreateCq{
    pub fn new_create_cq_command() -> Self{
        NvmeCreateCq{
            opcode: 0x04,
            flags: 0,
            command_id: 0,
            rsvd1: alloc::vec![0 as u32; 5],
            prp1: 0,
            rsvd8: 0,
            sqid: 0,
            qsize: 0,
            cq_flags: 0,
            irq_vector: 0,
            rsvd12: alloc::vec![0 as u32; 4],
        }
    }
}
pub struct NvmeCreateSq{
    pub opcode: u8,
    pub flags: u8,
    pub command_id: u16,
    pub rsvd1: Vec<u32>,
    pub prp1: u64,
    pub rsvd8: u64,
    pub sqid: u16,
    pub qsize: u16,
    pub sq_flags: u16,
    pub cqid: u16,
    pub rsvd12: Vec<u32>,
}
impl NvmeCreateSq{
    pub fn new_create_sq_command() -> Self{
        NvmeCreateSq{
            opcode: 0x05,
            flags: 0,
            command_id: 0,
            rsvd1: alloc::vec![0 as u32; 5],
            prp1: 0,
            rsvd8: 0,
            sqid: 0,
            qsize: 0,
            sq_flags: 0,
            cqid: 0,
            rsvd12: alloc::vec![0 as u32; 4],
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



// static ulong nvme_blk_read(struct udevice *udev, lbaint_t blknr,
// 			   lbaint_t blkcnt, void *buffer)
// {
// 	return nvme_blk_rw(udev, blknr, blkcnt, buffer, true);
// }

// static ulong nvme_blk_write(struct udevice *udev, lbaint_t blknr,
// 			    lbaint_t blkcnt, const void *buffer)
// {
// 	return nvme_blk_rw(udev, blknr, blkcnt, (void *)buffer, false);
// }

// static const struct blk_ops nvme_blk_ops = {
// 	.read	= nvme_blk_read,
// 	.write	= nvme_blk_write,
// };




hdiutil convert -format UDRW -o /Users/Downloads/ubuntu-18.04.6-desktop-amd64 /Users/Downloads/ubuntu-18.04.6-desktop-amd64.iso