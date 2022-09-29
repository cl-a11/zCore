#![cfg_attr(not(feature = "libos"), no_std)]
// #![deny(warnings)]
#![no_main]
#![feature(naked_functions, asm_sym, asm_const)]
#![feature(default_alloc_error_handler)]

use core::sync::atomic::{AtomicBool, Ordering};

extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate cfg_if;

#[macro_use]
mod logging;

#[cfg(not(feature = "libos"))]
mod lang;

mod fs;
mod handler;
mod memory;
mod platform;
mod utils;

static STARTED: AtomicBool = AtomicBool::new(false);

fn primary_main(config: kernel_hal::KernelConfig) {
    logging::init();
    memory::init_heap();
    kernel_hal::primary_init_early(config, &handler::ZcoreKernelHandler);
    let options = utils::boot_options();
    logging::set_max_level(&options.log_level);
    info!("Boot options: {:#?}", options);
    memory::init_frame_allocator(&kernel_hal::mem::free_pmem_regions());
    kernel_hal::primary_init();
    STARTED.store(true, Ordering::SeqCst);




    //register irq for nvme
    use alloc::boxed::Box;
    let irq = kernel_hal::drivers::all_irq().find("riscv-plic").unwrap();

    let nvme1 = kernel_hal::drivers::all_block()
    .find("real_nvme")
    .unwrap();
    let irq_num = 0x21;
    irq.register_handler(irq_num, Box::new(move || nvme1.handle_irq(irq_num)));
    irq.unmask(irq_num);
    

    warn!("test nvme rw");
    let nvme2 = kernel_hal::drivers::all_block()
    .find("real_nvme")
    .unwrap();
    
    let write_buf:&[u8] = &[1u8;512];

    nvme2.write_block(1, &write_buf);


    info!("plic handle irq");


    // unsafe{
    //     use core::arch::asm;

    //     let ext = 1<<9;
    //     let timer = 1<<5;
    //     asm!("csrw sie, {}", in(reg) ext);

    //     asm!("csrw sie, {}", in(reg) timer);
    // }

    loop {
        irq.handle_irq(irq_num);
    }

    let mut read_buf = [0u8; 512];

    warn!("before read_buf: {:?}", read_buf);

    nvme2.read_block(1, &mut read_buf);

    warn!("after read_buf: {:?}", read_buf);
    
    warn!("Kernel loop!");

    cfg_if! {
        if #[cfg(all(feature = "linux", feature = "zircon"))] {
            panic!("Feature `linux` and `zircon` cannot be enabled at the same time!");
        } else if #[cfg(feature = "linux")] {
            let args = options.root_proc.split('?').map(Into::into).collect(); // parse "arg0?arg1?arg2"
            let envs = alloc::vec!["PATH=/usr/sbin:/usr/bin:/sbin:/bin".into()];
            let rootfs = fs::rootfs();
            let proc = zcore_loader::linux::run(args, envs, rootfs);
            utils::wait_for_exit(Some(proc))
        } else if #[cfg(feature = "zircon")] {
            let zbi = fs::zbi();
            let proc = zcore_loader::zircon::run_userboot(zbi, &options.cmdline);
            utils::wait_for_exit(Some(proc))
        } else {
            panic!("One of the features `linux` or `zircon` must be specified!");
        }
    }
}

#[cfg(not(any(feature = "libos", target_arch = "aarch64")))]
fn secondary_main() -> ! {
    while !STARTED.load(Ordering::SeqCst) {
        core::hint::spin_loop();
    }
    // Don't print anything between previous line and next line.
    // Boot hart has initialized the UART chip, so we will use
    // UART for output instead of SBI, but the current HART is
    // not mapped to UART MMIO, which means we can't output
    // until secondary_init is complete.
    kernel_hal::secondary_init();
    log::info!("hart{} inited", kernel_hal::cpu::cpu_id());
    utils::wait_for_exit(None)
}
