SHELL = /usr/bin/bash
.SHELLFLAGS = -o pipefail -c

TESTFLOAT_GEN = $(HOME)/opt/TestFloat-3e/build/Linux-x86_64-GCC/testfloat_gen -level 2
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
	@$(TESTFLOAT_GEN) f16_rem | $(SOFTFLOAT_TESTFLOAT) softfloat16_rem | $(TESTFLOAT_VER) f16_rem | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) i32_to_f16 | $(SOFTFLOAT_TESTFLOAT) i32_to_softfloat16 | $(TESTFLOAT_VER) i32_to_f16 | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_to_i32 | $(SOFTFLOAT_TESTFLOAT) softfloat16_to_i32 | $(TESTFLOAT_VER) f16_to_i32 | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f32_to_f16 | $(SOFTFLOAT_TESTFLOAT) f32_to_softfloat16 | $(TESTFLOAT_VER) f32_to_f16 | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_to_f32 | $(SOFTFLOAT_TESTFLOAT) softfloat16_to_f32 | $(TESTFLOAT_VER) f16_to_f32 | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_add | $(SOFTFLOAT_TESTFLOAT) softfloat16_add | $(TESTFLOAT_VER) f16_add | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_sub | $(SOFTFLOAT_TESTFLOAT) softfloat16_sub | $(TESTFLOAT_VER) f16_sub | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_mul | $(SOFTFLOAT_TESTFLOAT) softfloat16_mul | $(TESTFLOAT_VER) f16_mul | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_div | $(SOFTFLOAT_TESTFLOAT) softfloat16_div | $(TESTFLOAT_VER) f16_div | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_roundToInt | $(SOFTFLOAT_TESTFLOAT) softfloat16_round | $(TESTFLOAT_VER) f16_roundToInt | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_eq | $(SOFTFLOAT_TESTFLOAT) softfloat16_eq | $(TESTFLOAT_VER) f16_eq | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_le | $(SOFTFLOAT_TESTFLOAT) softfloat16_le | $(TESTFLOAT_VER) f16_le | ${COLOR_NO_ERRORS}
	@$(TESTFLOAT_GEN) f16_lt | $(SOFTFLOAT_TESTFLOAT) softfloat16_lt | $(TESTFLOAT_VER) f16_lt | ${COLOR_NO_ERRORS}
