"""Minimal control-plane server (M1 skeleton).

For now this only exposes liveness/readiness endpoints and a root descriptor —
the "empty HTTPS server" Cloud Run stands up. Auth, catalog, search, registry,
and token signing arrive in M2. App-layer auth (Firebase JWT + agent tokens) is
the real access gate; Cloud Run itself is public so browsers and agents can
reach it.
"""

from __future__ import annotations

import logging
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.config import settings

logging.basicConfig(level=logging.INFO, format="%(levelname)s %(name)s %(message)s")
log = logging.getLogger(settings.service_name)


@asynccontextmanager
async def lifespan(_: FastAPI) -> AsyncIterator[None]:
    log.info("starting %s v%s (%s)", settings.service_name, settings.version, settings.environment)
    yield
    log.info("shutting down")


app = FastAPI(title=settings.service_name, version=settings.version, lifespan=lifespan)

# Deny-by-default CORS: only explicitly configured origins are allowed.
if settings.cors_allow_origins:
    app.add_middleware(
        CORSMiddleware,
        allow_origins=settings.cors_allow_origins,
        allow_credentials=True,
        allow_methods=["GET", "POST"],
        allow_headers=["Authorization", "Content-Type"],
    )


@app.get("/", tags=["meta"])
async def root() -> dict[str, str]:
    return {"service": settings.service_name, "version": settings.version}


@app.get("/healthz", tags=["meta"])
async def healthz() -> dict[str, str]:
    """Liveness: process is up."""
    return {"status": "ok"}


@app.get("/readyz", tags=["meta"])
async def readyz() -> dict[str, str]:
    """Readiness: dependencies reachable. Trivial until M2 adds the DB."""
    return {"status": "ready"}
