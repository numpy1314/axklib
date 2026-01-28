#![allow(improper_ctypes_definitions)]

use axklib::{AxResult, IrqHandler, Klib, PhysAddr, VirtAddr};
use core::time::Duration;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use trait_ffi::impl_extern_trait;

static MEM_IOMAP_CALLED: AtomicBool = AtomicBool::new(false);
static TIME_WAIT_CALLED: AtomicBool = AtomicBool::new(false);
static IRQ_REG_CALLED: AtomicBool = AtomicBool::new(false);
static LAST_IRQ_NUM: AtomicUsize = AtomicUsize::new(0);

struct MockPlatform;

#[impl_extern_trait(name = "axklib_0_1", abi = "rust")]
impl Klib for MockPlatform {
    fn mem_iomap(addr: PhysAddr, size: usize) -> AxResult<VirtAddr> {
        MEM_IOMAP_CALLED.store(true, Ordering::SeqCst);
        assert_eq!(usize::from(addr), 0x1000_0000, "Mock: Unexpected PhysAddr");
        assert_eq!(size, 0x1000, "Mock: Unexpected size");
        Ok(VirtAddr::from(usize::from(addr) + 0x1000))
    }

    fn time_busy_wait(dur: Duration) {
        TIME_WAIT_CALLED.store(true, Ordering::SeqCst);
        assert_eq!(dur.as_micros(), 100, "Mock: Unexpected duration");
    }

    fn irq_set_enable(_irq: usize, _enabled: bool) {}

    fn irq_register(irq: usize, _handler: IrqHandler) -> bool {
        IRQ_REG_CALLED.store(true, Ordering::SeqCst);
        LAST_IRQ_NUM.store(irq, Ordering::SeqCst);
        true
    }
}

#[test]
fn test_mem_module_reexport() {
    MEM_IOMAP_CALLED.store(false, Ordering::SeqCst);
    let paddr = PhysAddr::from(0x1000_0000);
    let size = 0x1000;

    let result = axklib::mem::iomap(paddr, size);

    assert!(result.is_ok());
    assert!(
        MEM_IOMAP_CALLED.load(Ordering::SeqCst),
        "Mock method was not called!"
    );
    let vaddr = result.unwrap();
    assert_eq!(usize::from(vaddr), 0x1000_0000 + 0x1000);
}

#[test]
fn test_time_module_reexport() {
    TIME_WAIT_CALLED.store(false, Ordering::SeqCst);
    axklib::time::busy_wait(Duration::from_micros(100));
    assert!(TIME_WAIT_CALLED.load(Ordering::SeqCst));
}

fn dummy_handler() {}

#[test]
fn test_irq_module_reexport() {
    IRQ_REG_CALLED.store(false, Ordering::SeqCst);
    LAST_IRQ_NUM.store(0, Ordering::SeqCst);

    let success = axklib::irq::register(32, dummy_handler);

    assert!(success);
    assert!(IRQ_REG_CALLED.load(Ordering::SeqCst));
    assert_eq!(LAST_IRQ_NUM.load(Ordering::SeqCst), 32);
}
