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




/*
 * Initialize a NVMe controller structures.  This needs to be called during
 * earliest initialization so that we have the initialized structured around
 * during probing.
 */
// int nvme_init_ctrl(struct nvme_ctrl *ctrl, struct device *dev,
// 		const struct nvme_ctrl_ops *ops, unsigned long quirks)
// {

// static struct nvme_ns *nvme_alloc_ns(struct nvme_dev *dev, unsigned nsid,
//             struct nvme_id_ns *id, struct nvme_lba_range_type *rt)
// {
//     struct nvme_ns *ns;
//     struct gendisk *disk;
//     int lbaf;

//     if (rt->attributes & NVME_LBART_ATTRIB_HIDE)
//         return NULL;

//     ns = kzalloc(sizeof(*ns), GFP_KERNEL);
//     if (!ns)
//         return NULL;
//     /* 分配一个request queue */
//     ns->queue = blk_alloc_queue(GFP_KERNEL);
//     if (!ns->queue)
//         goto out_free_ns;
//     ns->queue->queue_flags = QUEUE_FLAG_DEFAULT;
//     /* 禁止合并操作，包括bio合并到request操作，两个request合并操作 */
//     queue_flag_set_unlocked(QUEUE_FLAG_NOMERGES, ns->queue);
//     /* 表示是一个ssd设备 */
//     queue_flag_set_unlocked(QUEUE_FLAG_NONROT, ns->queue);
//     queue_flag_clear_unlocked(QUEUE_FLAG_ADD_RANDOM, ns->queue);
//     /* 绑定request queue的make_request_fn函数到nvme_make_request */
//     blk_queue_make_request(ns->queue, nvme_make_request);
//     ns->dev = dev;
//     ns->queue->queuedata = ns;

//     /* 分配一个gendisk结构,gendisk用于描述一个块设备 */
//     disk = alloc_disk(0);
//     if (!disk)
//         goto out_free_queue;
//     ns->ns_id = nsid;
//     ns->disk = disk;
//     lbaf = id->flbas & 0xf;
//     ns->lba_shift = id->lbaf[lbaf].ds;
//     ns->ms = le16_to_cpu(id->lbaf[lbaf].ms);
//     /* 物理sector的大小,用户看到的逻辑sector大小一般是512B,而物理sector大小不同厂商不同定义,可能跟一个nand flash page一样,也可能小于一个nand flash page */
//     blk_queue_logical_block_size(ns->queue, 1 << ns->lba_shift);
//     /* 设备允许的一次request支持最大sector数量,request中的sector数量不能超过此值 */
//     if (dev->max_hw_sectors)
//         blk_queue_max_hw_sectors(ns->queue, dev->max_hw_sectors);
//     if (dev->vwc & NVME_CTRL_VWC_PRESENT)
//         blk_queue_flush(ns->queue, REQ_FLUSH | REQ_FUA);

//     disk->major = nvme_major;
//     disk->first_minor = 0;
//     /* 此块设备的操作函数 */
//     disk->fops = &nvme_fops;
//     disk->private_data = ns;
//     /* 将上面初始化好的request queue与gendisk联系一起 */
//     disk->queue = ns->queue;
//     disk->driverfs_dev = &dev->pci_dev->dev;
//     /* 标记为允许扩展的设备,暂时不清楚什么意思 */
//     disk->flags = GENHD_FL_EXT_DEVT;
//     /* 在/dev/下显示的名字 */
//     sprintf(disk->disk_name, "nvme%dn%d", dev->instance, nsid);
//     /* 设置用户可用容量 */
//     set_capacity(disk, le64_to_cpup(&id->nsze) << (ns->lba_shift - 9));

//     /* 如果此nvme盘支持discard操作,则设置discard的一些初始参数,如discard必须以物理sector大小对齐 */
//     if (dev->oncs & NVME_CTRL_ONCS_DSM)
//         nvme_config_discard(ns);

//     return ns;

//  out_free_queue:
//     blk_cleanup_queue(ns->queue);
//  out_free_ns:
//     kfree(ns);
//     return NULL;
// }


// 这里主要初始化gendisk和request queue,gendisk用于描述一个块设备,也就是当gendisk初始化好后,并调用add_disk(),
// 就会在/dev/下出现一个此gendisk->name的块设备.而request_queue有什么用呢,注意看gendisk初始化时,
// 会将gendisk->queue设置为一个初始化好的request_queue.对于request_queue,最重要的是初始化一个make_request_fn的函数指针,
// 当有进程对此gendisk对应的块设备进行读写时,最终都会调用到gendisk的request_queue的make_request_fn所指的函数.
// 在nvme驱动中,主要将request_queue的make_request_fn初始化为了nvme_make_request()函数,未来在说nvme设备的读写流程时,会详细说明此函数.





























// pub trait BlockScheme: Scheme {
//     fn read_block(&self, block_id: usize, buf: &mut [u8]) -> DeviceResult;
//     fn write_block(&self, block_id: usize, buf: &[u8]) -> DeviceResult;
//     fn flush(&self) -> DeviceResult;
// }



// pub struct NvmeDev {

//     queues: Vec<NvmeQueue>,

//     io_queues: usize,

//     dbs: usize,

//     pci_dev: usize,

//     prp_page_pool: DmaPool,

//     prp_small_pool: DmaPool,

//     instance: u16,

//     max_qid: u16,
    
//     online_queues:u16,


//     queue_count: usize,
//     // io_queues:

//     // num_vecs: u16,

//     // q_depth: u32,

//     io_sqes: u16,

//     db_stride: u16,

//     bar: usize,

//     bar_mapped_size: usize,

//     vendor: &str,

//     serial: &str,

//     model: &str,

//     firmware_rev: &str,

//     max_transfer_shift: u32,

//     cap: u64,

//     stripe_size: u32,

//     page_size: u32,

//     cmb_size: u64,

//     cmb_use_sqes: bool,

//     vwc: bool,

//     cmbsz: u32,

//     cmbloc: u32,

// }
