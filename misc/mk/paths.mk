############################################################
## Paths

export ROOT_DIR  := $(CURDIR)
export BUILD_DIR := $(ROOT_DIR)/target
export MISC_DIR  := $(ROOT_DIR)/misc

export TARGET_BOOT_DIR   := $(ROOT_DIR)/boot/$(TARGET)
export TARGET_BOOT_BUILD := $(BUILD_DIR)/$(TARGET)/boot
export KERNEL_BIN 		 := $(TARGET_BOOT_BUILD)/kernel.bin

export TARGET_RUST_BUILD := $(BUILD_DIR)/$(TARGET)/$(CARGO_MODE)
export LIBKERNEL_A		 := $(TARGET_RUST_BUILD)/libkernel.a

export TARGET_BOOK_BUILD := $(BUILD_DIR)/book

export GRUB_CFG_DIR := $(MISC_DIR)/grub

export PACKAGE_ISO := $(BUILD_DIR)/iso/os.iso
