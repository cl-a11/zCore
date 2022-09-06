use alloc::slice;
use alloc::collections::VecDeque;
use core::marker::PhantomData;


use volatile::Volatile;

use super::NvmeCommand;

pub struct NvmeQueue{
    pub sq: VecDeque<NvmeCommand>,
    pub cq: VecDeque<NvmeCommand>,

    // queue doorbell register
    pub q_db: usize,

    pub q_depth: usize,

    pub cq_vector: usize,

    pub sq_head: usize,

    pub sq_tail: usize,

    pub qid: usize,



}






impl NvmeQueue{

    /* 分配一个nvme queue，包括其需要的CQ和SQ空间和DMA地址 */
    /* 通过admin queue告知nvme设备创建cq和sq，并且分配对应的irq */
    pub fn new() -> Self{
        NvmeQueue{
            sq: VecDeque::new(),
            cq: VecDeque::new(),
            q_db: 0,
            q_depth: 0,
            cq_vector: 0,
            sq_head: 0,
            sq_tail: 0,
            qid: 0,
        }
    }

    pub fn sq_push(&mut self, cmd: NvmeCommand){
        self.sq.push_front(cmd);
    }
}

