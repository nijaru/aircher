"""Test basic configuration and imports."""

import pytest

from aircher.agent import AircherAgent
from aircher.config import get_settings
from aircher.modes import AgentMode, get_mode_capabilities


def test_settings():
    """Test settings loading."""
    settings = get_settings()
    assert settings.agent_mode == "read"
    assert settings.default_model == "gpt-4o-mini"


def test_agent_modes():
    """Test agent mode functionality."""
    # Test mode capabilities
    read_caps = get_mode_capabilities(AgentMode.READ)
    assert read_caps.can_read_files is True
    assert read_caps.can_write_files is False

    edit_caps = get_mode_capabilities(AgentMode.EDIT)
    assert edit_caps.can_read_files is True
    assert edit_caps.can_write_files is True
    assert edit_caps.requires_confirmation is True

    turbo_caps = get_mode_capabilities(AgentMode.TURBO)
    assert turbo_caps.can_execute_commands is True
    assert turbo_caps.requires_confirmation is False


@pytest.mark.asyncio
async def test_agent_creation():
    """Test agent can be created."""
    agent = AircherAgent()
    assert agent is not None
    assert agent.graph is not None
