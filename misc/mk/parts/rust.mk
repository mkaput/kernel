.PHONY: $(LIBKERNEL_A)

$(LIBKERNEL_A):
	RUST_TARGET_PATH="$(ROOT_DIR)/.cargo" $(CARGO) build --target $(TARGET)
