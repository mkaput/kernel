//! Services for handling system interrupts

use core::mem;

use spin::{Mutex, Once};
use x86_64;
use x86_64::VirtualAddress;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::idt::{ExceptionStackFrame, HandlerFunc, Idt, PageFaultErrorCode};
use x86_64::structures::tss::TaskStateSegment;

use dev::text_video::{TextColor, TextStyle};
use kio;
use mem::alloc_stack;
use mem::gdt::{self, Gdt};

const DOUBLE_FAULT_IST_INDEX: usize = 0;
const MACHINE_CHECK_IST_INDEX: usize = 1;

static IDT: Mutex<Option<Idt>> = Mutex::new(None);

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
    *(IDT.lock()) = Some(create_idt());

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

    x86_64::instructions::segmentation::set_cs(code_selector);
    x86_64::instructions::tables::load_tss(tss_selector);

    load_idt();
}

/// Registers handler function for custom interrupts (INTn >= 32)
///
/// **IDT has to be initialized before calling this function.**
pub unsafe fn register_interrupt(int: u8, handler: HandlerFunc) {
    assert!(32 <= int);

    let mut idt_lock = IDT.lock();
    let x = &mut *idt_lock;
    if let &mut Some(ref mut idt) = x {
        idt[int as usize].set_handler_fn(handler);
    } else {
        panic!("IDT have not been initialized yet");
    }
}

pub unsafe fn enable() {
    x86_64::instructions::interrupts::enable();
}

pub unsafe fn disable() {
    x86_64::instructions::interrupts::disable();
}

fn load_idt() {
    // WTF
    let mut idt_lock = IDT.lock();
    let x = &mut *idt_lock;
    let y: &'static mut Option<Idt> = unsafe { mem::transmute(x) };
    if let &mut Some(ref mut idt) = y {
        idt.load();
    } else {
        unreachable!();
    }
}

fn create_idt() -> Idt {
    let mut idt = Idt::new();

    idt.divide_by_zero.set_handler_fn(divide_by_zero_handler);
    // TODO: Debug
    // TODO: Non-maskable Interrupt
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.overflow.set_handler_fn(overflow_handler);
    // TODO: Bound range exceeded
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    // TODO: Device not available

    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
    }

    // TODO: Invalid TSS
    // TODO: Segment Not Present
    // TODO: Stack Segment Fault
    // TODO: General Protection Fault

    unsafe {
        idt.page_fault
            .set_handler_fn(page_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
    }

    // TODO: x87 Floating-Point Exception
    // TODO: Alignment Check Exception

    unsafe {
        idt.machine_check
            .set_handler_fn(machine_check_handler)
            .set_stack_index(MACHINE_CHECK_IST_INDEX as u16);
    }

    // TODO: SIMD Floating-Point
    // TODO: Virtualization
    // TODO: Security Exception

    for irq in idt.interrupts.iter_mut() {
        irq.set_handler_fn(default_handler);
    }

    idt
}

extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: &mut ExceptionStackFrame) {
    print_exception("DIVIDE BY ZERO", stack_frame);
    loop {}
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame) {
    print_exception("BREAKPOINT", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: &mut ExceptionStackFrame) {
    print_exception("OVERFLOW", stack_frame);
    loop {}
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: &mut ExceptionStackFrame) {
    print_exception("INVALID OPCODE", stack_frame);
    loop {}
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    print_exception("DOUBLE FAULT", stack_frame);
    loop {}
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut ExceptionStackFrame,
    error_code: PageFaultErrorCode,
) {
    print_exception_ex("PAGE FAULT", || {
        println!("Error code: {:#?}", error_code);
        println!("{:#?}", stack_frame);
    });
    loop {}
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: &mut ExceptionStackFrame) {
    print_exception("OOPS MACHINE CHECK", stack_frame);
    loop {}
}

extern "x86-interrupt" fn default_handler(_stack_frame: &mut ExceptionStackFrame) {}

fn print_exception(name: &str, stack_frame: &ExceptionStackFrame) {
    print_exception_ex(name, || {
        println!("{:#?}", stack_frame);
    });
}

fn print_exception_ex(name: &str, info_provider: impl FnOnce()) {
    let header = TextStyle {
        foreground: TextColor::White,
        background: TextColor::Red,
    };

    let info = TextStyle {
        foreground: TextColor::LightRed,
        background: TextColor::Black,
    };

    println!();

    kio::with_output_style(header, || {
        println!("=== EXCEPTION: {} ===", name);
    });
    kio::with_output_style(info, info_provider);

    println!();
}
