PROTO_DIR := proto
PROTO_FILE := $(PROTO_DIR)/event.proto

ELIXIR_OUT := business_logic/elixir/lib/business_logic/proto
RUST_OUT := ingress/rust/src/proto

.PHONY: all proto elixir rust clean

all: proto

proto: elixir rust


elixir:
	@echo "Generating Elixir protobuf modules..."
	@mkdir -p $(ELIXIR_OUT)
	PATH="$(HOME)/.mix/escripts:$$PATH" protoc --elixir_out=$(ELIXIR_OUT) \
	       --proto_path=$(PROTO_DIR) \
	       $(PROTO_FILE)




rust:
	@echo "Generating Rust gRPC modules..."
	@mkdir -p $(RUST_OUT)
	@cd ingress/rust && \
		cargo build


clean:
	@echo "Cleaning generated Elixir files..."
	@rm -rf $(ELIXIR_OUT)

	@echo "Cleaning Rust build artifacts..."
	@cd ingress/rust && cargo clean
