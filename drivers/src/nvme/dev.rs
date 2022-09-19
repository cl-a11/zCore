use alloc::vec::Vec;



use super::queue::NvmeQueue;
/* nvme设备描述符,描述一个nvme设备 */
/*
* Represents an NVM Express device.  Each nvme_dev is a PCI function.
*/
 // 此处最重要的是queues, 一个nvme设备至少有两个队列,admin队列和io队列,
 // 每个队列都有一个sq(submition queue)和一个cq(completion queue)

pub struct NvmeDev{

    //一个设备只能有一个admin队列
    pub admin_queue: NvmeQueue,

    //io队列可以有多个, 每个队列都有一个sq和一个cq
    pub io_queues: Vec<NvmeQueue>,

    // bar0~bar5, 目前只用bar0
    //base address register  pcie controller 向os提供的一组寄存器用于访问设备内部的空间(接收控制信息)
    pub bar: usize,

    // pub cap: usize,
    // pub dbs: usize,
}



impl NvmeDev{
    pub fn new(bar: usize) -> Self{
        NvmeDev{
            admin_queue: NvmeQueue::new(),
            io_queues: Vec::new(),
            bar: bar,
        }
    }
}
