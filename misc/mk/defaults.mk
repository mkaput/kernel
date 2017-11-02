############################################################
## Build default configuration
############################################################

## Some of variables are dependent on compilation architecture, and so
## they are in some level coupled. All of these should relate to one
## single arch. Otherwise strange things may happen.

############################################################
## Target configuration

# Target triple for which the kernel will be compiled.
# Possible values:
# * x86_64-unknown-kernel - AMD64 / Intel EM64T
export TARGET ?= x86_64-unknown-kernel

# Cargo build mode for Rust code
# Possible values:
# * debug
# * release
export CARGO_MODE ?= debug

############################################################
## Build tools

# Cargo executable name
# We use xargo for cross-compiling core library.
export CARGO ?= xargo

# NASM executable name
export NASM ?= nasm

# GRUB mkrescue
export GRUB_MKRESCUE ?= grub2-mkrescue

# QEMU used for `run` target
export QEMU_RUN ?= qemu-system-x86_64

# Gitbook used for building docs
export GITBOOK ?= gitbook
