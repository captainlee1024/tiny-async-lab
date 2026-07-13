.PHONY: book ci docs

ci: docs

docs: book
	markdownlint-cli2
	typos
	lychee --offline --include-fragments=full --no-progress .
	mdbook test docs
	scripts/check-mermaid.sh

book:
	mdbook build docs
