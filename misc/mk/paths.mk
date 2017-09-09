############################################################
## Paths

export ROOT_DIR    := $(CURDIR)
export BUILD_DIR   := $(ROOT_DIR)/build
export MISC_DIR    := $(ROOT_DIR)/misc

export ARCH_BOOT_DIR   := $(ROOT_DIR)/boot/$(ARCH)
export ARCH_BOOT_BUILD := $(BUILD_DIR)/boot/$(ARCH)
export KERNEL_BIN := $(ARCH_BOOT_BUILD)/kernel.bin

export GRUB_CFG_DIR := $(MISC_DIR)/grub

export PACKAGE_ISO := $(BUILD_DIR)/iso/os.iso
