
# pci设备
https://medium.com/@michael2012zhao_67085/understanding-pci-node-in-fdt-769a894a13cc


# BAR是pcie设备上的控制器提供给os的一组寄存器.  用来接收命令
bar
NVMe驱动解析-关键的BAR空间 https://mp.weixin.qq.com/s/mCm7rDpprAY6M8bdFpxmJA
**http://www.ssdfans.com/?p=8210

http://www.ssdfans.com/?p=8171
http://www.ssdfans.com/?p=8171
http://www.ssdfans.com/?p=8210


一个PCIe设备，可能有若干个内部空间（属性可能不一样，比如有些可预读，有些不可预读）需要映射到内存空间，设备出厂时，这些空间的大小和属性都写在Configuration BAR寄存器里面，然后上电后，
系统软件读取这些BAR，分别为其分配对应的系统内存空间，并把相应的内存基地址写回到BAR。（BAR的地址其实是PCI总线域的地址，CPU访问的是存储器域的地址，CPU访问PCIe设备时，需要把总线域地址转换成存储器域的地址。）


设备内存用page划分
Physical Region Page

# prp
用一个简单的例子窥探NVMe的PRP规则 https://mp.weixin.qq.com/s/9oFnJ9JWmGIh-mgVz3jk4Q
http://www.ssdfans.com/?p=8173
http://www.ssdfans.com/?p=8141




# linux 块设备驱动
https://www.bilibili.com/read/cv17063262



# NVMe驱动解析-响应I/O请求
https://mp.weixin.qq.com/s?__biz=MzIyNDU0ODk4OA==&mid=2247483711&idx=1&sn=726890a3d3729d5b688a1f51a95900e5&chksm=e80c002cdf7b893a6cce50fd5387d10e3ebdbf49804d89d37c79315b7e7d5279b6759d361ccf&scene=126&&sessionid=1662083002#rd

## Device-to-device memory-transfer offload with P2PDMA
https://lwn.net/Articles/767281/

PCI devices expose memory to the host system in form of memory regions defined by base address registers (BARs). 
Those are regions mapped into the host's physical memory space. 
All regions are mapped into the same address space, and PCI DMA operations can use those addresses directly.
It is thus possible for a driver to configure a PCI DMA operation to perform transfers between the memory zones of two devices while bypassing system memory completely. 

# linux地址空间  pcie dma
https://www.oreilly.com/library/view/linux-device-drivers/0596005903/ch15.html
NVMe驱动解析-DMA传输 https://mp.weixin.qq.com/s/iF6LHniCjYCZ1kAnw3x9cQ


Host如果想往SSD上写入用户数据，需要告诉SSD写入什么数据，

写入多少数据，以及数据源在内存中的什么位置，这些信息包含在Host向SSD发送的Write命令中。

每笔用户数据对应着一个叫做LBA（Logical Block Address）的东西，Write命令通过指定LBA来告诉SSD写入的是什么数据。

对NVMe/PCIe来说，SSD收到Write命令后，通过PCIe去Host的内存数据所在位置读取数据，然后把这些数据写入到闪存中，同时得到LBA与闪存位置的映射关系。





但是，还有一个问题，这个Admin Command是怎么传过去的呢？还是要看NVMe Spec。之前提到的NVMe的BAR空间中就有这么两个寄存器，它们用来存储Admin Queue 的 Command DMA基地址。
图片
如下，在创建Admin Queue的时候就向Controller写入DMA地址：



# Doorbellregister
SQ位于Host内存中，Host要发送命令时，先把准备好的命令放在SQ中，然后通知SSD来取；
CQ也是位于Host内存中，一个命令执行完成，成功或失败，SSD总会往CQ中写入命令完成状态。
DB又是干什么用的呢？Host发送命令时，不是直接往SSD中发送命令的，而是把命令准备好放在自己的内存中，
那怎么通知SSD来获取命令执行呢？
Host就是通过写SSD端的DB寄存器来告知SSD的

SQ = Submission Queue
CQ = Completion Queue
DB = Doorbell Register

第一步：Host写命令到SQ；

第二步：Host写DB，通知SSD取指；

第三步：SSD收到通知，于是从SQ中取指；

第四步：SSD执行指令；

第五步：指令执行完成，SSD往CQ中写指令执行结果；

第六步：然后SSD发起中断通知Host指令完成；

第七步：收到中断，Host处理CQ，查看指令完成状态；

第八步：Host处理完CQ中的指令执行结果，通过DB回复SSD：指令执行结果已处理，辛苦您了！



host往sq1中写入3个命令, sq1.tail=3, qs DBR = 3, 

执行完2个命令, cq DBR=2


db记录了sq 和 cq 的头和尾

ssd 控制器知道sq的head位置

host知道sq的tail位置

SSD往CQ中写入命令状态信息的同时，还把SQ Head DB的信息告知了Host

cq host 知道head 不知道tail
一开始cq中每条命令完成条目中的 p bit初始化为0, ssd在往cq中写入命令完成条目是p bit置为1, host在处理cq中的命令完成条目时, p bit置为0,
cq是在host的内存中, hist记住上次的tail, 检查p 得出新的tail




# nvme设备初始化

参考 <https://blog.csdn.net/yiyeguzhou100/article/details/105478124>

## 1. 创建admin queue

linux 5.19

```c
static int nvme_pci_configure_admin_queue(struct nvme_dev *dev)

```


u-boot

```c
static int nvme_configure_admin_queue(struct nvme_dev *dev)
{
}

struct nvme_bar {
	__u64 cap;	/* Controller Capabilities */
	__u32 vs;	/* Version */
	__u32 intms;	/* Interrupt Mask Set */
	__u32 intmc;	/* Interrupt Mask Clear */
	__u32 cc;	/* Controller Configuration */
	__u32 rsvd1;	/* Reserved */
	__u32 csts;	/* Controller Status */
	__u32 rsvd2;	/* Reserved */
	__u32 aqa;	/* Admin Queue Attributes */
	__u64 asq;	/* Admin SQ Base Address */
	__u64 acq;	/* Admin CQ Base Address */
};
```


# 更多参考

<https://blog.csdn.net/panzhenjie/article/details/51581063>
<https://nvmexpress.org/developers/nvme-specification/>




/*
Doorbell  Stride  (DSTRD):  Each  Submission  Queue  and  Completion  Queue  
Doorbell  register  is  32-bits  in  size
This  register  indicates  the  stride  between  
doorbell registers. The stride is specified as (2 ^ (2 + DSTRD)) in bytes. A value 
of 0h indicates a stride of 4 bytes, where the doorbell registers are packed without 
reserved space between each register. 
*/










pci_register_host_bridge

    --pci_setup_device




pci_assign_resource
--------pci_scan_device--------
[    0.336150] pci_scan_child_bus_extend scanning bus



pci_assign_resource

_pci_assign_resource

__pci_assign_resource


assign_requested_resources_sorted
bus_for_each_dev



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




















    // pub fn init(&mut self, bar: usize, len:usize) {
    
    //     let nvme_version = unsafe{
    //         read_volatile((bar + NVME_REG_VS) as *const u32)
    //     };

    //     warn!("nvme version: {:?}", nvme_version);

    //     //第一步在pci扫描到设备时已经完成

    //     //第二步 设置admin queue,包括其需要的CQ和SQ空间和DMA地址
    //     // let nvme: Nvme<ProviderImpl> = super::Nvme::new();

    //     // // let q_db = dbs[qid * 2 * db_stride]
    //     // // admin queue 队列深度 31
    //     // // aqa寄存器高16bit存储cq深度，低16bit存储sq深度

    //     // let bar = self.dev.bar;

    //     // let aqa_low_16 = 31 as u16;
    //     // let aqa_high_16 = 31 as u16;
    //     // let aqa = (aqa_high_16 as u32) << 16 | aqa_low_16 as u32;
    //     // let aqa_address = bar + NVME_REG_AQA;

    //     // // 将admin queue配置信息写入nvme设备寄存器AQA, admin_queue_attributes
    //     // unsafe{
    //     //     write_volatile(aqa_address as *mut u32, aqa);
    //     // }

    //     // info!("nvme aqa {:#x?} aqa_address {:#x?}", aqa, aqa_address);



        
    //     // 将admin queue的sq dma物理地址写入nvme设备上的寄存器ASQ
    //     let sq_dma_pa = nvme.sq_dma_pa as u32;
    //     let asq_address = bar + NVME_REG_ASQ ;
    //     info!("nvme asq_address {:#x?} sq_dma_pa {:#x?}", asq_address, sq_dma_pa);
    //     unsafe{
    //         write_volatile(asq_address as *mut u32, sq_dma_pa);
    //     }

    //     // 将admin queue的cq dma物理地址写入nvme设备上的寄存器ACQ
    //     let cq_dma_pa = nvme.cq_dma_pa as u32;
    //     let acq_address = bar + NVME_REG_ACQ;
    //     info!("nvme acq_address {:#x?} cq_dma_pa {:#x?}", acq_address, cq_dma_pa);
    //     unsafe{
    //         write_volatile(acq_address as *mut u32, cq_dma_pa);
    //     }

    //     let dev_dbs = bar + NVME_REG_DBS;

    //     let enable_ctrl = 0x460061;        
    //     unsafe{
    //         write_volatile((bar + NVME_REG_CC) as *mut u32, enable_ctrl)
    //     }
        
    //     let dev_status = unsafe {
    //         read_volatile((bar + NVME_REG_CSTS) as *mut u32)
    //     };

    //     info!("nvme dev_status {:#x?}", dev_status);

        
    //     // sq: &'static mut[Volatile<NvmeCommonCommand>],
    //     // ---------------------------------------------------------------------------------------------------
    //     //config admin queue






    //     let mut cmd = NvmeIdentify::nvme_init_non_mdts_limits();
    //     cmd.prp1 = nvme.data_dma_pa as u64;
    //     cmd.command_id = 0x1019;
    //     cmd.nsid = 1;
    //     cmd.cns = 0x6;
    //     let mut z = unsafe {
    //         core::mem::transmute(cmd)
    //     };

    //     info!("cmd :{:#x?}", z);
    //     nvme.sq[1].write(z);
    //     let admin_q_db = dev_dbs;
    //     unsafe{
    //         write_volatile(admin_q_db as *mut u32, 2)
    //     }
    //     loop {
    //         let status = nvme.cq[1].read();
    //         // let cq_phase = status1.status & 1;
    //         if status.status != 0 {
    //             info!("nvme cq :{:#x?}", status);
    //             unsafe{
    //                 write_volatile((admin_q_db + 0x4) as *mut u32, 2)
    //             }
    //             break;
    //         }
    //     }

    //     //nvme_set_queue_count
    //     let mut cmd = NvmeCommonCommand::new();
    //     cmd.opcode = 0x09;
    //     cmd.command_id = 0x101a;
    //     cmd.nsid = 1;
    //     cmd.cdw10 = 0x7;
    //     let mut z = unsafe {
    //         core::mem::transmute(cmd)
    //     };

    //     info!("cmd :{:#x?}", z);
    //     nvme.sq[2].write(z);

    //     unsafe{
    //         write_volatile(admin_q_db as *mut u32, 3)
    //     }

    //     loop {
    //         let status = nvme.cq[2].read();
    //         if status.status != 0 {
    //             info!("nvme cq :{:#x?}", status);
    //             unsafe{
    //                 write_volatile((admin_q_db + 0x4) as *mut u32, 3)
    //             }
    //             break;
    //         }
    //     }




    //     //nvme create cq
    //     let mut cmd = NvmeCommonCommand::new();
    //     cmd.opcode = 0x05;
    //     cmd.command_id = 0x101b;
    //     cmd.nsid = 1;
    //     cmd.prp1 = nvme.cq_dma_pa as u64;
    //     cmd.cdw10 = 0x3ff0001;
    //     cmd.cdw11 = 0x3;
    //     let mut z = unsafe {
    //         core::mem::transmute(cmd)
    //     };
    //     info!("cmd :{:#x?}", z);
    //     nvme.sq[3].write(z);
    //             unsafe{
    //         write_volatile(admin_q_db as *mut u32, 4)
    //     }
    //     loop {
    //         let status = nvme.cq[3].read();
    //         if status.status != 0 {
    //             info!("nvme cq :{:#x?}", status);
    //             unsafe{
    //                 write_volatile((admin_q_db + 0x4) as *mut u32, 4)
    //             }
    //             break;
    //         }
    //     }


    //     //nvme create sq
    //     let mut cmd = NvmeCommonCommand::new();
    //     cmd.opcode = 0x01;
    //     cmd.command_id = 0x2018;
    //     cmd.nsid = 1;
    //     cmd.prp1 = nvme.sq_dma_pa as u64;
    //     cmd.cdw10 = 0x3ff0001;
    //     cmd.cdw11 = 0x10001;
    //     let mut z = unsafe {
    //         core::mem::transmute(cmd)
    //     };
    //     info!("cmd :{:#x?}", z);
    //     nvme.sq[4].write(z);
    //             unsafe{
    //         write_volatile(admin_q_db as *mut u32, 5)
    //     }
    //     loop {
    //         let status = nvme.cq[4].read();
    //         if status.status != 0 {
    //             info!("nvme cq :{:#x?}", status);
    //             unsafe{
    //                 write_volatile((admin_q_db + 0x4) as *mut u32, 5)
    //             }
    //             break;
    //         }
    //     }
        

    // }











// register by real hardware
pci->map_irq

of_irq_parse_and_map_pci

irq_create_mapping
