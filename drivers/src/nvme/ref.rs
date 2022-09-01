


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










// Host如果想往SSD上写入用户数据，需要告诉SSD写入什么数据，
// 写入多少数据，以及数据源在内存中的什么位置，这些信息包含在Host向SSD发送的Write命令中。
// 每笔用户数据对应着一个叫做LBA（Logical Block Address）的东西，Write命令通过指定LBA来告诉SSD写入的是什么数据。
// 对NVMe/PCIe来说，SSD收到Write命令后，通过PCIe去Host的内存数据所在位置读取数据，然后把这些数据写入到闪存中，同时得到LBA与闪存位置的映射关系。