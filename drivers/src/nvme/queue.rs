use alloc::collections::{VecDeque};




use super::NvmeCommand;

pub struct NvmeQueue{
    pub sq: VecDeque<NvmeCommand>,
    pub cq: VecDeque<NvmeCommand>,
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

    pub fn sq_push(&mut self, cmd: NvmeCommand){
        self.sq.push_front(cmd);
    }
}

