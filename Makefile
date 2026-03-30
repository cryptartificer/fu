# fu — fast terminal plotting
# ──────────────────────────────────────────────────────────────────

CARGO   := cargo
BINARY  := target/release/fu
DEBUG   := target/debug/fu
PREFIX  := /usr/local

# ── Build ────────────────────────────────────────────────────────

.PHONY: build release debug clean

build: release

release:
	$(CARGO) build --release

debug:
	$(CARGO) build

clean:
	$(CARGO) clean

# ── Test ─────────────────────────────────────────────────────────

.PHONY: test test-verbose test-lib test-bin

test:
	$(CARGO) test

test-verbose:
	$(CARGO) test -- --nocapture

test-lib:
	$(CARGO) test --lib

test-bin:
	$(CARGO) test --bin fu

# ── Lint & Format ────────────────────────────────────────────────

.PHONY: check clippy fmt fmt-check lint

check:
	$(CARGO) check

clippy:
	$(CARGO) clippy -- -D warnings

fmt:
	$(CARGO) fmt

fmt-check:
	$(CARGO) fmt -- --check

lint: fmt-check clippy

# ── Benchmarks ───────────────────────────────────────────────────

.PHONY: bench

bench:
	$(CARGO) bench

# ── Showcase & Images ─────────────────────────────────────────────

.PHONY: showcase compare images

showcase: release
	@for f in showcase/[0-9]*.sh; do \
		echo "" && echo "═══ Running $$f ═══" && echo "" && bash "$$f"; \
	done

compare: release
	@bash showcase/compare.sh $(ARGS)

images: release
	@python3 img/render.py

# ── Install / Uninstall ─────────────────────────────────────────

.PHONY: install uninstall

install: release
	install -d $(DESTDIR)$(PREFIX)/bin
	install -m 755 $(BINARY) $(DESTDIR)$(PREFIX)/bin/fu

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/fu

# ── Run shortcuts ────────────────────────────────────────────────

.PHONY: run

run: debug
	@echo "Usage: make run ARGS=\"data.csv\""
	@if [ -n "$(ARGS)" ]; then $(DEBUG) $(ARGS); fi

# ── Documentation ────────────────────────────────────────────────

.PHONY: doc doc-open

doc:
	$(CARGO) doc --no-deps

doc-open:
	$(CARGO) doc --no-deps --open

# ── CI-style full check ─────────────────────────────────────────

.PHONY: ci

ci: fmt-check clippy test
	@echo ""
	@echo "All checks passed."

# ── Size report ──────────────────────────────────────────────────

.PHONY: size

size: release
	@echo ""
	@ls -lh $(BINARY) | awk '{ printf "Binary size: %s\n", $$5 }'

# ── Line count ───────────────────────────────────────────────────

.PHONY: loc

loc:
	@echo ""
	@echo "Lines of code:"
	@if command -v tokei >/dev/null 2>&1; then \
		tokei src/; \
	else \
		find src -name '*.rs' | xargs wc -l | sort -n; \
	fi

# ── Help ─────────────────────────────────────────────────────────

.PHONY: help

help:
	@echo "fu — fast terminal plotting"
	@echo ""
	@echo "Build:"
	@echo "  make              Build release binary"
	@echo "  make debug        Build debug binary"
	@echo "  make clean        Remove build artifacts"
	@echo ""
	@echo "Test:"
	@echo "  make test         Run all tests"
	@echo "  make test-verbose Run tests with output"
	@echo "  make test-lib     Run library tests only"
	@echo "  make test-bin     Run binary tests only"
	@echo ""
	@echo "Lint:"
	@echo "  make lint         Format check + clippy"
	@echo "  make fmt          Auto-format code"
	@echo "  make fmt-check    Check formatting (no changes)"
	@echo "  make check        Compile check (no codegen)"
	@echo "  make clippy       Run clippy lints"
	@echo ""
	@echo "Bench:"
	@echo "  make bench        Run all criterion benchmarks"
	@echo ""
	@echo "Run:"
	@echo "  make run ARGS=..  Run fu with arguments"
	@echo "  make showcase     Run showcase scripts (showcase/)"
	@echo "  make compare      Visual comparison: fu vs uplot"
	@echo "  make images       Regenerate README images (img/)"
	@echo ""
	@echo "Other:"
	@echo "  make ci           Full CI check (fmt, clippy, test)"
	@echo "  make install      Install to $(PREFIX)/bin"
	@echo "  make uninstall    Remove from $(PREFIX)/bin"
	@echo "  make doc          Generate rustdoc"
	@echo "  make doc-open     Generate and open rustdoc"
	@echo "  make size         Binary size report"
	@echo "  make loc          Lines of code"

.DEFAULT_GOAL := help
