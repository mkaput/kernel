.PHONY: $(LIBKERNEL_A)

$(LIBKERNEL_A):
	$(CARGO) build --target $(TARGET)
