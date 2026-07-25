.DEFAULT_GOAL := help

help: ## List targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN{FS=":.*?## "}{printf "  %-16s %s\n",$$1,$$2}'

dev: ## Run the server locally with reload
	cd server && uvicorn app.main:app --reload --port 8080

server-check: ## Lint + test the server (what CI runs)
	cd server && ruff check . && ruff format --check . && pytest

infra-check: ## Validate Terraform
	cd infra && terraform fmt -check -recursive && terraform init -backend=false && terraform validate

agent-check: ## Format/lint/build the agent
	cd agent && cargo fmt --check && cargo clippy -- -D warnings && cargo build

fmt: ## Auto-format everything
	cd server && ruff format . && ruff check --fix .
	cd infra && terraform fmt -recursive
	cd agent && cargo fmt
