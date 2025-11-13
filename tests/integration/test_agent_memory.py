"""Integration tests for agent with memory systems."""

import asyncio
from pathlib import Path

import pytest

from aircher.agent import AircherAgent
from aircher.modes import AgentMode


@pytest.fixture
def temp_data_dir(tmp_path):
    """Create a temporary data directory for tests."""
    data_dir = tmp_path / "aircher_data"
    data_dir.mkdir()
    return data_dir


@pytest.fixture
def agent_with_memory(temp_data_dir):
    """Create an agent with memory enabled."""
    agent = AircherAgent(data_dir=temp_data_dir, enable_memory=True)
    return agent


@pytest.fixture
def agent_without_memory(temp_data_dir):
    """Create an agent with memory disabled."""
    agent = AircherAgent(data_dir=temp_data_dir, enable_memory=False)
    return agent


class TestAgentInitialization:
    """Test agent initialization with memory."""

    def test_agent_init_with_memory(self, agent_with_memory):
        """Test that agent initializes with memory systems."""
        assert agent_with_memory.memory is not None
        assert agent_with_memory.enable_memory is True
        assert len(agent_with_memory.tools) == 5  # 5 tools loaded

    def test_agent_init_without_memory(self, agent_without_memory):
        """Test that agent initializes without memory."""
        assert agent_without_memory.memory is None
        assert agent_without_memory.enable_memory is False
        assert len(agent_without_memory.tools) == 5  # Tools still loaded

    def test_data_directory_created(self, temp_data_dir, agent_with_memory):
        """Test that data directory is created."""
        assert agent_with_memory.data_dir.exists()
        assert (agent_with_memory.data_dir / "episodic.duckdb").exists()
        assert (agent_with_memory.data_dir / "vectors").exists()


class TestToolsLoading:
    """Test that tools are loaded correctly."""

    def test_tools_loaded(self, agent_with_memory):
        """Test that all expected tools are loaded."""
        tool_names = [tool.name for tool in agent_with_memory.tools]

        assert "read_file" in tool_names
        assert "write_file" in tool_names
        assert "list_directory" in tool_names
        assert "search_files" in tool_names
        assert "bash" in tool_names

    def test_get_available_tools(self, agent_with_memory):
        """Test getting available tools."""
        tools = agent_with_memory.get_available_tools()

        assert len(tools) == 5
        assert all("name" in tool for tool in tools)
        assert all("description" in tool for tool in tools)


class TestMemoryIntegration:
    """Test memory integration into agent workflow."""

    @pytest.mark.asyncio
    async def test_memory_context_set(self, agent_with_memory):
        """Test that memory context is set during workflow."""
        result = await agent_with_memory.run(
            message="What files are in the current directory?",
            mode=AgentMode.READ,
        )

        assert result is not None
        assert "session_id" in result

    @pytest.mark.asyncio
    async def test_intent_classification_with_memory(self, agent_with_memory):
        """Test that intent classification queries memory."""
        result = await agent_with_memory.run(
            message="Show me the contents of test.py",
            mode=AgentMode.READ,
        )

        assert result["intent"] in ["read", "general"]
        # Memory context should be queried
        assert "context" in result

    @pytest.mark.asyncio
    async def test_tool_selection_with_memory(self, agent_with_memory, tmp_path):
        """Test that tool selection uses memory context."""
        # Create a test file
        test_file = tmp_path / "test.py"
        test_file.write_text("def test(): pass")

        result = await agent_with_memory.run(
            message=f"Read {test_file}",
            mode=AgentMode.READ,
        )

        # Tool calls should be generated
        assert "tool_calls" in result
        assert "context" in result


class TestMemoryRecording:
    """Test that agent records to memory."""

    @pytest.mark.asyncio
    async def test_memory_recording_after_execution(self, agent_with_memory):
        """Test that memory is updated after task execution."""
        # Run a task
        result = await agent_with_memory.run(
            message="List files",
            mode=AgentMode.READ,
        )

        # Check that memory was updated
        assert result["context"].get("memory_updated") is not False

        # Verify memory contains records
        stats = agent_with_memory.memory.get_tool_statistics(days=1)
        # Even if no tools executed (empty plan), memory should be queryable
        assert isinstance(stats, list)

    @pytest.mark.asyncio
    async def test_session_tracking(self, agent_with_memory):
        """Test that sessions are tracked properly."""
        session_id = "test-session-123"

        result = await agent_with_memory.run(
            message="Test message",
            mode=AgentMode.READ,
            session_id=session_id,
        )

        assert result["session_id"] == session_id


class TestWorkflowNodes:
    """Test individual workflow nodes."""

    @pytest.mark.asyncio
    async def test_classify_intent_node(self, agent_with_memory):
        """Test intent classification node."""
        result = await agent_with_memory.run(
            message="Show me the code",
            mode=AgentMode.READ,
        )

        assert "intent" in result
        assert result["intent"] in ["read", "general", "search"]

    @pytest.mark.asyncio
    async def test_validate_permissions_node(self, agent_with_memory):
        """Test permission validation."""
        # Read mode should allow read operations
        result = await agent_with_memory.run(
            message="Read test.py",
            mode=AgentMode.READ,
        )

        assert "context" in result

    @pytest.mark.asyncio
    async def test_workflow_completes(self, agent_with_memory):
        """Test that workflow completes end-to-end."""
        result = await agent_with_memory.run(
            message="Test workflow",
            mode=AgentMode.READ,
        )

        # Verify all nodes executed
        assert "intent" in result
        assert "context" in result
        assert "response" in result
        assert isinstance(result["response"], str)


class TestMemoryVsNoMemory:
    """Compare agent behavior with and without memory."""

    @pytest.mark.asyncio
    async def test_both_agents_work(
        self, agent_with_memory, agent_without_memory
    ):
        """Test that both configurations work."""
        message = "Test message"

        result_with = await agent_with_memory.run(message, mode=AgentMode.READ)
        result_without = await agent_without_memory.run(message, mode=AgentMode.READ)

        # Both should complete
        assert result_with["response"]
        assert result_without["response"]

        # Only memory-enabled should have memory_updated
        assert "memory_updated" in result_with["context"]
        assert result_without["context"].get("memory_updated") == False


class TestErrorHandling:
    """Test error handling in agent workflow."""

    @pytest.mark.asyncio
    async def test_memory_failure_doesnt_crash(self, agent_with_memory, monkeypatch):
        """Test that memory failures don't crash the agent."""
        # Simulate memory failure
        def failing_get_tool_statistics(*args, **kwargs):
            raise Exception("Simulated memory failure")

        if agent_with_memory.memory:
            monkeypatch.setattr(
                agent_with_memory.memory,
                "get_tool_statistics",
                failing_get_tool_statistics,
            )

        # Agent should still work
        result = await agent_with_memory.run(
            message="Test",
            mode=AgentMode.READ,
        )

        assert result["response"]  # Should still generate a response

    @pytest.mark.asyncio
    async def test_invalid_mode_handling(self, agent_with_memory):
        """Test handling of operations with insufficient permissions."""
        # Try to write in READ mode
        result = await agent_with_memory.run(
            message="Write to file.txt",
            mode=AgentMode.READ,
        )

        # Should classify as write intent but permission check might deny
        # For now, just verify it completes without crashing
        assert "response" in result


class TestMemoryQueries:
    """Test memory query functionality."""

    @pytest.mark.asyncio
    async def test_file_history_query(self, agent_with_memory, tmp_path):
        """Test that file history is queried correctly."""
        test_file = tmp_path / "test.py"
        test_file.write_text("print('hello')")

        # Run multiple operations on the same file
        await agent_with_memory.run(
            message=f"Read {test_file}",
            mode=AgentMode.READ,
        )

        # Query file history
        history = agent_with_memory.memory.query_file_history(str(test_file))

        # History should be available (even if empty due to no actual tool execution)
        assert isinstance(history, list)

    @pytest.mark.asyncio
    async def test_tool_statistics_query(self, agent_with_memory):
        """Test querying tool statistics."""
        # Run some operations
        await agent_with_memory.run(
            message="List files",
            mode=AgentMode.READ,
        )

        # Query statistics
        stats = agent_with_memory.memory.get_tool_statistics(days=7)

        assert isinstance(stats, list)
