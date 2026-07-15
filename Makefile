TOOLS_DIR := $(CURDIR)/.tools
TOOLS_BIN := $(TOOLS_DIR)/bin
NODE_TOOLS_BIN := $(CURDIR)/node_modules/.bin

MDBOOK_VERSION := 0.5.2
MDBOOK_LINKCHECK2_VERSION := 0.11.0
MDBOOK_MERMAID_VERSION := 0.17.0
TYPOS_VERSION := 1.48.0
LYCHEE_VERSION := 0.24.2
MARKDOWNLINT_VERSION := 0.23.0
MERMAID_CLI_VERSION := 11.6.0
NODE_VERSION := $(shell sed -n '1p' .node-version)
NPM_VERSION := 11.16.0

MDBOOK := $(TOOLS_BIN)/mdbook
MARKDOWNLINT := $(NODE_TOOLS_BIN)/markdownlint-cli2
TYPOS := $(TOOLS_BIN)/typos
LYCHEE := $(TOOLS_BIN)/lychee

export PATH := $(TOOLS_BIN):$(NODE_TOOLS_BIN):$(PATH)

.PHONY: agent-workflow-check book book-preview ci docs toolchain-check tools tools-check upstream

ci: docs agent-workflow-check

agent-workflow-check: toolchain-check
	node scripts/check-agent-workflow-review.mjs --validate

docs: book
	"$(MARKDOWNLINT)"
	"$(TYPOS)"
	"$(LYCHEE)" --offline --include-fragments=full --no-progress .
	"$(MDBOOK)" test docs
	scripts/check-mermaid.sh

book: tools-check
	"$(MDBOOK)" build docs

book-preview: tools-check
	"$(MDBOOK)" serve docs --hostname 127.0.0.1 --port 3000

upstream:
	scripts/checkout-upstream.sh

tools: toolchain-check
	cargo install --locked --root "$(TOOLS_DIR)" \
		"mdbook@$(MDBOOK_VERSION)" \
		"mdbook-linkcheck2@$(MDBOOK_LINKCHECK2_VERSION)" \
		"mdbook-mermaid@$(MDBOOK_MERMAID_VERSION)" \
		"typos-cli@$(TYPOS_VERSION)" \
		"lychee@$(LYCHEE_VERSION)"
	npm ci --ignore-scripts --no-audit --no-fund

tools-check: toolchain-check
	@check_version() { \
		tool="$$1"; \
		expected="$$2"; \
		if [ ! -x "$$tool" ]; then \
			echo "缺少仓库本地工具：$$tool" >&2; \
			echo "请先运行 make tools。" >&2; \
			exit 1; \
		fi; \
		actual="$$("$$tool" --version 2>&1)"; \
		case "$$actual" in \
			"$$expected"*) ;; \
			*) \
				echo "工具版本不匹配：$$tool" >&2; \
				echo "需要 $$expected，当前为 $$actual。" >&2; \
				echo "请重新运行 make tools。" >&2; \
				exit 1; \
				;; \
		esac; \
	}; \
	check_version "$(MDBOOK)" "mdbook v$(MDBOOK_VERSION)"; \
	check_version "$(TOOLS_BIN)/mdbook-linkcheck2" "mdbook-linkcheck2 $(MDBOOK_LINKCHECK2_VERSION)"; \
	check_version "$(TOOLS_BIN)/mdbook-mermaid" "mdbook-mermaid $(MDBOOK_MERMAID_VERSION)"; \
	check_version "$(MARKDOWNLINT)" "markdownlint-cli2 v$(MARKDOWNLINT_VERSION)"; \
	check_version "$(TYPOS)" "typos-cli $(TYPOS_VERSION)"; \
	check_version "$(LYCHEE)" "lychee $(LYCHEE_VERSION)"; \
	check_version "$(NODE_TOOLS_BIN)/mmdc" "$(MERMAID_CLI_VERSION)"

toolchain-check:
	@if ! command -v node >/dev/null 2>&1; then \
		echo "未找到 Node.js；请先安装 .node-version 固定的版本。" >&2; \
		exit 1; \
	fi; \
	actual="$$(node --version)"; \
	expected="v$(NODE_VERSION)"; \
	if [ "$$actual" != "$$expected" ]; then \
		echo "Node.js 版本不匹配：需要 $$expected，当前为 $$actual。" >&2; \
		exit 1; \
	fi; \
	if ! command -v npm >/dev/null 2>&1; then \
		echo "未找到 npm $(NPM_VERSION)。" >&2; \
		exit 1; \
	fi; \
	actual="$$(npm --version)"; \
	if [ "$$actual" != "$(NPM_VERSION)" ]; then \
		echo "npm 版本不匹配：需要 $(NPM_VERSION)，当前为 $$actual。" >&2; \
		exit 1; \
	fi
