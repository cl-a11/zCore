use core::marker::PhantomData;
use volatile::Volatile;
use alloc::slice;





use super::NvmeCreateSq;
use super::NvmeCreateCq;


// 0x40
pub const NVME_COMMAND_SIZE: usize = 64;


pub struct Nvme<P: Provider> {
    header: usize,
    size: usize,
    provider: PhantomData<P>,
    sq: &'static mut[Volatile<NvmeCreateSq>],
    cq: &'static mut[Volatile<NvmeCreateCq>],

    // registers: &'static mut [Volatile<u32>],
}

impl<P: Provider> Nvme<P> {

    pub fn new(header:usize, size:usize) -> Self{

        let (sq_va, sq_pa) = P::alloc_dma(P::PAGE_SIZE);
        let (cq_va, cq_pa) = P::alloc_dma(P::PAGE_SIZE);

        let submit_queue = unsafe{
            slice::from_raw_parts_mut(
                sq_va as *mut Volatile<NvmeCreateSq>, 
                PAGE_SIZE/ NVME_COMMAND_SIZE
            )
        };

        let complete_queue = unsafe{
            slice::from_raw_parts_mut(
                cq_va as *mut Volatile<NvmeCreateCq>, 
                PAGE_SIZE/ NVME_COMMAND_SIZE
            )
        };

        let sq_cmd = NvmeCreateSq::new_create_sq_command();
        submit_queue[0].write(sq_cmd);

        Nvme{
            header,
            size,
            provider: PhantomData,
            sq: submit_queue,
            cq: complete_queue,
        }
    
    }

    pub fn handle_interrupt(&mut self) -> bool {
        true
    }
}


impl<P: Provider> Drop for Nvme<P> {

    fn drop(&mut self) {

    }
}


pub struct NvmeDriver {

}

impl NvmeDriver{
    pub fn new() -> Self{
        NvmeDriver{
        }
    }

    pub fn handle_interrupt(&self) -> usize{
        0
    }
}




/// External functions that drivers must use
pub trait Provider {
    /// Page size (usually 4K)
    const PAGE_SIZE: usize;

    /// Allocate consequent physical memory for DMA.
    /// Return (`virtual address`, `physical address`).
    /// The address is page aligned.
    fn alloc_dma(size: usize) -> (usize, usize);

    /// Deallocate DMA
    fn dealloc_dma(vaddr: usize, size: usize);
}



pub struct ProviderImpl;

impl Provider for ProviderImpl {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_dma(size: usize) -> (usize, usize) {
        let paddr = unsafe { drivers_dma_alloc(size / PAGE_SIZE) };
        let vaddr = phys_to_virt(paddr);
        (vaddr, paddr)
    }

    fn dealloc_dma(vaddr: usize, size: usize) {
        let paddr = virt_to_phys(vaddr);
        unsafe { drivers_dma_dealloc(paddr, size / PAGE_SIZE) };
    }
}

pub fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    unsafe { drivers_phys_to_virt(paddr) }
}

pub fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    unsafe { drivers_virt_to_phys(vaddr) }
}

pub fn timer_now_as_micros() -> u64 {
    unsafe { drivers_timer_now_as_micros() }
}

extern "C" {
    fn drivers_dma_alloc(pages: usize) -> PhysAddr;
    fn drivers_dma_dealloc(paddr: PhysAddr, pages: usize) -> i32;
    fn drivers_phys_to_virt(paddr: PhysAddr) -> VirtAddr;
    fn drivers_virt_to_phys(vaddr: VirtAddr) -> PhysAddr;
    fn drivers_timer_now_as_micros() -> u64;
}

pub const PAGE_SIZE: usize = 4096;

type VirtAddr = usize;
type PhysAddr = usize;