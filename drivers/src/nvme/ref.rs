


// bar
// **http://www.ssdfans.com/?p=8210

// http://www.ssdfans.com/?p=8171
// http://www.ssdfans.com/?p=8171
// http://www.ssdfans.com/?p=8210


// 一个PCIe设备，可能有若干个内部空间（属性可能不一样，比如有些可预读，有些不可预读）需要映射到内存空间，设备出厂时，这些空间的大小和属性都写在Configuration BAR寄存器里面，然后上电后，
// 系统软件读取这些BAR，分别为其分配对应的系统内存空间，并把相应的内存基地址写回到BAR。（BAR的地址其实是PCI总线域的地址，CPU访问的是存储器域的地址，CPU访问PCIe设备时，需要把总线域地址转换成存储器域的地址。）





// 设备内存用page划分
// Physical Region Page

// prp
// http://www.ssdfans.com/?p=8173
// http://www.ssdfans.com/?p=8141




//linux 块设备驱动
// https://www.bilibili.com/read/cv17063262



// Device-to-device memory-transfer offload with P2PDMA
// https://lwn.net/Articles/767281/
// PCI devices expose memory to the host system in form of memory regions defined by base address registers (BARs). 
// Those are regions mapped into the host's physical memory space. 
// All regions are mapped into the same address space, and PCI DMA operations can use those addresses directly.
// It is thus possible for a driver to configure a PCI DMA operation to perform transfers between the memory zones of two devices while bypassing system memory completely. 

// linux地址空间  pcie dma
// https://www.oreilly.com/library/view/linux-device-drivers/0596005903/ch15.html



// Host如果想往SSD上写入用户数据，需要告诉SSD写入什么数据，
// 写入多少数据，以及数据源在内存中的什么位置，这些信息包含在Host向SSD发送的Write命令中。
// 每笔用户数据对应着一个叫做LBA（Logical Block Address）的东西，Write命令通过指定LBA来告诉SSD写入的是什么数据。
// 对NVMe/PCIe来说，SSD收到Write命令后，通过PCIe去Host的内存数据所在位置读取数据，然后把这些数据写入到闪存中，同时得到LBA与闪存位置的映射关系。




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