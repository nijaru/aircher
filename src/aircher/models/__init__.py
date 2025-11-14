"""Model routing and cost tracking."""

from datetime import datetime
from enum import Enum
from typing import Any

from langchain_anthropic import ChatAnthropic
from langchain_core.language_models.chat_models import BaseChatModel
from langchain_openai import ChatOpenAI
from loguru import logger
from pydantic import BaseModel, Field


class ModelProvider(str, Enum):
    """Supported LLM providers."""

    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    OLLAMA = "ollama"
    VLLM = "vllm"
    OPENROUTER = "openrouter"


class ModelTier(str, Enum):
    """Model tiers based on capability and cost."""

    LARGE = "large"  # Opus, GPT-4 - most capable, expensive
    MEDIUM = "medium"  # Sonnet, GPT-4o - balanced
    SMALL = "small"  # Haiku, GPT-4o-mini - fast, cheap
    LOCAL = "local"  # Ollama models - unlimited usage


class ModelConfig(BaseModel):
    """Configuration for a specific model."""

    name: str
    provider: ModelProvider
    tier: ModelTier
    cost_per_1k_input: float  # USD per 1k input tokens
    cost_per_1k_output: float  # USD per 1k output tokens
    context_window: int
    supports_streaming: bool = True
    max_retries: int = 3


# Pricing as of 2025-01 (verify before production use)
SUPPORTED_MODELS = {
    # OpenAI models
    "gpt-4o": ModelConfig(
        name="gpt-4o",
        provider=ModelProvider.OPENAI,
        tier=ModelTier.MEDIUM,
        cost_per_1k_input=0.0025,
        cost_per_1k_output=0.010,
        context_window=128000,
    ),
    "gpt-4o-mini": ModelConfig(
        name="gpt-4o-mini",
        provider=ModelProvider.OPENAI,
        tier=ModelTier.SMALL,
        cost_per_1k_input=0.00015,
        cost_per_1k_output=0.0006,
        context_window=128000,
    ),
    "gpt-4": ModelConfig(
        name="gpt-4",
        provider=ModelProvider.OPENAI,
        tier=ModelTier.LARGE,
        cost_per_1k_input=0.03,
        cost_per_1k_output=0.06,
        context_window=8192,
    ),
    # Anthropic models
    "claude-opus-4-20250514": ModelConfig(
        name="claude-opus-4-20250514",
        provider=ModelProvider.ANTHROPIC,
        tier=ModelTier.LARGE,
        cost_per_1k_input=0.015,
        cost_per_1k_output=0.075,
        context_window=200000,
    ),
    "claude-sonnet-4-20250514": ModelConfig(
        name="claude-sonnet-4-20250514",
        provider=ModelProvider.ANTHROPIC,
        tier=ModelTier.MEDIUM,
        cost_per_1k_input=0.003,
        cost_per_1k_output=0.015,
        context_window=200000,
    ),
    "claude-haiku-4-20250514": ModelConfig(
        name="claude-haiku-4-20250514",
        provider=ModelProvider.ANTHROPIC,
        tier=ModelTier.SMALL,
        cost_per_1k_input=0.0008,
        cost_per_1k_output=0.004,
        context_window=200000,
    ),
    # Ollama (local models - zero cost)
    "ollama/llama3.2": ModelConfig(
        name="ollama/llama3.2",
        provider=ModelProvider.OLLAMA,
        tier=ModelTier.LOCAL,
        cost_per_1k_input=0.0,
        cost_per_1k_output=0.0,
        context_window=128000,
        supports_streaming=True,
    ),
    # vLLM on fedora (RTX 4090) - zero cost
    "vllm/qwen2.5-32b": ModelConfig(
        name="vllm/qwen2.5-32b",
        provider=ModelProvider.VLLM,
        tier=ModelTier.MEDIUM,
        cost_per_1k_input=0.0,
        cost_per_1k_output=0.0,
        context_window=32768,
        supports_streaming=True,
    ),
    "vllm/llama-3.3-70b": ModelConfig(
        name="vllm/llama-3.3-70b",
        provider=ModelProvider.VLLM,
        tier=ModelTier.LARGE,
        cost_per_1k_input=0.0,
        cost_per_1k_output=0.0,
        context_window=128000,
        supports_streaming=True,
    ),
    "vllm/deepseek-coder-33b": ModelConfig(
        name="vllm/deepseek-coder-33b",
        provider=ModelProvider.VLLM,
        tier=ModelTier.MEDIUM,
        cost_per_1k_input=0.0,
        cost_per_1k_output=0.0,
        context_window=16384,
        supports_streaming=True,
    ),
    # OpenRouter - access to many models via single API
    "anthropic/claude-sonnet-4.5": ModelConfig(
        name="anthropic/claude-sonnet-4.5",
        provider=ModelProvider.OPENROUTER,
        tier=ModelTier.MEDIUM,
        cost_per_1k_input=0.003,
        cost_per_1k_output=0.015,
        context_window=200000,
    ),
    "anthropic/claude-opus-4": ModelConfig(
        name="anthropic/claude-opus-4",
        provider=ModelProvider.OPENROUTER,
        tier=ModelTier.LARGE,
        cost_per_1k_input=0.015,
        cost_per_1k_output=0.075,
        context_window=200000,
    ),
    "openai/gpt-4o": ModelConfig(
        name="openai/gpt-4o",
        provider=ModelProvider.OPENROUTER,
        tier=ModelTier.MEDIUM,
        cost_per_1k_input=0.0025,
        cost_per_1k_output=0.010,
        context_window=128000,
    ),
}


class UsageStats(BaseModel):
    """Token usage statistics."""

    input_tokens: int = 0
    output_tokens: int = 0
    total_tokens: int = 0
    estimated_cost: float = 0.0
    model_name: str = ""
    timestamp: datetime = Field(default_factory=datetime.now)


class SessionCostTracker(BaseModel):
    """Track costs per session."""

    session_id: str
    usage_by_model: dict[str, list[UsageStats]] = Field(default_factory=dict)
    total_cost: float = 0.0
    created_at: datetime = Field(default_factory=datetime.now)

    def add_usage(
        self,
        model_name: str,
        input_tokens: int,
        output_tokens: int,
    ) -> UsageStats:
        """Add usage statistics for a model call."""
        if model_name not in SUPPORTED_MODELS:
            logger.warning(f"Unknown model {model_name}, cannot track cost")
            return UsageStats(
                input_tokens=input_tokens,
                output_tokens=output_tokens,
                total_tokens=input_tokens + output_tokens,
                model_name=model_name,
            )

        model_config = SUPPORTED_MODELS[model_name]

        # Calculate cost
        input_cost = (input_tokens / 1000.0) * model_config.cost_per_1k_input
        output_cost = (output_tokens / 1000.0) * model_config.cost_per_1k_output
        total_cost = input_cost + output_cost

        usage = UsageStats(
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            total_tokens=input_tokens + output_tokens,
            estimated_cost=total_cost,
            model_name=model_name,
        )

        # Track by model
        if model_name not in self.usage_by_model:
            self.usage_by_model[model_name] = []
        self.usage_by_model[model_name].append(usage)

        # Update total
        self.total_cost += total_cost

        logger.debug(
            f"Added usage: {model_name} - {input_tokens} in, {output_tokens} out, "
            f"${total_cost:.6f} (session total: ${self.total_cost:.4f})"
        )

        return usage

    def get_summary(self) -> dict[str, Any]:
        """Get cost summary for this session."""
        total_input = sum(
            sum(u.input_tokens for u in usages)
            for usages in self.usage_by_model.values()
        )
        total_output = sum(
            sum(u.output_tokens for u in usages)
            for usages in self.usage_by_model.values()
        )

        return {
            "session_id": self.session_id,
            "total_cost": round(self.total_cost, 4),
            "total_input_tokens": total_input,
            "total_output_tokens": total_output,
            "total_tokens": total_input + total_output,
            "models_used": list(self.usage_by_model.keys()),
            "call_count": sum(len(usages) for usages in self.usage_by_model.values()),
        }


class ModelRouter:
    """Smart model router with cost tracking and fallback."""

    def __init__(
        self,
        default_model: str = "gpt-4o-mini",
        enable_fallback: bool = True,
        session_id: str | None = None,
    ):
        self.default_model = default_model
        self.enable_fallback = enable_fallback
        self.session_id = session_id or "default"

        # Cost tracking
        self.cost_tracker = SessionCostTracker(session_id=self.session_id)

        # Fallback chain: large -> medium -> small -> local
        self.fallback_chain = self._build_fallback_chain()

        logger.info(
            f"ModelRouter initialized: default={default_model}, fallback={enable_fallback}"
        )

    def _build_fallback_chain(self) -> list[str]:
        """Build fallback chain based on available models."""
        chain = []

        # Add models by tier
        for tier in [
            ModelTier.LARGE,
            ModelTier.MEDIUM,
            ModelTier.SMALL,
            ModelTier.LOCAL,
        ]:
            tier_models = [
                name for name, config in SUPPORTED_MODELS.items() if config.tier == tier
            ]
            chain.extend(tier_models)

        return chain

    def get_model(
        self,
        model_name: str | None = None,
        temperature: float = 0.7,
        max_tokens: int | None = None,
    ) -> BaseChatModel:
        """Get a configured LLM model.

        Args:
            model_name: Specific model to use (or default)
            temperature: Model temperature
            max_tokens: Maximum output tokens

        Returns:
            Configured LangChain chat model
        """
        model_name = model_name or self.default_model

        if model_name not in SUPPORTED_MODELS:
            logger.warning(
                f"Unknown model {model_name}, falling back to {self.default_model}"
            )
            model_name = self.default_model

        config = SUPPORTED_MODELS[model_name]

        try:
            if config.provider == ModelProvider.OPENAI:
                return ChatOpenAI(
                    model=config.name,
                    temperature=temperature,
                    max_tokens=max_tokens,
                )
            elif config.provider == ModelProvider.ANTHROPIC:
                return ChatAnthropic(
                    model=config.name,
                    temperature=temperature,
                    max_tokens=max_tokens or 4096,
                )
            elif config.provider == ModelProvider.OLLAMA:
                # Ollama uses ChatOpenAI with custom base_url
                return ChatOpenAI(
                    model=config.name.replace("ollama/", ""),
                    base_url="http://localhost:11434/v1",
                    api_key="ollama",  # Ollama doesn't need real API key
                    temperature=temperature,
                    max_tokens=max_tokens,
                )
            elif config.provider == ModelProvider.VLLM:
                # vLLM on fedora via Tailscale (OpenAI-compatible API)
                return ChatOpenAI(
                    model=config.name.replace("vllm/", ""),
                    base_url="http://100.93.39.25:8000/v1",
                    api_key="vllm",  # vLLM doesn't need real API key
                    temperature=temperature,
                    max_tokens=max_tokens,
                )
            elif config.provider == ModelProvider.OPENROUTER:
                # OpenRouter - access to many models via single API
                import os

                return ChatOpenAI(
                    model=config.name,  # Keep full name (e.g., "anthropic/claude-sonnet-4.5")
                    base_url="https://openrouter.ai/api/v1",
                    api_key=os.getenv("OPENROUTER_API_KEY"),
                    temperature=temperature,
                    max_tokens=max_tokens,
                    default_headers={
                        "HTTP-Referer": "https://github.com/nijaru/aircher",
                        "X-Title": "Aircher",
                    },
                )
            else:
                raise ValueError(f"Unsupported provider: {config.provider}")

        except Exception as e:
            logger.error(f"Failed to initialize {model_name}: {e}")

            if self.enable_fallback:
                return self._try_fallback(model_name, temperature, max_tokens)
            else:
                raise

    def _try_fallback(
        self,
        failed_model: str,
        temperature: float,
        max_tokens: int | None,
    ) -> BaseChatModel:
        """Try fallback models when primary fails."""
        # Find next model in fallback chain
        try:
            failed_idx = self.fallback_chain.index(failed_model)
            remaining_models = self.fallback_chain[failed_idx + 1 :]
        except ValueError:
            remaining_models = self.fallback_chain

        for fallback_model in remaining_models:
            try:
                logger.info(f"Trying fallback model: {fallback_model}")
                config = SUPPORTED_MODELS[fallback_model]

                if config.provider == ModelProvider.OPENAI:
                    return ChatOpenAI(
                        model=config.name,
                        temperature=temperature,
                        max_tokens=max_tokens,
                    )
                elif config.provider == ModelProvider.ANTHROPIC:
                    return ChatAnthropic(
                        model=config.name,
                        temperature=temperature,
                        max_tokens=max_tokens or 4096,
                    )
                elif config.provider == ModelProvider.OLLAMA:
                    return ChatOpenAI(
                        model=config.name.replace("ollama/", ""),
                        base_url="http://localhost:11434/v1",
                        api_key="ollama",
                        temperature=temperature,
                        max_tokens=max_tokens,
                    )
                elif config.provider == ModelProvider.VLLM:
                    return ChatOpenAI(
                        model=config.name.replace("vllm/", ""),
                        base_url="http://100.93.39.25:8000/v1",
                        api_key="vllm",
                        temperature=temperature,
                        max_tokens=max_tokens,
                    )
                elif config.provider == ModelProvider.OPENROUTER:
                    import os

                    return ChatOpenAI(
                        model=config.name,
                        base_url="https://openrouter.ai/api/v1",
                        api_key=os.getenv("OPENROUTER_API_KEY"),
                        temperature=temperature,
                        max_tokens=max_tokens,
                        default_headers={
                            "HTTP-Referer": "https://github.com/nijaru/aircher",
                            "X-Title": "Aircher",
                        },
                    )
            except Exception as e:
                logger.warning(f"Fallback {fallback_model} also failed: {e}")
                continue

        raise RuntimeError("All fallback models failed")

    def track_usage(
        self,
        model_name: str,
        input_tokens: int,
        output_tokens: int,
    ) -> UsageStats:
        """Track token usage and cost for a model call."""
        return self.cost_tracker.add_usage(model_name, input_tokens, output_tokens)

    def get_cost_summary(self) -> dict[str, Any]:
        """Get cost summary for current session."""
        return self.cost_tracker.get_summary()

    def select_model_for_task(self, task_type: str) -> str:
        """Select appropriate model based on task complexity.

        Args:
            task_type: Type of task ("main_agent", "sub_agent", "simple_query", etc.)

        Returns:
            Model name
        """
        # Task-based routing
        routing = {
            "main_agent": ModelTier.MEDIUM,  # gpt-4o, sonnet
            "sub_agent": ModelTier.SMALL,  # gpt-4o-mini, haiku
            "simple_query": ModelTier.SMALL,
            "complex_reasoning": ModelTier.LARGE,  # opus, gpt-4
            "code_generation": ModelTier.MEDIUM,
            "code_review": ModelTier.SMALL,
        }

        desired_tier = routing.get(task_type, ModelTier.MEDIUM)

        # Find first available model of desired tier
        for name, config in SUPPORTED_MODELS.items():
            if config.tier == desired_tier:
                return name

        # Fallback to default
        return self.default_model
