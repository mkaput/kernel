# Inside main

```rust
#[no_mangle]
pub extern "C" fn krnl_main(mb_info_addr: usize) {
```

This is quick walkthrough of internals of `krnl_main` function.

---

```rust
    // ATTENTION: we have small stack, no guard page and no interrupts configured
```
The `krnl_main` starts in quite harsh environment, kernel stack is partially available (first 16KiB), and there is no guard page so kernel stack overflows are dangerous.

## Early console

```rust
    // Set up early console ASAP, so we will be able to use `println!`
    unsafe {
        kio::early_init();
    }
```

The first thing kernel does, is configuring VGA text buffer (`0xb8000`), so it can communicate any errors to user. Text buffer is managed by `drv::gfx::vga::text_buffer` driver, which implements `dev::output_serial::OutputSerial` and `dev::text_video::TextVideo` device abstractions.

The initialization is done inside [KIO] subsystem code. Kernel enables hardware text cursor, and clears the screen, it talks directly with driver static code, as [Device Manager] is not implemented yet.

```rust
    // in kio::early_init()
    println!("early console works");
```

And so this is the very beginning of kernel-user interaction!

## Reading Multiboot2 information table

```rust
    let boot_info = unsafe { multiboot2::load(mb_info_addr) };

    let bootloader_name = boot_info.boot_loader_name_tag().map(|t| t.name());
    println!("bootloader: {}", bootloader_name.unwrap_or("unknown"));

    let cmdline = boot_info.command_line_tag().map(|t| t.command_line());
    println!("cmdline: {}", cmdline.unwrap_or("none"));
```

Next, kernel reads Multiboot2 information table, using [multiboot2] crate. Apart more useful information, it also contains things like bootloader name and version, and kernel boot command line. We print them, cause why not :-)

## Initializing memory

```rust
    unsafe {
        mem::init(boot_info);
```

Kernel now initializes [memory manager] subsystem. It does lots of things, including setting up:

- Frame allocator
- Paging
- Kernel heap
- Kernel stack
- Stack allocator, used by interrupts

This procedure also remaps the kernel, i.e. it sets virtual addressing of kernel code & data. 

At this moment, we can allocate arbitrary memory, and we are protected from kernel stack overflows, though we cannot in anyway react to such errors, as interrupts are not configured yet. 

## Setting up interrupts

```rust
        kio::idt::init();
        kio::pic::init();
        kio::idt::enable();
    }
```

Having proper memory management, kernel sets up [Interrupt Descriptor Table] and configures [Programmable Interrupt Controller], in this case [Intel 8259], which is the most basic solution.  

---

```rust
    // ATTENTION: now everything is fine
```

## Loading drivers

```rust
    dev::mgr::init();

    drv::hid::atkbd::init();
```

Now kernel initializes [Device Manager] which is responsible for managing lifetime od devices and drivers.

One of drivers making use if it is `drv::hid::atkbd`, which handles PC/AT keyboards, communicating via IRQ1 and `0x60` and `0x64` ports. This device is available as *kbd0* in device manager.

## Starting the shell

```rust
    shell::start();
```

We are nearing the end of main function. The last step is to start the [Kernel Shell], which takes control of kernel and user interaction.

---

```rust
    unreachable!();
}
```

Although shell should never end, in case of programmer's bug there is hanging `unreachable!()` macro invocation, which simply panics with reasonable message.

[multiboot2]: https://crates.io/crates/multiboot2
[Interrupt Descriptor Table]: https://en.wikipedia.org/wiki/Interrupt_descriptor_table
[Intel 8259]: https://en.wikipedia.org/wiki/Intel_8259
[Programmable Interrupt Controller]: https://en.wikipedia.org/wiki/Programmable_interrupt_controller

[Memory manager]: ../sys/mem.md
[KIO]: ../sys/kio.md
[Device manager]: ../sys/devmgr.md
[Kernel Shell]: ../sys/shell.md
