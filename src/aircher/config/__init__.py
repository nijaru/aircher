"""Configuration management for Aircher."""

from functools import lru_cache
from pathlib import Path
from typing import Optional

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Application settings."""

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        env_prefix="AIRCHER_",
    )

    # Agent Configuration
    agent_mode: str = Field(default="read", description="Agent mode: read/edit/turbo")
    bypass_safety: bool = Field(default=False, description="Bypass safety checks")

    # Model Configuration
    default_model: str = Field(default="gpt-4o-mini", description="Default LLM model")
    openai_api_key: str | None = Field(default=None, description="OpenAI API key")
    anthropic_api_key: str | None = Field(default=None, description="Anthropic API key")
    vllm_base_url: str = Field(
        default="http://100.93.39.25:8000/v1", description="vLLM base URL"
    )

    # Database Configuration
    data_dir: Path = Field(default=Path("./data"), description="Data directory")
    chroma_persist_dir: Path = Field(
        default=Path("./data/chroma"), description="ChromaDB persist directory"
    )

    # ACP Protocol
    acp_host: str = Field(default="localhost", description="ACP server host")
    acp_port: int = Field(default=8080, description="ACP server port")

    # Development
    debug: bool = Field(default=False, description="Enable debug logging")
    log_level: str = Field(default="INFO", description="Log level")


@lru_cache
def get_settings() -> Settings:
    """Get cached settings instance."""
    return Settings()
