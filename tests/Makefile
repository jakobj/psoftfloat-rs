SHELL = /usr/bin/bash
.SHELLFLAGS = -o pipefail -c

TESTFLOAT_GEN = $(HOME)/opt/TestFloat-3e/build/Linux-x86_64-GCC/testfloat_gen
TESTFLOAT_VER = $(HOME)/opt/TestFloat-3e/build/Linux-x86_64-GCC/testfloat_ver
SOFTFLOAT_TESTFLOAT = ../soft_float/target/release/testfloat

BOLD=\033[1m
BOLD_GREEN=\033[1;32m
NO_COLOR=\033[0m
COLOR_NO_ERRORS=GREP_COLORS='ms=01;32' grep --color "no errors"

.PHONY: all testfloat

all: testfloat

testfloat:
	RUSTFLAGS=-Awarnings cd ../soft_float/ && cargo build --release --bin testfloat
	@echo -e "${BOLD}Testing SoftFloat16${NO_COLOR}"
	@$(TESTFLOAT_GEN) f32_to_f16 -level 2 | $(SOFTFLOAT_TESTFLOAT) f32_to_softfloat16 | $(TESTFLOAT_VER) f32_to_f16 | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_to_f32 -level 2 | $(SOFTFLOAT_TESTFLOAT) softfloat16_to_f32 | $(TESTFLOAT_VER) f16_to_f32 | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_add -level 2 | $(SOFTFLOAT_TESTFLOAT) softfloat16_add | $(TESTFLOAT_VER) f16_add | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_sub -level 2 | $(SOFTFLOAT_TESTFLOAT) softfloat16_sub | $(TESTFLOAT_VER) f16_sub | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_mul -level 2 | $(SOFTFLOAT_TESTFLOAT) softfloat16_mul | $(TESTFLOAT_VER) f16_mul | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_div -level 2 | $(SOFTFLOAT_TESTFLOAT) softfloat16_div | $(TESTFLOAT_VER) f16_div | ${COLOR_NO_ERRORS}
