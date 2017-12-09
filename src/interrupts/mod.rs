//! Services for handling system interrupts

mod gdt;

use spin::Once;
use x86_64::VirtualAddress;
use x86_64::instructions::segmentation::set_cs;
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::idt::{ExceptionStackFrame, Idt};
use x86_64::structures::tss::TaskStateSegment;

use mem::alloc_stack;

use self::gdt::Gdt;

const DOUBLE_FAULT_IST_INDEX: usize = 0;

static IDT: Once<Idt> = Once::new();

static GDT: Once<Gdt> = Once::new();
static TSS: Once<TaskStateSegment> = Once::new();

/// Initializes kernel's Interrupt Descriptor Table.
///
/// **Memory subsystem is required to be initialized.**
///
/// **This function should be called only once.**
///
/// [`init_ist`]: ./fn.init_ist.html
pub unsafe fn init() {
    let idt = IDT.call_once(create_idt);

    let double_fault_stack =
        alloc_stack(1).expect("could not allocate double fault interrupt handler stack");

    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = VirtualAddress(double_fault_stack.top);
        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);
    let gdt = GDT.call_once(|| {
        let mut gdt = Gdt::new();
        code_selector = gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        tss_selector = gdt.add_entry(gdt::Descriptor::tss_segment(&tss));
        gdt
    });

    gdt.load();

    set_cs(code_selector);
    load_tss(tss_selector);

    idt.load();
}

fn create_idt() -> Idt {
    let mut idt = Idt::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);

    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
    }

    idt
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame) {
    kprintln!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    kprintln!("\nEXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}
