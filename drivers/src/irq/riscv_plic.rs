use core::arch::asm;
use core::ops::Range;

use lock::Mutex;

use crate::io::{Io, Mmio};
use crate::prelude::IrqHandler;
use crate::scheme::{IrqScheme, Scheme};
use crate::{utils::IrqManager, DeviceError, DeviceResult};

const IRQ_RANGE: Range<usize> = 1..1024;

const PLIC_PRIORITY_BASE: usize = 0x0;
const PLIC_ENABLE_BASE: usize = 0x2080;

const PLIC_CONTEXT_BASE: usize = 0x20_1000;
const PLIC_CONTEXT_THRESHOLD: usize = 0x0;
const PLIC_CONTEXT_CLAIM: usize = 0x4 / core::mem::size_of::<u32>();

const PLIC_ENABLE_HART_OFFSET: usize = 0x100 / core::mem::size_of::<u32>();
const PLIC_PRIORITY_HART_OFFSET: usize = 0x2000 / core::mem::size_of::<u32>();
const PLIC_CONTEXT_CLAIM_HART_OFFSET: usize = 0x2000 / core::mem::size_of::<u32>();

struct PlicUnlocked {
    priority_base: &'static mut Mmio<u32>,
    enable_base: &'static mut Mmio<u32>,
    context_base: &'static mut Mmio<u32>,
    manager: IrqManager<1024>,
}



use core::ptr;
// qemu puts platform-level interrupt controller (PLIC) here.
pub const PLIC_BASE: usize = 0x0c000000;

/// qemu puts UART registers here in physical memory.
pub const UART0:usize = 0x10000000;
pub const UART0_IRQ: u32 = 10;

/// virtio mmio interface
pub const VIRTIO0:usize = 0x10001000;
pub const VIRTIO0_IRQ: u32 = 1;

fn plic_senable(hart_id: usize) -> usize {
    PLIC_BASE + 0x2080 + hart_id * 0x100
}

fn plic_spriority(hart_id: usize) -> usize {
    PLIC_BASE + 0x201000 + hart_id * 0x2000
}

fn plic_sclaim(hart_id: usize) -> usize {
    PLIC_BASE + 0x201004 + hart_id * 0x2000
}


pub struct Plic {
    inner: Mutex<PlicUnlocked>,
}

impl PlicUnlocked {
    /// Toggle irq enable on the current hart.
    fn toggle(&mut self, irq_num: usize, enable: bool) {
        debug_assert!(IRQ_RANGE.contains(&irq_num));
        let hart_id = cpu_id() as usize;
        let size = core::mem::size_of::<u32>();
        let mmio = self
            .enable_base
            .add(irq_num / 32 * (size));

        // 1 << (hwirq % 32)    
        let mask = 1 << (irq_num % 32);

        let enable_write = mmio.read() | mask;
        let disable_write = mmio.read() & !mask;
        info!("enable_write: {:#x}, disable_write: {:#x}", enable_write, disable_write);
        if enable {
            mmio.write(enable_write);
        } else {
            mmio.write(disable_write);
        }
    }

    /// Ask the PLIC what type of interrupt is occurred on the current hart.
    fn pending_irq(&mut self) -> Option<usize> {
        let hart_id = cpu_id() as usize;
        let irq_num = self
            .context_base
            .add(PLIC_CONTEXT_CLAIM_HART_OFFSET * hart_id + PLIC_CONTEXT_CLAIM)
            .read() as usize;
        if irq_num == 0 {
            None
        } else {
            Some(irq_num)
        }
    }

    /// Tell the PLIC we've served this IRQ.
    fn eoi(&mut self, irq_num: usize) {
        debug_assert!(IRQ_RANGE.contains(&irq_num));
        let hart_id = cpu_id() as usize;
        self.context_base
            .add(PLIC_CONTEXT_CLAIM + PLIC_CONTEXT_CLAIM_HART_OFFSET * hart_id)
            .write(irq_num as _);
    }

    /// Set the priority for the irq_num.
    fn set_priority(&mut self, irq_num: usize, priority: u8) {
        debug_assert!(IRQ_RANGE.contains(&irq_num));
        self.priority_base.add(irq_num).write(priority as _);
    }

    /// Set current hart's priority threshold to 0.
    fn set_threshold(&mut self, threshold: u8) {
        let hart_id = cpu_id() as usize;
        self.context_base
            .add(PLIC_PRIORITY_HART_OFFSET * hart_id + PLIC_CONTEXT_THRESHOLD)
            .write(threshold as _);
    }

    fn init_hart(&mut self) {
        self.set_threshold(0);
    }
}

impl Plic {
    pub fn new(base: usize) -> Self {
        let mut inner = PlicUnlocked {
            priority_base: unsafe { Mmio::<u32>::from_base(base + PLIC_PRIORITY_BASE) },
            enable_base: unsafe { Mmio::<u32>::from_base(base + PLIC_ENABLE_BASE) },
            context_base: unsafe { Mmio::<u32>::from_base(base + PLIC_CONTEXT_BASE) },
            manager: IrqManager::new(IRQ_RANGE),
        };
        inner.init_hart();
        Self {
            inner: Mutex::new(inner),
        }
    }
}

impl Scheme for Plic {
    fn name(&self) -> &str {
        "riscv-plic"
    }

    fn handle_irq(&self, _unused: usize) {
        let mut inner = self.inner.lock();
        while let Some(irq_num) = inner.pending_irq() {
            if inner.manager.handle(irq_num).is_err() {
                warn!("no registered handler for IRQ {}!", irq_num);
            }
            trace!("riscv plic handle irq: {}", irq_num);
            inner.eoi(irq_num);
        }
    }
}

impl IrqScheme for Plic {
    fn is_valid_irq(&self, irq_num: usize) -> bool {
        IRQ_RANGE.contains(&irq_num)
    }

    fn mask(&self, irq_num: usize) -> DeviceResult {
        if self.is_valid_irq(irq_num) {
            self.inner.lock().toggle(irq_num, false);
            Ok(())
        } else {
            Err(DeviceError::InvalidParam)
        }
    }

    fn unmask(&self, irq_num: usize) -> DeviceResult {
        if self.is_valid_irq(irq_num) {
            self.inner.lock().toggle(irq_num, true);
            Ok(())
        } else {
            Err(DeviceError::InvalidParam)
        }
    }

    fn register_handler(&self, irq_num: usize, handler: IrqHandler) -> DeviceResult {
        let mut inner = self.inner.lock();
        inner.manager.register_handler(irq_num, handler).map(|_| {
            inner.set_priority(irq_num, 7);
        })
    }

    fn unregister(&self, irq_num: usize) -> DeviceResult {
        self.inner.lock().manager.unregister_handler(irq_num)
    }

    fn init_hart(&self) {
        self.inner.lock().init_hart();
    }
}

fn cpu_id() -> u8 {
    let mut cpu_id;
    unsafe {
        asm!("mv {0}, tp", out(reg) cpu_id);
    }
    cpu_id
}
