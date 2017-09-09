X86_64_LINKER_SCRIPT    := $(ARCH_BOOT_DIR)/linker.ld
X86_64_ASM_SRC          := $(wildcard $(ARCH_BOOT_DIR)/*.asm)
X86_64_ASM_OBJ          := $(patsubst $(ARCH_BOOT_DIR)/%.asm, \
                                      $(ARCH_BOOT_BUILD)/%.o, \
                                      $(X86_64_ASM_SRC))

.PHONY: clean-boot-x86_64

clean-boot-x86_64:
	rm -rfd $(ARCH_BOOT_BUILD)

$(KERNEL_BIN): $(X86_64_ASM_OBJ) $(X86_64_LINKER_SCRIPT)
	$(LD) -n -T $(X86_64_LINKER_SCRIPT) -o $(KERNEL_BIN) $(X86_64_ASM_OBJ)

$(ARCH_BOOT_BUILD)/%.o: $(ARCH_BOOT_DIR)/%.asm
	mkdir -p $(@D)
	$(NASM) -felf64 $< -o $@
