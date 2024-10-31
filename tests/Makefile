SHELL = /usr/bin/bash
.SHELLFLAGS = -o pipefail -c

TESTFLOAT_GEN = $(HOME)/opt/TestFloat-3e/build/Linux-x86_64-GCC/testfloat_gen
TESTFLOAT_VER = $(HOME)/opt/TestFloat-3e/build/Linux-x86_64-GCC/testfloat_ver

BOLD_GREEN=\033[1;32m
NO_COLOR=\033[0m

.PHONY: all testfloat

all: testfloat

testfloat:
	RUSTFLAGS=-Awarnings cd ../soft_float/ && cargo build --release --bin testfloat
	@echo -e "${BOLD_GREEN}Testing SoftFloat16${NO_COLOR}"
	# $(TESTFLOAT_GEN) f32_to_f16 | target/release/testfloat f32_to_softfloat16 | $(TESTFLOAT_VER) f32_to_f16
	$(TESTFLOAT_GEN) f16_add | ../soft_float/target/release/testfloat softfloat16_add | $(TESTFLOAT_VER) f16_add
	$(TESTFLOAT_GEN) f16_sub | ../soft_float/target/release/testfloat softfloat16_sub | $(TESTFLOAT_VER) f16_sub
	$(TESTFLOAT_GEN) f16_mul | ../soft_float/target/release/testfloat softfloat16_mul | $(TESTFLOAT_VER) f16_mul
