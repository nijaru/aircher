"""Tests for ACP protocol."""

import json
from datetime import datetime

import pytest

from aircher.protocol import (
    ACPError,
    ACPMessage,
    ACPNotification,
    ACPProtocol,
    ACPRequest,
    ACPResponse,
    ACPSession,
    AgentMode,
    MessageType,
    ToolCall,
    ToolCallStatus,
)


class TestACPMessage:
    """Test ACP message classes."""

    def test_acp_request_creation(self):
        """Test creating an ACP request."""
        request = ACPRequest(
            id="req-123",
            type=MessageType.REQUEST,
            timestamp=datetime.now(),
            session_id="session-123",
            method="agent.prompt",
            params={"message": "Hello"},
            mode=AgentMode.READ,
        )

        assert request.id == "req-123"
        assert request.method == "agent.prompt"
        assert request.params["message"] == "Hello"
        assert request.mode == AgentMode.READ

    def test_acp_response_creation(self):
        """Test creating an ACP response."""
        response = ACPResponse(
            id="resp-123",
            type=MessageType.RESPONSE,
            timestamp=datetime.now(),
            session_id="session-123",
            request_id="req-123",
            result={"response": "Hi there!"},
        )

        assert response.request_id == "req-123"
        assert response.result["response"] == "Hi there!"
        assert response.error is None

    def test_acp_error_creation(self):
        """Test creating an ACP error."""
        error = ACPError(
            id="err-123",
            type=MessageType.ERROR,
            timestamp=datetime.now(),
            session_id="session-123",
            code="INVALID_PARAMS",
            message="Missing parameter",
            details={"param": "message"},
        )

        assert error.code == "INVALID_PARAMS"
        assert error.message == "Missing parameter"
        assert error.details["param"] == "message"

    def test_message_serialization(self):
        """Test message to_dict and from_dict."""
        original = ACPNotification(
            id="notif-123",
            type=MessageType.NOTIFICATION,
            timestamp=datetime.now(),
            session_id="session-123",
            event="tool.started",
            data={"tool": "read_file"},
        )

        # Serialize
        data = original.to_dict()
        assert isinstance(data["timestamp"], str)

        # Deserialize
        restored = ACPNotification.from_dict(data)
        assert restored.id == original.id
        assert restored.event == original.event
        assert isinstance(restored.timestamp, datetime)


class TestToolCall:
    """Test ToolCall class."""

    def test_tool_call_creation(self):
        """Test creating a tool call."""
        tool_call = ToolCall(
            id="tool-123",
            name="read_file",
            parameters={"path": "/path/to/file"},
            status=ToolCallStatus.PENDING,
        )

        assert tool_call.id == "tool-123"
        assert tool_call.name == "read_file"
        assert tool_call.status == ToolCallStatus.PENDING
        assert tool_call.result is None

    def test_tool_call_serialization(self):
        """Test tool call serialization."""
        original = ToolCall(
            id="tool-123",
            name="read_file",
            parameters={"path": "/path/to/file"},
            status=ToolCallStatus.COMPLETED,
            result="file contents",
            start_time=datetime.now(),
            end_time=datetime.now(),
        )

        # Serialize
        data = original.to_dict()
        assert isinstance(data["start_time"], str)
        assert isinstance(data["end_time"], str)

        # Deserialize
        restored = ToolCall.from_dict(data)
        assert restored.id == original.id
        assert restored.name == original.name
        assert isinstance(restored.start_time, datetime)


class TestACPSession:
    """Test ACP session."""

    def test_session_creation(self):
        """Test creating a session."""
        session = ACPSession(
            id="session-123",
            created_at=datetime.now(),
            last_activity=datetime.now(),
            mode=AgentMode.WRITE,
            user_id="user-456",
            metadata={"project": "test"},
        )

        assert session.id == "session-123"
        assert session.mode == AgentMode.WRITE
        assert session.user_id == "user-456"
        assert session.metadata["project"] == "test"

    def test_session_serialization(self):
        """Test session serialization."""
        original = ACPSession(
            id="session-123",
            created_at=datetime.now(),
            last_activity=datetime.now(),
            mode=AgentMode.READ,
        )

        # Serialize
        data = original.to_dict()
        assert isinstance(data["created_at"], str)

        # Deserialize
        restored = ACPSession.from_dict(data)
        assert restored.id == original.id
        assert restored.mode == original.mode
        assert isinstance(restored.created_at, datetime)


class TestACPProtocol:
    """Test ACP protocol handler."""

    def test_create_session(self):
        """Test creating a session."""
        protocol = ACPProtocol()

        session = protocol.create_session(
            mode=AgentMode.WRITE,
            user_id="user-123",
            metadata={"test": "data"},
        )

        assert session.id is not None
        assert session.mode == AgentMode.WRITE
        assert session.user_id == "user-123"
        assert session in protocol.sessions.values()

    def test_get_session(self):
        """Test getting a session."""
        protocol = ACPProtocol()

        # Create session
        created = protocol.create_session(mode=AgentMode.READ)

        # Get session
        retrieved = protocol.get_session(created.id)

        assert retrieved is not None
        assert retrieved.id == created.id
        assert retrieved.mode == created.mode

    def test_create_request(self):
        """Test creating a request."""
        protocol = ACPProtocol()

        # Create session first
        session = protocol.create_session(mode=AgentMode.READ)

        # Create request
        request = protocol.create_request(
            session_id=session.id,
            method="agent.prompt",
            params={"message": "Hello"},
        )

        assert request.method == "agent.prompt"
        assert request.session_id == session.id
        assert request.mode == AgentMode.READ  # Inherits from session

    def test_create_response(self):
        """Test creating a response."""
        protocol = ACPProtocol()

        # Create session and request
        session = protocol.create_session(mode=AgentMode.READ)
        request = protocol.create_request(
            session_id=session.id,
            method="agent.prompt",
            params={"message": "Hello"},
        )

        # Create response
        response = protocol.create_response(
            request=request,
            result={"response": "Hi there!"},
        )

        assert response.request_id == request.id
        assert response.result["response"] == "Hi there!"
        assert request.id not in protocol.pending_requests

    def test_validate_mode_permission(self):
        """Test mode permission validation."""
        protocol = ACPProtocol()

        # Admin can do anything
        admin_session = ACPSession(
            id="session-1",
            created_at=datetime.now(),
            last_activity=datetime.now(),
            mode=AgentMode.ADMIN,
        )
        assert protocol.validate_mode_permission(admin_session, AgentMode.WRITE)

        # Read can only read
        read_session = ACPSession(
            id="session-2",
            created_at=datetime.now(),
            last_activity=datetime.now(),
            mode=AgentMode.READ,
        )
        assert protocol.validate_mode_permission(read_session, AgentMode.READ)
        assert not protocol.validate_mode_permission(read_session, AgentMode.WRITE)

        # Write can read and write
        write_session = ACPSession(
            id="session-3",
            created_at=datetime.now(),
            last_activity=datetime.now(),
            mode=AgentMode.WRITE,
        )
        assert protocol.validate_mode_permission(write_session, AgentMode.READ)
        assert protocol.validate_mode_permission(write_session, AgentMode.WRITE)

    def test_serialize_deserialize_message(self):
        """Test message serialization/deserialization."""
        protocol = ACPProtocol()

        # Create a request
        request = ACPRequest(
            id="req-123",
            type=MessageType.REQUEST,
            timestamp=datetime.now(),
            session_id="session-123",
            method="agent.prompt",
            params={"message": "Hello"},
            mode=AgentMode.READ,
        )

        # Serialize
        json_str = protocol.serialize_message(request)
        data = json.loads(json_str)
        assert data["method"] == "agent.prompt"

        # Deserialize
        restored = protocol.deserialize_message(json_str)
        assert isinstance(restored, ACPRequest)
        assert restored.method == request.method
        assert restored.params == request.params
