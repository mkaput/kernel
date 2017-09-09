############################################################
## Build default configuration
############################################################

## Some of variables are dependent on compilation architecture, and so
## they are in some level coupled. All of these should relate to one
## single arch. Otherwise strange things may happen.

############################################################
## Architecture configuration

# CPU architecture for which the kernel will be compiled.
# Possible values:
# * x86_64 - AMD64 / Intel EM64T
export ARCH ?= x86_64

############################################################
## Build tools

# NASM executable name
export NASM ?= nasm

# GRUB mkrescue
GRUB_MKRESCUE ?= grub2-mkrescue

# QEMU used for `run` target
QEMU_RUN ?= qemu-system-$(ARCH)
