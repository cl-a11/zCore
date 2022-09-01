use alloc::collections::{VecDeque};




// pub struct RawQueue{

// }



use super::NvmeRWCommand;

pub struct NvmeQueue{
    pub sq: VecDeque<NvmeRWCommand>,
    pub cq: VecDeque<NvmeRWCommand>,
}






impl NvmeQueue{

    /* 分配一个nvme queue，包括其需要的CQ和SQ空间和DMA地址 */
    /* 通过admin queue告知nvme设备创建cq和sq，并且分配对应的irq */
    pub fn new() -> Self{
        NvmeQueue{
            sq: VecDeque::new(),
            cq: VecDeque::new(),
        }
    }

    pub fn sq_push(&mut self, cmd: NvmeRWCommand){
        self.sq.push_front(cmd);
    }
}




// Doorbellregister
// SQ位于Host内存中，Host要发送命令时，先把准备好的命令放在SQ中，然后通知SSD来取；
// CQ也是位于Host内存中，一个命令执行完成，成功或失败，SSD总会往CQ中写入命令完成状态。
// DB又是干什么用的呢？Host发送命令时，不是直接往SSD中发送命令的，而是把命令准备好放在自己的内存中，
// 那怎么通知SSD来获取命令执行呢？
// Host就是通过写SSD端的DB寄存器来告知SSD的

// SQ = Submission Queue
// CQ = Completion Queue
// DB = Doorbell Register

// 第一步：Host写命令到SQ；

// 第二步：Host写DB，通知SSD取指；

// 第三步：SSD收到通知，于是从SQ中取指；

// 第四步：SSD执行指令；

// 第五步：指令执行完成，SSD往CQ中写指令执行结果；

// 第六步：然后SSD发起中断通知Host指令完成；

// 第七步：收到中断，Host处理CQ，查看指令完成状态；

// 第八步：Host处理完CQ中的指令执行结果，通过DB回复SSD：指令执行结果已处理，辛苦您了！



// host往sq1中写入3个命令, sq1.tail=3, qs DBR = 3, 

// 执行完2个命令, cq DBR=2


// db记录了sq 和 cq 的头和尾

// ssd 控制器知道sq的head位置

// host知道sq的tail位置

// SSD往CQ中写入命令状态信息的同时，还把SQ Head DB的信息告知了Host

// cq host 知道head 不知道tail
// 一开始cq中每条命令完成条目中的 p bit初始化为0, ssd在往cq中写入命令完成条目是p bit置为1, host在处理cq中的命令完成条目时, p bit置为0,
// cq是在host的内存中, hist记住上次的tail, 检查p 得出新的tail
// /*
//  * An NVM Express queue.  Each device has at least two (one for admin
//  * commands and one for I/O commands).
//  */
// struct nvme_queue {
// 	struct nvme_dev *dev;
// 	spinlock_t sq_lock;
// 	void *sq_cmds;
// 	 /* only used for poll queues: */
// 	spinlock_t cq_poll_lock ____cacheline_aligned_in_smp;
// 	struct nvme_completion *cqes;
// 	dma_addr_t sq_dma_addr;
// 	dma_addr_t cq_dma_addr;
// 	u32 __iomem *q_db;
// 	u32 q_depth;
// 	u16 cq_vector;
// 	u16 sq_tail;
// 	u16 last_sq_tail;
// 	u16 cq_head;
// 	u16 qid;
// 	u8 cq_phase;
// 	u8 sqes;
// 	unsigned long flags;
// #define NVMEQ_ENABLED		0
// #define NVMEQ_SQ_CMB		1
// #define NVMEQ_DELETE_ERROR	2
// #define NVMEQ_POLLED		3
// 	u32 *dbbuf_sq_db;
// 	u32 *dbbuf_cq_db;
// 	u32 *dbbuf_sq_ei;
// 	u32 *dbbuf_cq_ei;
// 	struct completion delete_done;
// };




// pub struct NvmeQueue{
//     pub queue_id: u16,
//     pub cq: Arc<Mutex<NvmeCompletionQueue>>,
//     pub sq: Arc<Mutex<NvmeSubmissionQueue>>,
//     pub nvme: Arc<Mutex<NvmeController>>,
//     pub irq: Arc<Mutex<Option<Irq>>>,
//     pub irq_handler: Arc<Mutex<Option<Box<dyn FnMut()>>>>,
// }

// impl NvmeQueue {
//     pub fn new(queue_id: u16, nvme: Arc<Mutex<NvmeController>>, irq: Option<Irq>) -> NvmeQueue {
//         let cq = NvmeCompletionQueue::new(queue_id, nvme.clone());
//         let sq = NvmeSubmissionQueue::new(queue_id, nvme.clone());
//         let irq_handler = None;
//         NvmeQueue {
//             queue_id,
//             cq: Arc::new(Mutex::new(cq)),
//             sq: Arc::new(Mutex::new(sq)),
//             nvme,
//             irq: Arc::new(Mutex::new(irq)),
//             irq_handler: Arc::new(Mutex::new(irq_handler)),
//         }
//     }

//     pub fn irq_handler(&self, handler: Box<dyn FnMut()>) {
//         let mut irq_handler = self.irq_handler.lock();
//         *irq_handler = Some(handler);
//     }

//     pub fn irq_handler_run(&self) {
//         let mut irq_handler = self.irq_handler.lock();
//         if let Some(handler) = irq_handler.as_mut() {
//             handler();
//         }
//     }

//     pub fn irq_handler_clear(&self) {
//         let mut irq_handler = self.irq_handler.lock();
//         *irq_handler = None;
//     }

//     pub fn irq_enable(&self) {
//         let mut irq = self.irq.lock();
//         if let Some(irq) = irq.as_mut() {
//             irq.enable();
//         }
//     }

//     pub fn irq_disable(&self) {
//         let mut irq = self.irq.lock();
//         if let Some(irq) = irq.as_mut() {
//             irq.disable();
//         }
//     }

//     pub fn irq_handler_install(&self) {
//         let nvme = self.nvme.clone();
//         let irq_handler = self.irq_handler.clone

// }









// /*
//  * An NVM Express queue.  Each device has at least two (one for admin
//  * commands and one for I/O commands).
//  */
// struct nvme_queue {
// 	struct nvme_dev *dev;
// 	spinlock_t sq_lock;
// 	void *sq_cmds;
// 	 /* only used for poll queues: */
// 	spinlock_t cq_poll_lock ____cacheline_aligned_in_smp;
// 	struct nvme_completion *cqes;
// 	dma_addr_t sq_dma_addr;
// 	dma_addr_t cq_dma_addr;
// 	u32 __iomem *q_db;
// 	u32 q_depth;
// 	u16 cq_vector;
// 	u16 sq_tail;
// 	u16 last_sq_tail;
// 	u16 cq_head;
// 	u16 qid;
// 	u8 cq_phase;
// 	u8 sqes;
// 	unsigned long flags;
// #define NVMEQ_ENABLED		0
// #define NVMEQ_SQ_CMB		1
// #define NVMEQ_DELETE_ERROR	2
// #define NVMEQ_POLLED		3
// 	u32 *dbbuf_sq_db;
// 	u32 *dbbuf_cq_db;
// 	u32 *dbbuf_sq_ei;
// 	u32 *dbbuf_cq_ei;
// 	struct completion delete_done;
// };




// pub struct NvmeDev {
//     queues: Vec<Arc<Mutex<NvmeQueue>>>,

//     q_depth: u32,
//     max_qid: u16,
//     prp_page_pool: Arc<Mutex<DmaPool>>,
//     prp_small_pool: Arc<Mutex<DmaPool>>,
//     online_queues: u16,
//     max_qid: u16,
// }


// /*
//  * Represents an NVM Express device.  Each nvme_dev is a PCI function.
//  */
// struct nvme_dev {
// 	struct nvme_queue *queues;
// 	struct blk_mq_tag_set tagset;
// 	struct blk_mq_tag_set admin_tagset;
// 	u32 __iomem *dbs;
// 	struct device *dev;
// 	struct dma_pool *prp_page_pool;
// 	struct dma_pool *prp_small_pool;
// 	unsigned online_queues;
// 	unsigned max_qid;
// 	unsigned io_queues[HCTX_MAX_TYPES];
// 	unsigned int num_vecs;
// 	u32 q_depth;
// 	int io_sqes;
// 	u32 db_stride;
// 	void __iomem *bar;
// 	unsigned long bar_mapped_size;
// 	struct work_struct remove_work;
// 	struct mutex shutdown_lock;
// 	bool subsystem;
// 	u64 cmb_size;
// 	bool cmb_use_sqes;
// 	u32 cmbsz;
// 	u32 cmbloc;
// 	struct nvme_ctrl ctrl;
// 	u32 last_ps;
// 	bool hmb;

// 	mempool_t *iod_mempool;

// 	/* shadow doorbell buffer support: */
// 	u32 *dbbuf_dbs;
// 	dma_addr_t dbbuf_dbs_dma_addr;
// 	u32 *dbbuf_eis;
// 	dma_addr_t dbbuf_eis_dma_addr;

// 	/* host memory buffer support: */
// 	u64 host_mem_size;
// 	u32 nr_host_mem_descs;
// 	dma_addr_t host_mem_descs_dma;
// 	struct nvme_host_mem_buf_desc *host_mem_descs;
// 	void **host_mem_desc_bufs;
// 	unsigned int nr_allocated_queues;
// 	unsigned int nr_write_queues;
// 	unsigned int nr_poll_queues;

// 	bool attrs_added;
// };


// // struct dma_pool {		/* the pool */
// // 	struct list_head page_list;
// // 	spinlock_t lock;
// // 	size_t size;
// // 	struct device *dev;
// // 	size_t allocation;
// // 	size_t boundary;
// // 	char name[32];
// // 	struct list_head pools;
// // };

// pub struct DmaPool{
//     page_list: Vec<Arc<Mutex<DmaPage>>>,
//     lock: Mutex<()>,
//     size: usize,
//     dev: Arc<Mutex<Device>>,
//     allocation: usize,
//     boundary: usize,
//     name: String,
//     pools: Vec<Arc<Mutex<DmaPool>>>,
// }




// pub fn nvme_alloc_queue(){

// }



// nvme_pci_setup_prps



    // /* Writing/Reading PRP1 */
    // res = do_rw_prp(n, e->prp1, &data_size, &file_offset, mapping_addr,
    //     e->opcode);
    
    // if (data_size > 0) {
    //     if (data_size <= PAGE_SIZE) {
    //         res = do_rw_prp(n, e->prp2, &data_size, &file_offset, mapping_addr,
    //             e->opcode);
    //     } else {
    //         res = do_rw_prp_list(n, sqe, &data_size, &file_offset,
    //             mapping_addr);
    //     }
    // }


    // let dma_addr = 0x1200;

    // let mut length = 0x1200;

    // let page_size = 0x1000;

    // let offset = dma_addr & (page_size as usize - 1 as usize);

    // length -= page_size - offset;



// static int nvme_setup_prps(struct nvme_dev *dev, u64 *prp2,
// 			   int total_len, u64 dma_addr)
// {
// 	u32 page_size = dev->page_size;

//     //page_size 余数
// 	int offset = dma_addr & (page_size - 1);
// 	u64 *prp_pool;
// 	int length = total_len;
// 	int i, nprps;
// 	u32 prps_per_page = page_size >> 3;
// 	u32 num_pages;

// 	length -= (page_size - offset);

// 	if (length <= 0) {
// 		*prp2 = 0;
// 		return 0;
// 	}

// 	if (length)
// 		dma_addr += (page_size - offset);

// 	if (length <= page_size) {
// 		*prp2 = dma_addr;
// 		return 0;
// 	}

// 	nprps = DIV_ROUND_UP(length, page_size);
// 	num_pages = DIV_ROUND_UP(nprps, prps_per_page);

// 	if (nprps > dev->prp_entry_num) {
// 		free(dev->prp_pool);
// 		/*
// 		 * Always increase in increments of pages.  It doesn't waste
// 		 * much memory and reduces the number of allocations.
// 		 */
// 		dev->prp_pool = memalign(page_size, num_pages * page_size);
// 		if (!dev->prp_pool) {
// 			printf("Error: malloc prp_pool fail\n");
// 			return -ENOMEM;
// 		}
// 		dev->prp_entry_num = prps_per_page * num_pages;
// 	}

// 	prp_pool = dev->prp_pool;
// 	i = 0;
// 	while (nprps) {
// 		if (i == ((page_size >> 3) - 1)) {
// 			*(prp_pool + i) = cpu_to_le64((ulong)prp_pool +
// 					page_size);
// 			i = 0;
// 			prp_pool += page_size;
// 		}
// 		*(prp_pool + i++) = cpu_to_le64(dma_addr);
// 		dma_addr += page_size;
// 		nprps--;
// 	}
// 	*prp2 = (ulong)dev->prp_pool;

// 	flush_dcache_range((ulong)dev->prp_pool, (ulong)dev->prp_pool +
// 			   dev->prp_entry_num * sizeof(u64));

// 	return 0;
// }