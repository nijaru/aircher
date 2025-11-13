"""Agent mode definitions and management."""

from enum import Enum

from pydantic import BaseModel


class AgentMode(str, Enum):
    """Agent operation modes."""

    READ = "read"
    EDIT = "edit"
    TURBO = "turbo"


class ModeCapabilities(BaseModel):
    """Capabilities for each agent mode."""

    can_read_files: bool = True
    can_write_files: bool = False
    can_execute_commands: bool = False
    can_spawn_subagents: bool = False
    requires_confirmation: bool = False


# Mode capability definitions
MODE_CAPABILITIES = {
    AgentMode.READ: ModeCapabilities(
        can_read_files=True,
        can_write_files=False,
        can_execute_commands=False,
        can_spawn_subagents=True,
        requires_confirmation=False,
    ),
    AgentMode.EDIT: ModeCapabilities(
        can_read_files=True,
        can_write_files=True,
        can_execute_commands=False,
        can_spawn_subagents=False,
        requires_confirmation=True,
    ),
    AgentMode.TURBO: ModeCapabilities(
        can_read_files=True,
        can_write_files=True,
        can_execute_commands=True,
        can_spawn_subagents=True,
        requires_confirmation=False,
    ),
}


def get_mode_capabilities(mode: AgentMode) -> ModeCapabilities:
    """Get capabilities for a specific mode."""
    return MODE_CAPABILITIES[mode]


def list_available_modes() -> list[str]:
    """List all available agent modes."""
    return [mode.value for mode in AgentMode]
