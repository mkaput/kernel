// MIT License
//
// Copyright (c) [year] [fullname]
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! High level Rust wrapper for CPU I/O instructions, code forked from the
//! [cpuio] crate.
//!
//! [cpuio]: https://crates.io/crates/cpuio

use core::marker::PhantomData;

use x86_64::instructions::port::{inb, inl, inw, outb, outl, outw};

/// This trait is defined for any type which can be read or written over a
/// port.  The processor supports I/O with `u8`, `u16` and `u32`.  The
/// functions in this trait are all unsafe because they can write to
/// arbitrary ports.
pub trait InOut {
    /// Read a value from the specified port.
    unsafe fn port_in(port: u16) -> Self;

    /// Write a value to the specified port.
    unsafe fn port_out(port: u16, value: Self);
}

impl InOut for u8 {
    unsafe fn port_in(port: u16) -> u8 {
        inb(port)
    }
    unsafe fn port_out(port: u16, value: u8) {
        outb(port, value);
    }
}

impl InOut for u16 {
    unsafe fn port_in(port: u16) -> u16 {
        inw(port)
    }
    unsafe fn port_out(port: u16, value: u16) {
        outw(port, value);
    }
}

impl InOut for u32 {
    unsafe fn port_in(port: u16) -> u32 {
        inl(port)
    }
    unsafe fn port_out(port: u16, value: u32) {
        outl(port, value);
    }
}

/// An I/O port over an arbitrary type supporting the `InOut` interface.
///
/// This version of `Port` has safe `read` and `write` functions, and it's
/// appropriate for communicating with hardware that can't violate Rust's
/// safety guarantees.
#[derive(Debug)]
pub struct Port<T: InOut> {
    // Port address.
    port: u16,

    // Zero-byte placeholder.  This is only here so that we can have a
    // type parameter `T` without a compiler error.
    ty: PhantomData<T>,
}

impl<T: InOut> Port<T> {
    /// Create a new I/O port.
    pub const unsafe fn new(port: u16) -> Port<T> {
        Port {
            port,
            ty: PhantomData,
        }
    }

    /// Read data from the port.  This is nominally safe, because you
    /// shouldn't be able to get hold of a port object unless somebody
    /// thinks it's safe to give you one.
    pub fn read(&self) -> T {
        unsafe { T::port_in(self.port) }
    }

    /// Write data to the port.
    pub fn write(&self, value: T) {
        unsafe {
            T::port_out(self.port, value);
        }
    }
}

/// An unsafe I/O port over an arbitrary type supporting the `InOut`
/// interface.
///
/// This version of `Port` has unsafe `read` and `write` functions, and
/// it's appropriate for speaking to hardware that can potentially corrupt
/// memory or cause undefined behavior.
#[derive(Debug)]
pub struct UnsafePort<T: InOut> {
    port: u16,
    ty: PhantomData<T>,
}

impl<T: InOut> UnsafePort<T> {
    /// Create a new I/O port.
    pub const unsafe fn new(port: u16) -> UnsafePort<T> {
        UnsafePort {
            port,
            ty: PhantomData,
        }
    }

    /// Read data from the port.
    pub unsafe fn read(&self) -> T {
        T::port_in(self.port)
    }

    /// Write data to the port.
    pub unsafe fn write(&self, value: T) {
        T::port_out(self.port, value);
    }
}
