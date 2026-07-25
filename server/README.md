# server

Control-plane server (FastAPI, deployed to Cloud Run). M1 provides only
liveness/readiness + a root descriptor. Auth, catalog/search, agent registry,
and hybrid token signing land in M2.

## Local dev
```bash
python -m venv .venv && . .venv/bin/activate
pip install -e ".[dev]"
cp .env.example .env
uvicorn app.main:app --reload --port 8080
```

## Checks (what CI runs)
```bash
ruff check .
ruff format --check .
pytest
```
