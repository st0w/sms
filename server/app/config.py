"""Application configuration, sourced entirely from the environment.

No secrets are hard-coded. In production these come from Cloud Run env vars
wired to Secret Manager; locally they come from a `.env` (see `.env.example`).
"""

from __future__ import annotations

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_prefix="APP_", extra="ignore")

    service_name: str = "music-server"
    version: str = "0.1.0"
    environment: str = Field(default="local")  # local | staging | prod

    # Cloud Run injects PORT; default matches Cloud Run's default.
    port: int = 8080

    # Comma-separated list of allowed browser origins for CORS. Empty = none.
    cors_allow_origins: list[str] = Field(default_factory=list)

    # Populated in M2. Present here so the shape is stable.
    database_url: str | None = None
    firebase_project_id: str | None = None


settings = Settings()
