X86_64_LINKER_SCRIPT    := $(TARGET_BOOT_DIR)/linker.ld
X86_64_ASM_SRC          := $(wildcard $(TARGET_BOOT_DIR)/*.asm)
X86_64_ASM_OBJ          := $(patsubst $(TARGET_BOOT_DIR)/%.asm, \
                                      $(TARGET_BOOT_BUILD)/%.o, \
                                      $(X86_64_ASM_SRC))

.PHONY: clean-boot-x86_64

clean-boot-x86_64:
	rm -rfd $(TARGET_BOOT_BUILD)

$(KERNEL_BIN): $(LIBKERNEL_A) $(X86_64_ASM_OBJ) $(X86_64_LINKER_SCRIPT)
	$(LD) -n --gc-sections -T $(X86_64_LINKER_SCRIPT) -o $(KERNEL_BIN) \
		$(X86_64_ASM_OBJ) $(LIBKERNEL_A)

$(TARGET_BOOT_BUILD)/%.o: $(TARGET_BOOT_DIR)/%.asm
	mkdir -p $(@D)
	$(NASM) -felf64 $< -o $@
