//! IBM PC/AT 8259 PIC initialization code
//!
//! This module configures master and slave PICs to signal IRQs 0-15
//! as IRQs 32-48. It also provides primitives for handling these interrupts.

use kio::port::Port;
use kio::port::UnsafePort;

const MASTER_OFFSET: u8 = 0x20;
const SLAVE_OFFSET: u8 = MASTER_OFFSET + 8;

const MASTER_CMD: UnsafePort<u8> = unsafe { UnsafePort::new(0x20) };
const MASTER_DATA: UnsafePort<u8> = unsafe { UnsafePort::new(0x21) };
const SLAVE_CMD: UnsafePort<u8> = unsafe { UnsafePort::new(0xA0) };
const SLAVE_DATA: UnsafePort<u8> = unsafe { UnsafePort::new(0xA1) };

/// Scumbag port used to generate delays to wait for PICs to catch up with changes.
const WAIT_PORT: Port<u8> = unsafe { Port::new(0x80) };

/// End-of-interrupt command code.
const PIC_EOI: u8 = 0x20;

/// ICW4 (not) needed
const ICW1_ICW4: u8 = 0x01;

#[allow(dead_code)]
/// Single (cascade) mode
const ICW1_SINGLE: u8 = 0x02;

#[allow(dead_code)]
/// Call address interval 4 (8)
const ICW1_INTERVAL4: u8 = 0x04;

#[allow(dead_code)]
/// Level triggered (edge) mode
const ICW1_LEVEL: u8 = 0x08;

/// Initialization - required!
const ICW1_INIT: u8 = 0x10;

/// 8086/88 (MCS-80/85) mode
const ICW4_8086: u8 = 0x01;

#[allow(dead_code)]
/// Auto (normal) EOI
const ICW4_AUTO: u8 = 0x02;

#[allow(dead_code)]
/// Buffered mode/slave
const ICW4_BUF_SLAVE: u8 = 0x08;

#[allow(dead_code)]
/// Buffered mode/master
const ICW4_BUF_MASTER: u8 = 0x0C;

#[allow(dead_code)]
/// Special fully nested (not)
const ICW4_SFNM: u8 = 0x10;

/// Initializes PIC
///
/// **This function should be only called once.**
///
/// **IDT is required to be initialized.**
pub unsafe fn init() {
    // Tell each PIC that we're going to send it a three-byte
    // initialization sequence on its data port.
    MASTER_CMD.write(ICW1_INIT + ICW1_ICW4);
    io_wait();
    SLAVE_CMD.write(ICW1_INIT + ICW1_ICW4);
    io_wait();

    // Byte 1: set vector offsets
    MASTER_DATA.write(MASTER_OFFSET);
    io_wait();
    SLAVE_DATA.write(SLAVE_OFFSET);
    io_wait();

    // Byte 2: configure chaining between PIC1 and PIC2
    MASTER_DATA.write(4);
    io_wait();
    SLAVE_DATA.write(2);
    io_wait();

    // Byte 3: set PIC mode
    MASTER_DATA.write(ICW4_8086);
    io_wait();
    SLAVE_DATA.write(ICW4_8086);
    io_wait();

    // Clear IMRs
    MASTER_DATA.write(0);
    io_wait();
    SLAVE_DATA.write(0);
    io_wait();
}

/// Notifies end of interrupt
pub unsafe fn eoi(irq: u8) {
    assert!(valid_irq(irq));

    if irq >= SLAVE_OFFSET {
        SLAVE_CMD.write(PIC_EOI);
    }

    MASTER_CMD.write(PIC_EOI);
}

/// Sets mask of IRQ in IMR
pub unsafe fn mask(irq: u8) {
    assert!(valid_irq(irq));
    let (port, irqline) = get_data_port_and_irqline(irq);
    let val = port.read() | (1 << irqline);
    port.write(val);
}

/// Clears mask of IRQ in IMR
pub unsafe fn unmask(irq: u8) {
    assert!(valid_irq(irq));
    let (port, irqline) = get_data_port_and_irqline(irq);
    let val = port.read() & !(1 << irqline);
    port.write(val);
}

#[inline]
const fn valid_irq(irq: u8) -> bool {
    MASTER_OFFSET <= irq && irq < SLAVE_OFFSET + 8
}

fn get_data_port_and_irqline(irq: u8) -> (UnsafePort<u8>, u8) {
    if irq >= SLAVE_OFFSET {
        (SLAVE_DATA, irq - SLAVE_OFFSET)
    } else {
        (MASTER_DATA, irq - MASTER_OFFSET)
    }
}

#[inline]
fn io_wait() {
    WAIT_PORT.write(0);
}
