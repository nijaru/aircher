"""Tests for model router."""

import pytest

from aircher.models import (
    ModelConfig,
    ModelProvider,
    ModelRouter,
    ModelTier,
    SessionCostTracker,
    SUPPORTED_MODELS,
)


class TestModelConfig:
    """Test model configuration."""

    def test_model_config_creation(self):
        """Test creating a model config."""
        config = ModelConfig(
            name="test-model",
            provider=ModelProvider.OPENAI,
            tier=ModelTier.SMALL,
            cost_per_1k_input=0.001,
            cost_per_1k_output=0.002,
            context_window=8192,
        )

        assert config.name == "test-model"
        assert config.provider == ModelProvider.OPENAI
        assert config.tier == ModelTier.SMALL
        assert config.cost_per_1k_input == 0.001
        assert config.cost_per_1k_output == 0.002

    def test_supported_models_defined(self):
        """Test that supported models are defined."""
        assert len(SUPPORTED_MODELS) > 0
        assert "gpt-4o-mini" in SUPPORTED_MODELS
        assert "gpt-4o" in SUPPORTED_MODELS


class TestSessionCostTracker:
    """Test cost tracking."""

    def test_cost_tracker_creation(self):
        """Test creating a cost tracker."""
        tracker = SessionCostTracker(session_id="test")

        assert tracker.session_id == "test"
        assert tracker.total_cost == 0.0
        assert len(tracker.usage_by_model) == 0

    def test_add_usage_gpt4o_mini(self):
        """Test adding usage for gpt-4o-mini."""
        tracker = SessionCostTracker(session_id="test")

        # gpt-4o-mini: $0.00015 / 1k input, $0.0006 / 1k output
        usage = tracker.add_usage("gpt-4o-mini", input_tokens=1000, output_tokens=500)

        assert usage.input_tokens == 1000
        assert usage.output_tokens == 500
        assert usage.total_tokens == 1500

        # Cost calculation: (1000/1000 * 0.00015) + (500/1000 * 0.0006) = 0.00015 + 0.0003 = 0.00045
        assert abs(usage.estimated_cost - 0.00045) < 0.000001
        assert abs(tracker.total_cost - 0.00045) < 0.000001

    def test_add_usage_multiple_models(self):
        """Test adding usage for multiple models."""
        tracker = SessionCostTracker(session_id="test")

        tracker.add_usage("gpt-4o-mini", input_tokens=1000, output_tokens=500)
        tracker.add_usage("gpt-4o", input_tokens=2000, output_tokens=1000)

        assert len(tracker.usage_by_model) == 2
        assert "gpt-4o-mini" in tracker.usage_by_model
        assert "gpt-4o" in tracker.usage_by_model

    def test_get_summary(self):
        """Test getting cost summary."""
        tracker = SessionCostTracker(session_id="test")

        tracker.add_usage("gpt-4o-mini", input_tokens=1000, output_tokens=500)
        tracker.add_usage("gpt-4o-mini", input_tokens=2000, output_tokens=1000)

        summary = tracker.get_summary()

        assert summary["session_id"] == "test"
        assert summary["total_input_tokens"] == 3000
        assert summary["total_output_tokens"] == 1500
        assert summary["total_tokens"] == 4500
        assert summary["call_count"] == 2
        assert "gpt-4o-mini" in summary["models_used"]


class TestModelRouter:
    """Test model router."""

    def test_router_creation(self):
        """Test creating a model router."""
        router = ModelRouter(default_model="gpt-4o-mini", session_id="test")

        assert router.default_model == "gpt-4o-mini"
        assert router.session_id == "test"
        assert router.enable_fallback is True

    def test_select_model_for_task(self):
        """Test task-based model selection."""
        router = ModelRouter(default_model="gpt-4o-mini")

        # Main agent should use medium tier
        main_model = router.select_model_for_task("main_agent")
        config = SUPPORTED_MODELS[main_model]
        assert config.tier == ModelTier.MEDIUM

        # Sub-agent should use small tier
        sub_model = router.select_model_for_task("sub_agent")
        config = SUPPORTED_MODELS[sub_model]
        assert config.tier == ModelTier.SMALL

        # Complex reasoning should use large tier
        complex_model = router.select_model_for_task("complex_reasoning")
        config = SUPPORTED_MODELS[complex_model]
        assert config.tier == ModelTier.LARGE

    def test_track_usage(self):
        """Test tracking usage."""
        router = ModelRouter(session_id="test")

        usage = router.track_usage("gpt-4o-mini", input_tokens=1000, output_tokens=500)

        assert usage.input_tokens == 1000
        assert usage.output_tokens == 500
        assert usage.estimated_cost > 0

        summary = router.get_cost_summary()
        assert summary["total_cost"] > 0
        assert summary["total_tokens"] == 1500

    def test_get_model_openai(self):
        """Test getting an OpenAI model."""
        router = ModelRouter(default_model="gpt-4o-mini")

        # This requires API key, so we just test it returns a model object
        try:
            model = router.get_model("gpt-4o-mini", temperature=0.5)
            assert model is not None
        except Exception:
            # Expected if no API key configured
            pytest.skip("Requires OpenAI API key")

    def test_fallback_chain(self):
        """Test fallback chain is built correctly."""
        router = ModelRouter()

        # Should have models ordered by tier: large -> medium -> small -> local
        assert len(router.fallback_chain) > 0

        # Verify tiers are in order
        prev_tier_order = {"large": 0, "medium": 1, "small": 2, "local": 3}
        prev_tier = -1

        for model_name in router.fallback_chain:
            config = SUPPORTED_MODELS[model_name]
            current_tier_order = prev_tier_order[config.tier.value]
            assert current_tier_order >= prev_tier
            prev_tier = current_tier_order
