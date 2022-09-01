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

