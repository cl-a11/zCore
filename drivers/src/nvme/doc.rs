// 在nvme_dev结构中,最最重要的数据就是nvme_queue,struct nvme_queue用来表示一个nvme的queue,每一个nvme_queue会申请自己的中断,也有自己的中断处理函数,也就是每个nvme_queue在驱动层面是完全独立的.nvme_queue有两种,一种是admin queue,一种是io queue,这两种queue都用struct nvme_queue来描述,而这两种queue的区别如下:

// admin queue: 用于发送控制命令的queue,所有非io命令都会通过此queue发送给nvme设备,一个nvme设备只有一个admin queue,在nvme_dev中,使用queues[0]来描述.
// io queue: 用于发送io命令的queue,所有io命令都是通过此queue发送给nvme设备,简单来说读/写操作都是通过io queue发送给nvme设备的,一个nvme设备有一个或多个io queue,每个io queue的中断会绑定到不同的一个或多个CPU上.在nvme_dev中,使用queues[1~N]来描述.



// /* nvme的命令队列，其中包括sq和cq。一个nvme设备至少包含两个命令队列
//  * 一个是控制命令队列，一个是IO命令队列
//  */
// struct nvme_queue {
//     struct rcu_head r_head;
//     struct device *q_dmadev;
//     /* 所属的nvme_dev */
//     struct nvme_dev *dev;
//     /* 中断名字，名字格式为nvme%dq%d，在proc/interrupts可以查看到 */
//     char irqname[24];    /* nvme4294967295-65535\0 */
//     /* queue的锁，当操作nvme_queue时，需要占用此锁 */
//     spinlock_t q_lock;
//     /* sq的虚拟地址空间，主机需要发给设备的命令就存在这里面 */
//     struct nvme_command *sq_cmds;
//     /* cq的虚拟地址空间，设备返回的命令就存在这里面 */
//     volatile struct nvme_completion *cqes;
//     /* 实际就是sq_cmds的dma地址 */
//     dma_addr_t sq_dma_addr;
//     /* cq的dma地址，实际就是cqes对应的dma地址，用于dma传输 */
//     dma_addr_t cq_dma_addr;
//     /* 等待队列，当sq满时，进程会加到此等待队列，等待有空闲的cmd区域 */
//     wait_queue_head_t sq_full;
//     /* wait queue的一个entry,主要是当cmdinfo满时,会将它放入sq_full,而sq_full最后会通过它,唤醒nvme_thread */
//     wait_queue_t sq_cong_wait;
//     struct bio_list sq_cong;
//     /* iod是读写请求的封装,可以看成是一个bio的封装,此链表有可能为空,比如admin queue就为空 */
//     struct list_head iod_bio;
//     /* 当前sq_tail位置，是nvme设备上的一个寄存器，告知设备最新的发送命令存在哪，存在于bar空间中 */
//     u32 __iomem *q_db;
//     /* cq和sq最大能够存放的command数量 */
//     u16 q_depth;
//     /* 如果是admin queue，那么为0，之后的io queue按分配顺序依次增加，主要用于获取对应的irq entry，因为所有的queue的irq entry是一个数组 */
//     u16 cq_vector;
//     /* 当完成命令时会更新，当sq_head == sq_tail时表示cmd queue为空 */
//     u16 sq_head;
//     /* 当有新的命令存放到sq时，sq_tail++，如果sq_tail == q_depth，那么sq_tail会被重新设置为0，并且cq_phase翻转 
//      * 实际上就是一个环
//      */
//     u16 sq_tail;
//     /* 驱动已经处理完成的cmd位置,当cq_head == sq_tail时,表示cmd队列为空,当sq_tail == cq_head - 1时表示cmd队列已满 */
//     u16 cq_head;
//     /* 此nvme queue在此nvme设备中的queue id 
//      * 0: 控制命令队列
//      */
//     u16 qid;
//     /* 初始设为1，主要用于判断命令是否完成，当cqe.status & 1 != cq_phase时，表示命令还没有完成
//      * 当每次sq_tail == q_depth时,此值会取反
//      */
//     u8 cq_phase;
//     u8 cqe_seen;
//     /* 初始设为1 */
//     u8 q_suspended;
//     /* CPU亲和性，用于设置此nvme queue能够在哪些CPU上做中断和中断处理 */
//     cpumask_var_t cpu_mask;
//     struct async_cmd_info cmdinfo;
//     /* 实际就是cmdinfo，此包含d_depth个cmdinfo，一个cmdid表示一个cmdinfo，当对应的bit为0时，表示此槽位空闲，为1时表示此槽位存有cmd 
//      * 空闲的cmdinfo的默认完成回调函数都是special_completion
//      * 其内存结构如下
//      *                      d_depth bits                                       d_depth cmdinfo
//      *   (每个bit一个cmdid，用于表示此cmdinfo是空闲还是被占用)              (d_depth个struct nvme_cmd_info)
//      * |                                                      |                                                   |
//      */
//     unsigned long cmdid_data[];
// };




// nvme_queue是nvme驱动最核心的数据结构,它是nvme驱动和nvme设备通信的桥梁,重点也要围绕nvme_queue来说,之前也说过,一个nvme设备有多个nvme_queue(一个admin queue,至少一个io queue),每个nvme_queue是独立的,它们有

// 自己对应的中断(irq)
// 自己的submission queue(sq),用于将struct nvme command发送给nvme设备,并且最多能存dev->d_depth个nvme command
// 自己的completion queue(cq),用于nvme设备将完成的命令信息(struct nvme_completion)发送给host,并且最多能存dev->d_depth个nvme_completion.
// 自己的cmdinfo,用于描述一个nvme command.(struct nvme_cmd_info)
// 可以把sq想象成一个struct nvme_command sq[dev->d_depth]的数组,而cq为struct nvme_completion cq[dev->d_depth]的数组.

// struct nvme_command主要用于存储一个nvme命令,包括io命令,或者控制命令,当初始化好一个struct nvme_command后,直接将其下发给nvme设备,nvme设备就会根据它来执行对应操作,其结构如下:



// struct nvme_command {
//     union {
//         struct nvme_common_command common;
//         struct nvme_rw_command rw;
//         struct nvme_identify identify;
//         struct nvme_features features;
//         struct nvme_create_cq create_cq;
//         struct nvme_create_sq create_sq;
//         struct nvme_delete_queue delete_queue;
//         struct nvme_download_firmware dlfw;
//         struct nvme_format_cmd format;
//         struct nvme_dsm_cmd dsm;
//         struct nvme_abort_cmd abort;
//     };
// };


// struct nvme_format_cmd {
//     __u8            opcode;
//     __u8            flags;
//     __u16            command_id;　　
//     __le32            nsid;
//     __u64            rsvd2[4];
//     __le32            cdw10;
//     __u32            rsvd11[5];
// };




// 按之前说的,我们把sq和cq想象成两个数组,比如驱动之前将一个nvme_format_cmd放到了sq[10]中,设备对这个nvme_format_cmd命令做了处理,这时候设备就会返回一个nvme_completion,并且把这个nvme_completion放入到cq[6](这里的index为6是假设,实际上我认为一个nvme_command对应一个nvme_completion,如果这个假设成立的话,正常情况这里应该也是为10),并且产生一个中断,在nvme queue的中断处理中,会获取到这个nvme_completion,并通过nvme_completion->sq_head就能够获取到sq[10]中的nvme_format_cmd.这样sq和cq就能够完全联系起来了.

// 对于驱动来说,一个命令应该是由两部分组成:

// 命令的格式,要通过怎样的格式发送给硬件,硬件能够识别.
// 命令的额外信息.
// 对于第一点,实际上就是nvme_command来做,而对于第二点,就需要用nvme_cmd_info来保存了,nvme_cmd_info也是一个数组,根据d_depth来分配长度(因为sq和cq都是根据d_depth来分配长度),并且nvme_queue还会维护一个nvme_cmd_info的used_bitmap,用来表示哪个nvme_cmd_info数组中哪个cmd_info已经被占用,nvme_cmd_info如下:


// struct nvme_cmd_info {
//     nvme_completion_fn fn;    // 命令完成后的回调函数
//     void *ctx;                // 命令的信息,不同命令使用不同结构来描述,所以这里只提供一个指针
//     unsigned long timeout;    // 命令允许的超时时间
//     int aborted;            // 命令是否作废
// };


// 现在来说说nvme驱动怎么把nvme_command,nvme_completion和nvme_cmd_info联系起来,以上面的nvme_format_cmd为例,假设nvme驱动要发送一个nvme_format_cmd命令,那么先会从nvme_cmd_info的used_bitmap中获取一个空闲的nvme_cmd_info(包括这个cmd_info对应的index,实际就是nvme_cmd_info的数组下标,也称为cmdid),然后根据nvme_format_cmd驱动需要做的事情和信息,来初始化这个nvme_cmd_info,将nvme_format_cmd中的command_id设置为cmdid,发送nvme_format_cmd给nvme设备,nvme设备处理完毕后,发送nvme_format_cmd对应的nvme_completion给host,host获取到此nvme_comletion,从command_id中获取到cmdid,根据获取到的cmdid就能够获取到对应的nvme_cmd_info了.也就是说,在将命令发送给nvme设备时,要将cmd_info对应的cmd_id也一并传下去,之后命令返回时,nvme设备也会将这个cmd_id传回来,这样就能够将三者对应联系起来了.


// nvme设备初始化
// 之前也说了,nvme驱动加载好后,如果有新的nvme设备加入,那么会通过nvme_probe来初始化这个nvme设备,我们先看看nvme_probe这个函数.


// /* 当插入一个nvme设备时，会通过此函数进行nvme设备的初始化 */
// static int nvme_probe(struct pci_dev *pdev, const struct pci_device_id *id)
// {
//     int result = -ENOMEM;
//     /* nvme设备描述符 */
//     struct nvme_dev *dev;

//     dev = kzalloc(sizeof(*dev), GFP_KERNEL);
//     if (!dev)
//         return -ENOMEM;
//     /* nvme用的是msi/msix中断，这里应该是是按numa内的CPU个数来分配entry数量，entry是msix_entry */
//     dev->entry = kcalloc(num_possible_cpus(), sizeof(*dev->entry),
//                                 GFP_KERNEL);
//     if (!dev->entry)
//         goto free;
//     /* struct nvme_queue，数量是numa内的CPU个数+1 */
//     dev->queues = kcalloc(num_possible_cpus() + 1, sizeof(void *),
//                                 GFP_KERNEL);
//     if (!dev->queues)
//         goto free;
//     /* unsigned short的数组，每个CPU占一个 */
//     dev->io_queue = alloc_percpu(unsigned short);
//     if (!dev->io_queue)
//         goto free;

//     /* 初始化namespace链表 */
//     INIT_LIST_HEAD(&dev->namespaces);
//     /* reset work的调用函数 */
//     dev->reset_workfn = nvme_reset_failed_dev;
//     INIT_WORK(&dev->reset_work, nvme_reset_workfn);
//     INIT_WORK(&dev->cpu_work, nvme_cpu_workfn);
//     dev->pci_dev = pdev;
//     pci_set_drvdata(pdev, dev);
//     /* 分配一个ID，保存到dev->instance里,实际上第一个加入的nvme设备,它的instance为0,第二个加入的nvme设备,instance为1,以此类推 */
//     result = nvme_set_instance(dev);
//     if (result)
//         goto free;

//     /* 主要创建两个dma pool，一个是4k大小(prp list page)，一个是256B大小(prp list 256) */
//     result = nvme_setup_prp_pools(dev);
//     if (result)
//         goto release;

//     kref_init(&dev->kref);
//     /* 1.做bar空间的映射，映射地址存放到nvme_dev->bar 
//       * 2.当此设备是系统中第一个加载的nvme设备或者nvme_thread没有启动时，就会启动一个nvme_thread
//      * 3.初始化nvme的io queue(主要)
//      */
//     result = nvme_dev_start(dev);
//     if (result) {
//         if (result == -EBUSY)
//             goto create_cdev;
//         goto release_pools;
//     }

//     /* 分配request queue和disk,执行完此函数后,在/dev/下就有此nvme设备了 */
//     result = nvme_dev_add(dev);
//     if (result)
//         goto shutdown;

//  create_cdev:
//      /* 这里开始分配一个对应的混杂设备,可以理解为字符设备,主要用于应用层用ioctl接口来操作此nvme设备 
//       * 这个字符设备的名字为nvme%d
//       */
//     scnprintf(dev->name, sizeof(dev->name), "nvme%d", dev->instance);
//     dev->miscdev.minor = MISC_DYNAMIC_MINOR;
//     dev->miscdev.parent = &pdev->dev;
//     dev->miscdev.name = dev->name;
//     dev->miscdev.fops = &nvme_dev_fops;
//     result = misc_register(&dev->miscdev);
//     if (result)
//         goto remove;

//     dev->initialized = 1;
//     return 0;

//  remove:
//     nvme_dev_remove(dev);
//     nvme_free_namespaces(dev);
//  shutdown:
//     nvme_dev_shutdown(dev);
//  release_pools:
//     nvme_free_queues(dev, 0);
//     nvme_release_prp_pools(dev);
//  release:
//     nvme_release_instance(dev);
//  free:
//     free_percpu(dev->io_queue);
//     kfree(dev->queues);
//     kfree(dev->entry);
//     kfree(dev);
//     return result;
// }

// nvme_probe函数主要做如下几件事情:

// 为中断创建msi/msix的entry,按CPU的数量进行entry的分配,为什么要按照CPU数量进行分配,因为每个io queue会占用一个.而整个系统io queue最大值也就是possible_cpus.
// 分配possible个cpus+1的queue结构体,possible应该是系统最大能够插入的cpu核个数,其不等于online_cpus,注意这里是possible_cpus+1,而中断的msi/msix的entry个数为possible_cpus,而每个queue会用一个entry,这样不是就会导致有一个queue是没有entry用的吗?实际上admin queue和第一个io queue会共用entry0.
// 分配instance,实际上就是一个nvme id,从0开始依次递增.
// 分配两个dma pool,一个pool中的元素大小为4k,一个是256B,这两个pool都是用于数据传输时做dma分配用的.
// 调用nvme_dev_start和nvme_dev_add,这两个是主要函数,之后重点看这两个函数.
// nvme_dev_start和nvme_dev_add是负责不同的初始化,简单点说,nvme_dev_start是将硬件和驱动的联系进行初始化,当nvme_dev_start执行完成后,此nvme设备实际已经能够通过驱动正常使用了,但实际操作系统还是无法使用此设备,原因是需要nvme_dev_add函数将此设备注册到操作系统中,实际就是注册对应的gendisk和request queue,这样在/dev/和操作系统中都能过对此nvme设备进行操作.



// nvme_dev_start
// nvme_dev_start函数主要是做硬件方面与驱动方面的传输通道的初始化和硬件的一些初始化,实际主要就是建立admin queue和io queue,并且为这些queue绑定到各自的irq上.


// /* 1.做bar空间的映射，映射地址存放到nvme_dev->bar 
//  * 2.当此设备是系统中第一个加载的nvme设备或者nvme_thread没有启动时，就会启动一个nvme_thread
//  * 3.初始化nvme的io queue
//  */
// static int nvme_dev_start(struct nvme_dev *dev)
// {
//     int result;
//     bool start_thread = false;

//     /* 主要做bar空间的映射，映射地址存放到nvme_dev->bar,并且从bar空间获取nvme设备的d_queue,d_queue是queue中允许的最大cmd数量 */
//     result = nvme_dev_map(dev);
//     if (result)
//         return result;

//     /* 初始化控制命令队列，中断处理函数为nvme_irq */
//     result = nvme_configure_admin_queue(dev);
//     if (result)
//         goto unmap;

//     spin_lock(&dev_list_lock);
//     /* 当此设备是系统中第一个加载的nvme设备或者nvme_thread没有启动时，就会启动一个nvme_thread */
//     if (list_empty(&dev_list) && IS_ERR_OR_NULL(nvme_thread)) {
//         start_thread = true;
//         nvme_thread = NULL;
//     }
//     list_add(&dev->node, &dev_list);
//     spin_unlock(&dev_list_lock);

//     if (start_thread) {
//         /* 在此nvme设备的加载上下文中创建nvme_thread */
//         nvme_thread = kthread_run(nvme_kthread, NULL, "nvme");
//         wake_up(&nvme_kthread_wait);
//     } else
//         /* 非创建nvme_thread的nvme设备就会在这里等待nvme_thread创建完成 */
//         wait_event_killable(nvme_kthread_wait, nvme_thread);

//     if (IS_ERR_OR_NULL(nvme_thread)) {
//         result = nvme_thread ? PTR_ERR(nvme_thread) : -EINTR;
//         goto disable;
//     }

//     /* 初始化nvme的io queue，此为nvme_queue，一个nvme设备至少一个admin queue，一个io queue */
//     result = nvme_setup_io_queues(dev);
//     if (result && result != -EBUSY)
//         goto disable;

//     return result;

//  disable:
//     nvme_disable_queue(dev, 0);
//     nvme_dev_list_remove(dev);
//  unmap:
//     nvme_dev_unmap(dev);
//     return result;
// }





// 需要注意,d_queue默认是1024,驱动会通过此nvme设备的pci bar空间获取到设备支持的d_queue,并取两者的最小值作为此设备所有queue的d_queue,d_queue是queue中允许存放的cmd数量最大值.

// d_queue获取到后,第一件事情是初始化admin queue,使用nvme_configure_admin_queue:




// /* 初始化控制命令队列，中断处理函数为nvme_irq */
// static int nvme_configure_admin_queue(struct nvme_dev *dev)
// {
//     int result;
//     u32 aqa;
//     u64 cap = readq(&dev->bar->cap);
//     struct nvme_queue *nvmeq;

//     /* 应该是告诉nvme设备禁止操作 
//      * 实现方法是对bar空间的NVME_CC_ENABLEbit做操作,因为当前还没有做irq分配和使用,只能通过寄存器的方法做设置
//      */
//     result = nvme_disable_ctrl(dev, cap);
//     if (result < 0)
//         return result;

//     /* 获取qid为0的nvme queue，实际上就是admin queue */
//     nvmeq = raw_nvmeq(dev, 0);
//     /* 如果不存在，则分配一个nvme queue的内存空间用于admin queue(qid 0) */
//     /* 主要分配cq和sq的dma空间，大小为depth*(struct nvme_completion)，depth*(struct nvme_command) 
//      * 注意sq和cq的dma空间都必须使用dma_alloc_coherent来分配
//      */
//     if (!nvmeq) {
//         nvmeq = nvme_alloc_queue(dev, 0, 64, 0);
//         if (!nvmeq)
//             return -ENOMEM;
//     }

//     aqa = nvmeq->q_depth - 1;
//     aqa |= aqa << 16;

//     dev->ctrl_config = NVME_CC_ENABLE | NVME_CC_CSS_NVM;
//     dev->ctrl_config |= (PAGE_SHIFT - 12) << NVME_CC_MPS_SHIFT;
//     dev->ctrl_config |= NVME_CC_ARB_RR | NVME_CC_SHN_NONE;
//     dev->ctrl_config |= NVME_CC_IOSQES | NVME_CC_IOCQES;

//     /* 初始化sq和cq */
//     writel(aqa, &dev->bar->aqa);
//     writeq(nvmeq->sq_dma_addr, &dev->bar->asq);
//     writeq(nvmeq->cq_dma_addr, &dev->bar->acq);
//     writel(dev->ctrl_config, &dev->bar->cc);

//     /* 应该是告诉nvme设备使能操作 */
//     result = nvme_enable_ctrl(dev, cap);
//     if (result)
//         return result;

//     /* 分配中断，这里主要分配cq的中断，中断处理函数为nvme_irq */
//     result = queue_request_irq(dev, nvmeq, nvmeq->irqname);
//     if (result)
//         return result;

//     spin_lock_irq(&nvmeq->q_lock);
//     /* 初始化cq和sq */
//     nvme_init_queue(nvmeq, 0);
//     spin_unlock_irq(&nvmeq->q_lock);
//     return result;
// }


// /* 分配cq和sq的dma空间，大小为depth*(struct nvme_completion)，depth*(struct nvme_command) */
// static struct nvme_queue *nvme_alloc_queue(struct nvme_dev *dev, int qid,
//                             int depth, int vector)
// {
//     struct device *dmadev = &dev->pci_dev->dev;
//     unsigned extra = nvme_queue_extra(depth);
//     struct nvme_queue *nvmeq = kzalloc(sizeof(*nvmeq) + extra, GFP_KERNEL);
//     if (!nvmeq)
//         return NULL;

//     /* cq的dma区域,存放completion cmd的地方 */
//     nvmeq->cqes = dma_alloc_coherent(dmadev, CQ_SIZE(depth),
//                     &nvmeq->cq_dma_addr, GFP_KERNEL);
//     if (!nvmeq->cqes)
//         goto free_nvmeq;
//     memset((void *)nvmeq->cqes, 0, CQ_SIZE(depth));

//     /* sq的dma区域,存放submission cmd的地方 */
//     nvmeq->sq_cmds = dma_alloc_coherent(dmadev, SQ_SIZE(depth),
//                     &nvmeq->sq_dma_addr, GFP_KERNEL);
//     if (!nvmeq->sq_cmds)
//         goto free_cqdma;

//     if (qid && !zalloc_cpumask_var(&nvmeq->cpu_mask, GFP_KERNEL))
//         goto free_sqdma;

//     nvmeq->q_dmadev = dmadev;
//     nvmeq->dev = dev;
//     snprintf(nvmeq->irqname, sizeof(nvmeq->irqname), "nvme%dq%d",
//             dev->instance, qid);
//     spin_lock_init(&nvmeq->q_lock);
//     nvmeq->cq_head = 0;
//     nvmeq->cq_phase = 1;
//     /* 当sq中的cmdinfo满时,会将进程加入到此waitqueue做等待 */
//     init_waitqueue_head(&nvmeq->sq_full);
//     /* sq_cong_wait是用于加入到sq_full,当sq_full唤醒sq_cong_wait时,实际上是唤醒了nvme_thread */
//     init_waitqueue_entry(&nvmeq->sq_cong_wait, nvme_thread);
//     bio_list_init(&nvmeq->sq_cong);
//     INIT_LIST_HEAD(&nvmeq->iod_bio);
//     /* 当前sq_tail位置，是nvme设备上的一个寄存器，存在于bar空间中 
//      * 发送命令流程: cmd放入sq_cmds,sq_head++,更新sq_head到此q_db,nvme设置会感知到,然后dma sq cmds,并处理sq cmd.
//      */
//     nvmeq->q_db = &dev->dbs[qid * 2 * dev->db_stride];
//     /* 1024或者nvme设备支持的最大值 */
//     nvmeq->q_depth = depth;
//     /* admin queue为0,io queue从0~io queue count */
//     nvmeq->cq_vector = vector;
//     /* queue id, admin queue为0, io queue为1~ io_queue_count+1 */
//     nvmeq->qid = qid;
//     nvmeq->q_suspended = 1;
//     /* nvme设备的queue_count++ */
//     dev->queue_count++;
//     rcu_assign_pointer(dev->queues[qid], nvmeq);

//     return nvmeq;

//  free_sqdma:
//     dma_free_coherent(dmadev, SQ_SIZE(depth), (void *)nvmeq->sq_cmds,
//                             nvmeq->sq_dma_addr);
//  free_cqdma:
//     dma_free_coherent(dmadev, CQ_SIZE(depth), (void *)nvmeq->cqes,
//                             nvmeq->cq_dma_addr);
//  free_nvmeq:
//     kfree(nvmeq);
//     return NULL;
// }



// /* 初始化cq和sq */
// static void nvme_init_queue(struct nvme_queue *nvmeq, u16 qid)
// {
//     struct nvme_dev *dev = nvmeq->dev;
//     /* 大部分情况都是0 */
//     unsigned extra = nvme_queue_extra(nvmeq->q_depth);

//     nvmeq->sq_tail = 0;
//     nvmeq->cq_head = 0;
//     nvmeq->cq_phase = 1;
//     nvmeq->q_db = &dev->dbs[qid * 2 * dev->db_stride];
//     memset(nvmeq->cmdid_data, 0, extra);
//     memset((void *)nvmeq->cqes, 0, CQ_SIZE(nvmeq->q_depth));
//     /* 告诉设备取消处理当前设备中的io请求 */
//     nvme_cancel_ios(nvmeq, false);
//     nvmeq->q_suspended = 0;
//     dev->online_queues++;
// }


// 到这里admin queue已经初始化完成,可以通过对admin queue发送nvme控制命令来操作nvme设置.admin queue初始化完成后的结果如下:

// qid为0就是admin queue,并且nvme_dev->queues[0]就是admin queue.
// nvme_dev->entrys[0]是admin queue使用的.
// admin queue初始化完成后,创建nvme_thread,此内核线程不会在初始化流程中使用,暂时先不看,接下来就是初始化io queue了.

// 初始化io queue是nvme_setup_io_queue函数





// /* 初始化nvme设备的所有io queue */
// static int nvme_setup_io_queues(struct nvme_dev *dev)
// {
//     struct nvme_queue *adminq = raw_nvmeq(dev, 0);
//     struct pci_dev *pdev = dev->pci_dev;
//     int result, i, vecs, nr_io_queues, size;

//     /* 以CPU个数来分配io queue */
//     nr_io_queues = num_possible_cpus();
//     /* 此函数用于设置controller支持的io queue数量(通过发送NVME_FEAT_NUM_QUEUES命令)，nvme driver最优的结果是cpus个数个io queue
//      * 在服务器上nvme设备肯定不会支持那么多io queue，所以设置时controller最多只会设置自己支持的io queue，并返回自己支持的io queue个数
//      * 最后我们选择最小的那个数作为io queue个数，因为也有可能CPU很少，controller支持的io queue很多
//      */
//     result = set_queue_count(dev, nr_io_queues);
//     if (result < 0)
//         return result;
//     if (result < nr_io_queues)
//         nr_io_queues = result;

//     /* 4096 + ((nr_io_queues + 1) * 8 * dev->db_stride) */
//     size = db_bar_size(dev, nr_io_queues);
//     /* size过大，重新映射bar空间 */
//     if (size > 8192) {
//         iounmap(dev->bar);
//         do {
//             dev->bar = ioremap(pci_resource_start(pdev, 0), size);
//             if (dev->bar)
//                 break;
//             if (!--nr_io_queues)
//                 return -ENOMEM;
//             size = db_bar_size(dev, nr_io_queues);
//         } while (1);
//         dev->dbs = ((void __iomem *)dev->bar) + 4096;
//         adminq->q_db = dev->dbs;
//     }

//     /* Deregister the admin queue's interrupt */
//     /* 释放admin queue的irq */
//     free_irq(dev->entry[0].vector, adminq);

//     for (i = 0; i < nr_io_queues; i++)
//         dev->entry[i].entry = i;
//     /* 每个io queue申请一个msix，如果不支持msix，则用msi */
//     vecs = pci_enable_msix_range(pdev, dev->entry, 1, nr_io_queues);
//     if (vecs < 0) {
//         vecs = pci_enable_msi_range(pdev, 1, min(nr_io_queues, 32));
//         if (vecs < 0) {
//             vecs = 1;
//         } else {
//             for (i = 0; i < vecs; i++)
//                 dev->entry[i].vector = i + pdev->irq;
//         }
//     }

//     /*
//      * Should investigate if there's a performance win from allocating
//      * more queues than interrupt vectors; it might allow the submission
//      * path to scale better, even if the receive path is limited by the
//      * number of interrupts.
//      */
//     nr_io_queues = vecs;
//     dev->max_qid = nr_io_queues;

//     /* 重新分配admin queue的irq */
//     result = queue_request_irq(dev, adminq, adminq->irqname);
//     if (result) {
//         adminq->q_suspended = 1;
//         goto free_queues;
//     }

//     /* Free previously allocated queues that are no longer usable */
//     /* 释放多余的io queue */
//     nvme_free_queues(dev, nr_io_queues + 1);
//     /* 分配io queue需要的内存，并且分配对应的irq，最后设置CPU亲和性 */
//     nvme_assign_io_queues(dev);

//     return 0;

//  free_queues:
//     nvme_free_queues(dev, 1);
//     return result;
// }



// /* 分配一个nvme queue，包括其需要的CQ和SQ空间和DMA地址 */
// /* 通过admin queue告知nvme设备创建cq和sq，并且分配对应的irq */
// static void nvme_create_io_queues(struct nvme_dev *dev)
// {
//     unsigned i, max;

//     max = min(dev->max_qid, num_online_cpus());
//     for (i = dev->queue_count; i <= max; i++)
//         /* 分配一个nvme queue，包括其需要的CQ和SQ空间和DMA地址,注意这里第一个io queue使用的entry是0,也就是和admin queue共用 */
//         if (!nvme_alloc_queue(dev, i, dev->q_depth, i - 1))
//             break;

//     max = min(dev->queue_count - 1, num_online_cpus());
//     for (i = dev->online_queues; i <= max; i++)
//         /* 通过admin queue告知nvme设备创建cq和sq，并且分配对应的irq */
//         if (nvme_create_queue(raw_nvmeq(dev, i), i))
//             break;
// }


// static int nvme_create_queue(struct nvme_queue *nvmeq, int qid)
// {
//     struct nvme_dev *dev = nvmeq->dev;
//     int result;

//     /* 通过admin queue将nvme_admin_create_cq命令发送给nvme设备,主要将当前queue的cq_dma地址和qid传给nvme设备,这样就能将cq关联起来 */
//     result = adapter_alloc_cq(dev, qid, nvmeq);
//     if (result < 0)
//         return result;

//     /* 通过admin queue将nvme_admin_create_sq命令发送给nvme设备,主要将当前queue的sq_dma地址和qid传给nvme设备,这样就能将sq关联起来 */
//     result = adapter_alloc_sq(dev, qid, nvmeq);
//     if (result < 0)
//         goto release_cq;

//     /* 为此queue创建一个irq */
//     result = queue_request_irq(dev, nvmeq, nvmeq->irqname);
//     if (result < 0)
//         goto release_sq;

//     spin_lock_irq(&nvmeq->q_lock);
//     nvme_init_queue(nvmeq, qid);
//     spin_unlock_irq(&nvmeq->q_lock);

//     return result;

//  release_sq:
//     adapter_delete_sq(dev, qid);
//  release_cq:
//     adapter_delete_cq(dev, qid);
//     return result;
// }



// /* 分配io queue需要的内存，并且分配对应的irq，最后设置CPU亲和性 */
// static void nvme_assign_io_queues(struct nvme_dev *dev)
// {
//     unsigned cpu, cpus_per_queue, queues, remainder, i;
//     cpumask_var_t unassigned_cpus;

//     /* 分配一个nvme queue，包括其需要的CQ和SQ空间和DMA地址 */
//     /* 告知nvme设备创建cq和sq，并且分配对应的irq */
//     nvme_create_io_queues(dev);

//     /* 获取queue的数量,其至少<=CPUS */
//     queues = min(dev->online_queues - 1, num_online_cpus());
//     if (!queues)
//         return;

//     /* 计算每个io queue的中断可以绑定到多少个CPU上,结果>=1 */
//     cpus_per_queue = num_online_cpus() / queues;
//     /* 剩余的CPU个数,因为queues <= cpus,当queues < cpus时,那么必然有一些queues绑定的cpus比其他的少一个,具体看下面的代码 */
//     remainder = queues - (num_online_cpus() - queues * cpus_per_queue);
//     if (!alloc_cpumask_var(&unassigned_cpus, GFP_KERNEL))
//         return;

//     /* 将所有可用的CPU的mask复制到unassigned_cpus */
//     cpumask_copy(unassigned_cpus, cpu_online_mask);
//     /* 获取第一个可用的CPU */
//     cpu = cpumask_first(unassigned_cpus);
//     /* 遍历所有的io queue,从1开始是因为0是admin queue */
//     for (i = 1; i <= queues; i++) {
//         /* 根据获取对应的io queue */
//         struct nvme_queue *nvmeq = lock_nvmeq(dev, i);
//         cpumask_t mask;

//         /* 清除此io queue的cpumask */
//         cpumask_clear(nvmeq->cpu_mask);
//         /* 如果unassigned_cpus为0,表示没有CPU可以使用,则退出,之后会初始化nvme dev失败 */
//         if (!cpumask_weight(unassigned_cpus)) {
//             unlock_nvmeq(nvmeq);
//             break;
//         }

//         /* 根据CPU ID.获取此CPU的cpumask */
//         mask = *get_cpu_mask(cpu);
//         /* 设置此io queue使用此CPU */
//         nvme_set_queue_cpus(&mask, nvmeq, cpus_per_queue);
//         /* 如果绑定的CPU个数少于cpus_per_queue,那么先绑定此CPU对应的超线程的其他CPU */
//         if (cpus_weight(mask) < cpus_per_queue)
//             nvme_add_cpus(&mask, unassigned_cpus,
//                 topology_thread_cpumask(cpu),
//                 nvmeq, cpus_per_queue);

//         /* 如果绑定的CPU个数还少于cpus_per_queue,那么绑定此CPU对应的同一个socket上其他CPU */
//         if (cpus_weight(mask) < cpus_per_queue)
//             nvme_add_cpus(&mask, unassigned_cpus,
//                 topology_core_cpumask(cpu),
//                 nvmeq, cpus_per_queue);

//         /* 如果绑定的CPU个数还少于cpus_per_queue,那么绑定此CPU对应的node上的所有CPU */
//         if (cpus_weight(mask) < cpus_per_queue)
//             nvme_add_cpus(&mask, unassigned_cpus,
//                 cpumask_of_node(cpu_to_node(cpu)),
//                 nvmeq, cpus_per_queue);

//         /* 如果绑定的CPU个数还少于cpus_per_queue,那么绑定此CPU对应的node最近的node上的所有CPU */
//         if (cpus_weight(mask) < cpus_per_queue)
//             nvme_add_cpus(&mask, unassigned_cpus,
//                 cpumask_of_node(
//                     nvme_find_closest_node(
//                         cpu_to_node(cpu))),
//                 nvmeq, cpus_per_queue);

//         /* 如果绑定的CPU个数还少于cpus_per_queue,那么绑定所有可用的CPU */
//         if (cpus_weight(mask) < cpus_per_queue)
//             nvme_add_cpus(&mask, unassigned_cpus,
//                 unassigned_cpus,
//                 nvmeq, cpus_per_queue);

//         WARN(cpumask_weight(nvmeq->cpu_mask) != cpus_per_queue,
//             "nvme%d qid:%d mis-matched queue-to-cpu assignment\n",
//             dev->instance, i);

//         /* 到这里,已经获取到了此queue对应绑定的CPU的cpumask,并且哪个CPU绑定哪个queue,已经写到nvme_dev->io_queue */
        
//         /* 根据cpumask,设置中断的亲和性 */
//         irq_set_affinity_hint(dev->entry[nvmeq->cq_vector].vector,
//                             nvmeq->cpu_mask);
//         /* 将这些绑定的CPU从unassigned_cpus中删除 */
//         cpumask_andnot(unassigned_cpus, unassigned_cpus,
//                         nvmeq->cpu_mask);
//         /* cpu += 1 */
//         cpu = cpumask_next(cpu, unassigned_cpus);
//         /* 如果此时remainder为0了,那么从下一个queue开始,它绑定的cpus+1,也就是下一个及其之后的queue,绑定的CPUS都比之前的多1 */
//         if (remainder && !--remainder)
//             cpus_per_queue++;
//         unlock_nvmeq(nvmeq);
//     }
//     WARN(cpumask_weight(unassigned_cpus), "nvme%d unassigned online cpus\n",
//                                 dev->instance);
//     i = 0;
//     cpumask_andnot(unassigned_cpus, cpu_possible_mask, cpu_online_mask);
//     /* 如果还有剩余的可用CPU的情况,那么就从queue1开始依次绑到剩余这些CPU上 */
//     for_each_cpu(cpu, unassigned_cpus)
//         *per_cpu_ptr(dev->io_queue, cpu) = (i++ % queues) + 1;
//     free_cpumask_var(unassigned_cpus);
// }



// 一个nvme设备会有多个io queue,每个io queue会有自己的中断,并且nvme设备会将每个io queue的中断绑定到不同的CPU上(实际上并不是真正的做了绑定,注意irq_set_affinity_hint这个函数,它实际上是告知使用者,这个irq更适合在哪些CPU上做处理,但是kernel还是有可能将这个IRQ放到不属于这些CPU中的CPU去处理,不过如果在用户层使用了irqbalance命令,那么irqbalance会将这个IRQ放到这个函数设置的CPU上去处理.具体可以看/proc/irq中的值就能明白了,它改变的是smp_affinity_hint值,而非smp_affinity),就有了上面的代码.一般情况应该是一个io queue绑定到多个CPU上,比如CPU有16个,io queue有8个,那么io queue[0]绑定到cpu0,1上,io queue[1]绑定到cpu2,3上,依次类推.当io queue初始化完成后,一些需要注意的细节如下:

// io queue使用的entry是从0开始的,也就是io queue0会与admin queue共用一个entry.
// nvme_dev->queues是从1开始保存io queue.
// queue的sq_dma,cq_dma和qid通过admin queue发送给nvme设备,nvme设备会将其做绑定.并且注意,在nvme_alloc_queue时,queue->q_db指向的位置是通过qid计算的,所以实际上,sq_dma,cq_dma,qid和q_db都能过联系起来了.
// 到这里,admin queue和io queue都初始化完成了,之后就是在块层注册设备的操作.

// nvme_add_dev




// static int nvme_dev_add(struct nvme_dev *dev)
// {
//     struct pci_dev *pdev = dev->pci_dev;
//     int res;
//     unsigned nn, i;
//     struct nvme_ns *ns;
//     struct nvme_id_ctrl *ctrl;
//     struct nvme_id_ns *id_ns;
//     void *mem;
//     dma_addr_t dma_addr;
//     int shift = NVME_CAP_MPSMIN(readq(&dev->bar->cap)) + 12;

//     /* 分配一个一致性dma区域,注意大小是8192B,前4096B放盘的信息,后面4096B空闲,之后会使用 */
//     mem = dma_alloc_coherent(&pdev->dev, 8192, &dma_addr, GFP_KERNEL);
//     if (!mem)
//         return -ENOMEM;

//     /* 向controller发送一个identify命令,此命令会让controller将nvme卡的信息保存到mem这块一致性dma区域中 */
//     res = nvme_identify(dev, 0, 1, dma_addr);
//     if (res) {
//         dev_err(&pdev->dev, "Identify Controller failed (%d)\n", res);
//         res = -EIO;
//         goto out;
//     }

//     /* 已经获取到信息,包括sn号,model,fw版本,用户可用容量等信息,注意,nn是表示此nvme物理盘生成多少个块设备 */
//     ctrl = mem;
//     /* 决定了生成多少个块设备 */
//     nn = le32_to_cpup(&ctrl->nn);
//     dev->oncs = le16_to_cpup(&ctrl->oncs);
//     dev->abort_limit = ctrl->acl + 1;
//     dev->vwc = ctrl->vwc;
//     memcpy(dev->serial, ctrl->sn, sizeof(ctrl->sn));
//     memcpy(dev->model, ctrl->mn, sizeof(ctrl->mn));
//     memcpy(dev->firmware_rev, ctrl->fr, sizeof(ctrl->fr));
//     if (ctrl->mdts)
//         dev->max_hw_sectors = 1 << (ctrl->mdts + shift - 9);
//     if ((pdev->vendor == PCI_VENDOR_ID_INTEL) &&
//             (pdev->device == 0x0953) && ctrl->vs[3])
//         dev->stripe_size = 1 << (ctrl->vs[3] + shift);

//     id_ns = mem;
//     for (i = 1; i <= nn; i++) {
//         res = nvme_identify(dev, i, 0, dma_addr);
//         if (res)
//             continue;

//         if (id_ns->ncap == 0)
//             continue;

//         /* 通过admin queue获取设备盘容量,lba大小等信息,存放到mem的后4096B中 */
//         res = nvme_get_features(dev, NVME_FEAT_LBA_RANGE, i,
//                             dma_addr + 4096, NULL);
//         if (res)
//             memset(mem + 4096, 0, 4096);

//         /* 分配disk和request queue,一个块设备就是一个namespace */
//         ns = nvme_alloc_ns(dev, i, mem, mem + 4096);
//         /* 加入到nvme_dev->namespace链表中 */
//         if (ns)
//             list_add_tail(&ns->list, &dev->namespaces);
//     }
//     /* 将disk添加到系统中,这样用户就能在/dev/下面看到了 */
//     list_for_each_entry(ns, &dev->namespaces, list)
//         add_disk(ns->disk);
//     res = 0;

//  out:
//     dma_free_coherent(&dev->pci_dev->dev, 8192, mem, dma_addr);
//     return res;
// }



// 此函数主要做几件事情:

// 获取nvme设备的信息.
// 根据nvme设备的信息,创建对应的namespace,一个namespace实际就是一个块设备
// 将创建的namespace加入到系统中的块设备中.
// 主要是通过nvme_alloc_ns函数来初始化一个namespace,一个namespace是一个块设备,一个块设备主要初始化两个结构,一个是gendisk,一个是request queue,当两个结构都初始化好后,调用add_disk()函数,这个块设备就会正式加入到系统中的块设备中.



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



// 这里主要初始化gendisk和request queue,gendisk用于描述一个块设备,也就是当gendisk初始化好后,并调用add_disk(),就会在/dev/下出现一个此gendisk->name的块设备.而request_queue有什么用呢,注意看gendisk初始化时,会将gendisk->queue设置为一个初始化好的request_queue.对于request_queue,最重要的是初始化一个make_request_fn的函数指针,当有进程对此gendisk对应的块设备进行读写时,最终都会调用到gendisk的request_queue的make_request_fn所指的函数.在nvme驱动中,主要将request_queue的make_request_fn初始化为了nvme_make_request()函数,未来在说nvme设备的读写流程时,会详细说明此函数.







// //! Device wrappers that implement `rcore_fs::dev::Device`, which can loaded
// //! file systems on (e.g. `rcore_fs_sfs::SimpleFileSystem::open()`).

// use alloc::sync::Arc;

// extern crate rcore_fs;

// use kernel_hal::drivers::scheme::BlockScheme;
// use lock::RwLock;
// use rcore_fs::dev::{BlockDevice, DevError, Device, Result};

// /// A naive LRU cache layer for `BlockDevice`, re-exported from `rcore-fs`.
// pub use rcore_fs::dev::block_cache::BlockCache;

// /// Memory buffer for device.
// pub struct NvmeBuf();

// impl NvmeBuf {
//     /// create a [`NvmeBuf`] struct.
//     pub fn new() -> Self {

//     }
// }

// impl Device for NvmeBuf {
//     fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
//         let slice = self.0.read();
//         let len = buf.len().min(slice.len() - offset);
//         buf[..len].copy_from_slice(&slice[offset..offset + len]);
//         Ok(len)
//     }
//     fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize> {
//         let mut slice = self.0.write();
//         let len = buf.len().min(slice.len() - offset);
//         slice[offset..offset + len].copy_from_slice(&buf[..len]);
//         Ok(len)
//     }
//     fn sync(&self) -> Result<()> {
//         Ok(())
//     }
// }

// /// Block device implements [`BlockScheme`].
// pub struct Block(Arc<dyn BlockScheme>);

// impl Block {
//     /// create a [`Block`] struct.
//     pub fn new(block: Arc<dyn BlockScheme>) -> Self {
//         Self(block)
//     }
// }

// impl BlockDevice for Block {
//     const BLOCK_SIZE_LOG2: u8 = 9; // 512

//     fn read_at(&self, block_id: usize, buf: &mut [u8]) -> Result<()> {
//         self.0.read_block(block_id, buf).map_err(|_| DevError)
//     }

//     fn write_at(&self, block_id: usize, buf: &[u8]) -> Result<()> {
//         self.0.write_block(block_id, buf).map_err(|_| DevError)
//     }

//     fn sync(&self) -> Result<()> {
//         self.0.flush().map_err(|_| DevError)
//     }
// }



// pub struct SubmissionQueue{

// }

// pub struct CompletionQueue{

// }

// pub struct DoorbellRegister{

// }



// // https://blog.csdn.net/wangpeng22/article/details/73930872

// /**
// *struct udevice -驱动程序实例
// *
// *这包含有关设备的信息，该设备是绑定到
// *特定端口或外围设备（本质上是驱动程序实例）。
// *
// *设备将通过“绑定”调用而存在，或者是由于
// *一个 U_BOOT_DRVINFO() 宏（在这种情况下 plat 非空）或一个节点
// *在设备树中（在这种情况下 of_offset >= 0）。在后一种情况下
// *我们将设备树信息翻译成函数中的 plat
// *由驱动程序 of_to_plat 方法实现（在
// *如果设备有设备树节点，则探测方法。
// *
// *plat、priv 和 uclass_priv 这三个都可以由
// *驱动程序，或者您可以使用结构驱动程序的自动成员和
// *struct uclass_driver 让驱动程序模型自动执行此操作。
// *
// *@driver: 此设备使用的驱动程序
// *@name：设备名称，通常为 FDT 节点名称
// *@plat_：此设备的配置数据（不要访问外部驱动程序
// *	模型）
// *@parent_plat_：此设备的父总线配置数据（不要
// *访问外部驱动程序模型）
// *@uclass_plat_: 此设备的 uclass 配置数据（请勿访问
// *外部驱动模型）
// *@driver_data：与此设备匹配的条目的驱动程序数据字
// *它的驱动程序
// *@parent：此设备的父级，或顶级设备为 NULL
// *@priv_：此设备的私有数据（请勿访问外部驱动程序模型）
// *@uclass: 指向此设备的 uclass 的指针
// *@uclass_priv_：此设备的 uclass 私有数据（请勿访问
// *外部驱动模型）
// *@parent_priv_：此设备的父母的私人数据（请勿访问
// *外部驱动模型）
// *@uclass_node: uclass 用来链接它的设备
// *@child_head: 这个设备的孩子列表
// *@sibling_node: 所有设备列表中的下一个设备
// *@flags_：此设备的标志 `DM_FLAG_...`（不要访问外部驱动程序
// *	模型）
// *@seq_：为此设备分配的序列号（-1 = 无）。这是设置的
// *当设备被绑定并且在设备的 uclass 中是唯一的。如果
// *设备在设备树中有一个别名，用于设置序列
// *数字。否则，使用下一个可用号码。序号是
// *由需要设备编号的某些命令使用（例如“mmc dev”）。
// *（请勿访问外部驱动程序模型）
// *@node_: 引用此设备的设备树节点（请勿在外部访问
// *驱动器型号）
// *@devres_head：与此设备关联的内存分配列表。
// *当启用 CONFIG_DEVRES 时，devm_kmalloc() 和朋友会
// *添加到此列表。如此分配的内存将被释放
// *当设备被移除/解除绑定时自动
// *@dma_offset：物理地址空间（CPU）和
// *设备的总线地址空间
// */
// struct udevice {
// 	const struct driver *driver;
// 	const char *name;
// 	void *plat_;
// 	void *parent_plat_;
// 	void *uclass_plat_;
// 	ulong driver_data;
// 	struct udevice *parent;
// 	void *priv_;
// 	struct uclass *uclass;
// 	void *uclass_priv_;
// 	void *parent_priv_;
// 	struct list_head uclass_node;
// 	struct list_head child_head;
// 	struct list_head sibling_node;
// #if !CONFIG_IS_ENABLED(OF_PLATDATA_RT)
// 	u32 flags_;
// #endif
// 	int seq_;
// #if CONFIG_IS_ENABLED(OF_REAL)
// 	ofnode node_;
// #endif
// #if CONFIG_IS_ENABLED(DEVRES)
// 	struct list_head devres_head;
// #endif
// #if CONFIG_IS_ENABLED(DM_DMA)
// 	ulong dma_offset;
// #endif
// };


// static int nvme_probe(struct udevice *udev)
// {
// 	struct nvme_dev *ndev = dev_get_priv(udev);
// 	struct pci_child_plat *pplat;

// 	pplat = dev_get_parent_plat(udev);
// 	sprintf(ndev->vendor, "0x%.4x", pplat->vendor);

// 	ndev->instance = trailing_strtol(udev->name);
// 	ndev->bar = dm_pci_map_bar(udev, PCI_BASE_ADDRESS_0, 0, 0,
// 				   PCI_REGION_TYPE, PCI_REGION_MEM);
// 	return nvme_init(udev);
// }



// int nvme_init(struct udevice *udev)
// {
// 	struct nvme_dev *ndev = dev_get_priv(udev);
// 	struct nvme_id_ns *id;
// 	int ret;

// 	ndev->udev = udev;
// 	INIT_LIST_HEAD(&ndev->namespaces);
// 	if (readl(&ndev->bar->csts) == -1) {
// 		ret = -ENODEV;
// 		printf("Error: %s: Out of memory!\n", udev->name);
// 		goto free_nvme;
// 	}

// 	ndev->queues = malloc(NVME_Q_NUM * sizeof(struct nvme_queue *));
// 	if (!ndev->queues) {
// 		ret = -ENOMEM;
// 		printf("Error: %s: Out of memory!\n", udev->name);
// 		goto free_nvme;
// 	}
// 	memset(ndev->queues, 0, NVME_Q_NUM * sizeof(struct nvme_queue *));

// 	ndev->cap = nvme_readq(&ndev->bar->cap);
// 	ndev->q_depth = min_t(int, NVME_CAP_MQES(ndev->cap) + 1, NVME_Q_DEPTH);
// 	ndev->db_stride = 1 << NVME_CAP_STRIDE(ndev->cap);
// 	ndev->dbs = ((void __iomem *)ndev->bar) + 4096;

// 	ret = nvme_configure_admin_queue(ndev);
// 	if (ret)
// 		goto free_queue;

// 	/* Allocate after the page size is known */
// 	ndev->prp_pool = memalign(ndev->page_size, MAX_PRP_POOL);
// 	if (!ndev->prp_pool) {
// 		ret = -ENOMEM;
// 		printf("Error: %s: Out of memory!\n", udev->name);
// 		goto free_nvme;
// 	}
// 	ndev->prp_entry_num = MAX_PRP_POOL >> 3;

// 	ret = nvme_setup_io_queues(ndev);
// 	if (ret)
// 		goto free_queue;

// 	nvme_get_info_from_identify(ndev);

// 	/* Create a blk device for each namespace */

// 	id = memalign(ndev->page_size, sizeof(struct nvme_id_ns));
// 	if (!id) {
// 		ret = -ENOMEM;
// 		goto free_queue;
// 	}

// 	for (int i = 1; i <= ndev->nn; i++) {
// 		struct udevice *ns_udev;
// 		char name[20];

// 		memset(id, 0, sizeof(*id));
// 		if (nvme_identify(ndev, i, 0, (dma_addr_t)(long)id)) {
// 			ret = -EIO;
// 			goto free_id;
// 		}

// 		/* skip inactive namespace */
// 		if (!id->nsze)
// 			continue;

// 		/*
// 		 * Encode the namespace id to the device name so that
// 		 * we can extract it when doing the probe.
// 		 */
// 		sprintf(name, "blk#%d", i);

// 		/* The real blksz and size will be set by nvme_blk_probe() */
// 		ret = blk_create_devicef(udev, "nvme-blk", name, IF_TYPE_NVME,
// 					 -1, 512, 0, &ns_udev);
// 		if (ret)
// 			goto free_id;

// 		ret = blk_probe_or_unbind(ns_udev);
// 		if (ret)
// 			goto free_id;
// 	}

// 	free(id);
// 	return 0;

// free_id:
// 	free(id);
// free_queue:
// 	free((void *)ndev->queues);
// free_nvme:
// 	return ret;
// }



// #ifdef CONFIG_NVME_VERBOSE_ERRORS
// static const char * const nvme_ops[] = {
// 	[nvme_cmd_flush] = "Flush",
// 	[nvme_cmd_write] = "Write",
// 	[nvme_cmd_read] = "Read",
// 	[nvme_cmd_write_uncor] = "Write Uncorrectable",
// 	[nvme_cmd_compare] = "Compare",
// 	[nvme_cmd_write_zeroes] = "Write Zeros",
// 	[nvme_cmd_dsm] = "Dataset Management",
// 	[nvme_cmd_verify] = "Verify",
// 	[nvme_cmd_resv_register] = "Reservation Register",
// 	[nvme_cmd_resv_report] = "Reservation Report",
// 	[nvme_cmd_resv_acquire] = "Reservation Acquire",
// 	[nvme_cmd_resv_release] = "Reservation Release",
// 	[nvme_cmd_zone_mgmt_send] = "Zone Management Send",
// 	[nvme_cmd_zone_mgmt_recv] = "Zone Management Receive",
// 	[nvme_cmd_zone_append] = "Zone Management Append",
// };

// static const char * const nvme_admin_ops[] = {
// 	[nvme_admin_delete_sq] = "Delete SQ",
// 	[nvme_admin_create_sq] = "Create SQ",
// 	[nvme_admin_get_log_page] = "Get Log Page",
// 	[nvme_admin_delete_cq] = "Delete CQ",
// 	[nvme_admin_create_cq] = "Create CQ",
// 	[nvme_admin_identify] = "Identify",
// 	[nvme_admin_abort_cmd] = "Abort Command",
// 	[nvme_admin_set_features] = "Set Features",
// 	[nvme_admin_get_features] = "Get Features",
// 	[nvme_admin_async_event] = "Async Event",
// 	[nvme_admin_ns_mgmt] = "Namespace Management",
// 	[nvme_admin_activate_fw] = "Activate Firmware",
// 	[nvme_admin_download_fw] = "Download Firmware",
// 	[nvme_admin_dev_self_test] = "Device Self Test",
// 	[nvme_admin_ns_attach] = "Namespace Attach",
// 	[nvme_admin_keep_alive] = "Keep Alive",
// 	[nvme_admin_directive_send] = "Directive Send",
// 	[nvme_admin_directive_recv] = "Directive Receive",
// 	[nvme_admin_virtual_mgmt] = "Virtual Management",
// 	[nvme_admin_nvme_mi_send] = "NVMe Send MI",
// 	[nvme_admin_nvme_mi_recv] = "NVMe Receive MI",
// 	[nvme_admin_dbbuf] = "Doorbell Buffer Config",
// 	[nvme_admin_format_nvm] = "Format NVM",
// 	[nvme_admin_security_send] = "Security Send",
// 	[nvme_admin_security_recv] = "Security Receive",
// 	[nvme_admin_sanitize_nvm] = "Sanitize NVM",
// 	[nvme_admin_get_lba_status] = "Get LBA Status",
// };

// static const char * const nvme_statuses[] = {
// 	[NVME_SC_SUCCESS] = "Success",
// 	[NVME_SC_INVALID_OPCODE] = "Invalid Command Opcode",
// 	[NVME_SC_INVALID_FIELD] = "Invalid Field in Command",
// 	[NVME_SC_CMDID_CONFLICT] = "Command ID Conflict",
// 	[NVME_SC_DATA_XFER_ERROR] = "Data Transfer Error",
// 	[NVME_SC_POWER_LOSS] = "Commands Aborted due to Power Loss Notification",
// 	[NVME_SC_INTERNAL] = "Internal Error",
// 	[NVME_SC_ABORT_REQ] = "Command Abort Requested",
// 	[NVME_SC_ABORT_QUEUE] = "Command Aborted due to SQ Deletion",
// 	[NVME_SC_FUSED_FAIL] = "Command Aborted due to Failed Fused Command",
// 	[NVME_SC_FUSED_MISSING] = "Command Aborted due to Missing Fused Command",
// 	[NVME_SC_INVALID_NS] = "Invalid Namespace or Format",
// 	[NVME_SC_CMD_SEQ_ERROR] = "Command Sequence Error",
// 	[NVME_SC_SGL_INVALID_LAST] = "Invalid SGL Segment Descriptor",
// 	[NVME_SC_SGL_INVALID_COUNT] = "Invalid Number of SGL Descriptors",
// 	[NVME_SC_SGL_INVALID_DATA] = "Data SGL Length Invalid",
// 	[NVME_SC_SGL_INVALID_METADATA] = "Metadata SGL Length Invalid",
// 	[NVME_SC_SGL_INVALID_TYPE] = "SGL Descriptor Type Invalid",
// 	[NVME_SC_CMB_INVALID_USE] = "Invalid Use of Controller Memory Buffer",
// 	[NVME_SC_PRP_INVALID_OFFSET] = "PRP Offset Invalid",
// 	[NVME_SC_ATOMIC_WU_EXCEEDED] = "Atomic Write Unit Exceeded",
// 	[NVME_SC_OP_DENIED] = "Operation Denied",
// 	[NVME_SC_SGL_INVALID_OFFSET] = "SGL Offset Invalid",
// 	[NVME_SC_RESERVED] = "Reserved",
// 	[NVME_SC_HOST_ID_INCONSIST] = "Host Identifier Inconsistent Format",
// 	[NVME_SC_KA_TIMEOUT_EXPIRED] = "Keep Alive Timeout Expired",
// 	[NVME_SC_KA_TIMEOUT_INVALID] = "Keep Alive Timeout Invalid",
// 	[NVME_SC_ABORTED_PREEMPT_ABORT] = "Command Aborted due to Preempt and Abort",
// 	[NVME_SC_SANITIZE_FAILED] = "Sanitize Failed",
// 	[NVME_SC_SANITIZE_IN_PROGRESS] = "Sanitize In Progress",
// 	[NVME_SC_SGL_INVALID_GRANULARITY] = "SGL Data Block Granularity Invalid",
// 	[NVME_SC_CMD_NOT_SUP_CMB_QUEUE] = "Command Not Supported for Queue in CMB",
// 	[NVME_SC_NS_WRITE_PROTECTED] = "Namespace is Write Protected",
// 	[NVME_SC_CMD_INTERRUPTED] = "Command Interrupted",
// 	[NVME_SC_TRANSIENT_TR_ERR] = "Transient Transport Error",
// 	[NVME_SC_ADMIN_COMMAND_MEDIA_NOT_READY] = "Admin Command Media Not Ready",
// 	[NVME_SC_INVALID_IO_CMD_SET] = "Invalid IO Command Set",
// 	[NVME_SC_LBA_RANGE] = "LBA Out of Range",
// 	[NVME_SC_CAP_EXCEEDED] = "Capacity Exceeded",
// 	[NVME_SC_NS_NOT_READY] = "Namespace Not Ready",
// 	[NVME_SC_RESERVATION_CONFLICT] = "Reservation Conflict",
// 	[NVME_SC_FORMAT_IN_PROGRESS] = "Format In Progress",
// 	[NVME_SC_CQ_INVALID] = "Completion Queue Invalid",
// 	[NVME_SC_QID_INVALID] = "Invalid Queue Identifier",
// 	[NVME_SC_QUEUE_SIZE] = "Invalid Queue Size",
// 	[NVME_SC_ABORT_LIMIT] = "Abort Command Limit Exceeded",
// 	[NVME_SC_ABORT_MISSING] = "Reserved", /* XXX */
// 	[NVME_SC_ASYNC_LIMIT] = "Asynchronous Event Request Limit Exceeded",
// 	[NVME_SC_FIRMWARE_SLOT] = "Invalid Firmware Slot",
// 	[NVME_SC_FIRMWARE_IMAGE] = "Invalid Firmware Image",
// 	[NVME_SC_INVALID_VECTOR] = "Invalid Interrupt Vector",
// 	[NVME_SC_INVALID_LOG_PAGE] = "Invalid Log Page",
// 	[NVME_SC_INVALID_FORMAT] = "Invalid Format",
// 	[NVME_SC_FW_NEEDS_CONV_RESET] = "Firmware Activation Requires Conventional Reset",
// 	[NVME_SC_INVALID_QUEUE] = "Invalid Queue Deletion",
// 	[NVME_SC_FEATURE_NOT_SAVEABLE] = "Feature Identifier Not Saveable",
// 	[NVME_SC_FEATURE_NOT_CHANGEABLE] = "Feature Not Changeable",
// 	[NVME_SC_FEATURE_NOT_PER_NS] = "Feature Not Namespace Specific",
// 	[NVME_SC_FW_NEEDS_SUBSYS_RESET] = "Firmware Activation Requires NVM Subsystem Reset",
// 	[NVME_SC_FW_NEEDS_RESET] = "Firmware Activation Requires Reset",
// 	[NVME_SC_FW_NEEDS_MAX_TIME] = "Firmware Activation Requires Maximum Time Violation",
// 	[NVME_SC_FW_ACTIVATE_PROHIBITED] = "Firmware Activation Prohibited",
// 	[NVME_SC_OVERLAPPING_RANGE] = "Overlapping Range",
// 	[NVME_SC_NS_INSUFFICIENT_CAP] = "Namespace Insufficient Capacity",
// 	[NVME_SC_NS_ID_UNAVAILABLE] = "Namespace Identifier Unavailable",
// 	[NVME_SC_NS_ALREADY_ATTACHED] = "Namespace Already Attached",
// 	[NVME_SC_NS_IS_PRIVATE] = "Namespace Is Private",
// 	[NVME_SC_NS_NOT_ATTACHED] = "Namespace Not Attached",
// 	[NVME_SC_THIN_PROV_NOT_SUPP] = "Thin Provisioning Not Supported",
// 	[NVME_SC_CTRL_LIST_INVALID] = "Controller List Invalid",
// 	[NVME_SC_SELT_TEST_IN_PROGRESS] = "Device Self-test In Progress",
// 	[NVME_SC_BP_WRITE_PROHIBITED] = "Boot Partition Write Prohibited",
// 	[NVME_SC_CTRL_ID_INVALID] = "Invalid Controller Identifier",
// 	[NVME_SC_SEC_CTRL_STATE_INVALID] = "Invalid Secondary Controller State",
// 	[NVME_SC_CTRL_RES_NUM_INVALID] = "Invalid Number of Controller Resources",
// 	[NVME_SC_RES_ID_INVALID] = "Invalid Resource Identifier",
// 	[NVME_SC_PMR_SAN_PROHIBITED] = "Sanitize Prohibited",
// 	[NVME_SC_ANA_GROUP_ID_INVALID] = "ANA Group Identifier Invalid",
// 	[NVME_SC_ANA_ATTACH_FAILED] = "ANA Attach Failed",
// 	[NVME_SC_BAD_ATTRIBUTES] = "Conflicting Attributes",
// 	[NVME_SC_INVALID_PI] = "Invalid Protection Information",
// 	[NVME_SC_READ_ONLY] = "Attempted Write to Read Only Range",
// 	[NVME_SC_ONCS_NOT_SUPPORTED] = "ONCS Not Supported",
// 	[NVME_SC_ZONE_BOUNDARY_ERROR] = "Zoned Boundary Error",
// 	[NVME_SC_ZONE_FULL] = "Zone Is Full",
// 	[NVME_SC_ZONE_READ_ONLY] = "Zone Is Read Only",
// 	[NVME_SC_ZONE_OFFLINE] = "Zone Is Offline",
// 	[NVME_SC_ZONE_INVALID_WRITE] = "Zone Invalid Write",
// 	[NVME_SC_ZONE_TOO_MANY_ACTIVE] = "Too Many Active Zones",
// 	[NVME_SC_ZONE_TOO_MANY_OPEN] = "Too Many Open Zones",
// 	[NVME_SC_ZONE_INVALID_TRANSITION] = "Invalid Zone State Transition",
// 	[NVME_SC_WRITE_FAULT] = "Write Fault",
// 	[NVME_SC_READ_ERROR] = "Unrecovered Read Error",
// 	[NVME_SC_GUARD_CHECK] = "End-to-end Guard Check Error",
// 	[NVME_SC_APPTAG_CHECK] = "End-to-end Application Tag Check Error",
// 	[NVME_SC_REFTAG_CHECK] = "End-to-end Reference Tag Check Error",
// 	[NVME_SC_COMPARE_FAILED] = "Compare Failure",
// 	[NVME_SC_ACCESS_DENIED] = "Access Denied",
// 	[NVME_SC_UNWRITTEN_BLOCK] = "Deallocated or Unwritten Logical Block",
// 	[NVME_SC_INTERNAL_PATH_ERROR] = "Internal Pathing Error",
// 	[NVME_SC_ANA_PERSISTENT_LOSS] = "Asymmetric Access Persistent Loss",
// 	[NVME_SC_ANA_INACCESSIBLE] = "Asymmetric Access Inaccessible",
// 	[NVME_SC_ANA_TRANSITION] = "Asymmetric Access Transition",
// 	[NVME_SC_CTRL_PATH_ERROR] = "Controller Pathing Error",
// 	[NVME_SC_HOST_PATH_ERROR] = "Host Pathing Error",
// 	[NVME_SC_HOST_ABORTED_CMD] = "Host Aborted Command",
// };




//     /* 这里开始分配一个对应的混杂设备,可以理解为字符设备,主要用于应用层用ioctl接口来操作此nvme设备 
//       * 这个字符设备的名字为nvme%d