"""ACP (Agent Communication Protocol) implementation for Aircher."""

import json
import uuid
from dataclasses import asdict, dataclass
from datetime import datetime
from enum import Enum
from typing import Any, Dict, List, Optional, Union

from pydantic import BaseModel


class MessageType(str, Enum):
    """ACP message types."""

    REQUEST = "request"
    RESPONSE = "response"
    NOTIFICATION = "notification"
    ERROR = "error"


class AgentMode(str, Enum):
    """Agent operation modes."""

    READ = "read"
    WRITE = "write"
    ADMIN = "admin"


class ToolCallStatus(str, Enum):
    """Tool call execution status."""

    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


@dataclass
class ACPMessage:
    """Base ACP message."""

    id: str
    type: MessageType
    timestamp: datetime
    session_id: str

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        data = asdict(self)
        data["timestamp"] = self.timestamp.isoformat()
        return data

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "ACPMessage":
        """Create from dictionary."""
        data["timestamp"] = datetime.fromisoformat(data["timestamp"])
        return cls(**data)


@dataclass
class ACPRequest(ACPMessage):
    """ACP request message."""

    method: str
    params: dict[str, Any]
    mode: AgentMode = AgentMode.READ

    def __post_init__(self):
        if self.type != MessageType.REQUEST:
            raise ValueError("ACPRequest must have type 'request'")


@dataclass
class ACPResponse(ACPMessage):
    """ACP response message."""

    request_id: str
    result: Any | None = None
    error: dict[str, Any] | None = None

    def __post_init__(self):
        if self.type != MessageType.RESPONSE:
            raise ValueError("ACPResponse must have type 'response'")


@dataclass
class ACPNotification(ACPMessage):
    """ACP notification message."""

    event: str
    data: dict[str, Any]

    def __post_init__(self):
        if self.type != MessageType.NOTIFICATION:
            raise ValueError("ACPNotification must have type 'notification'")


@dataclass
class ACPError(ACPMessage):
    """ACP error message."""

    code: str
    message: str
    details: dict[str, Any] | None = None

    def __post_init__(self):
        if self.type != MessageType.ERROR:
            raise ValueError("ACPError must have type 'error'")


@dataclass
class ToolCall:
    """Tool call definition."""

    id: str
    name: str
    parameters: dict[str, Any]
    status: ToolCallStatus = ToolCallStatus.PENDING
    result: Any | None = None
    error: str | None = None
    start_time: datetime | None = None
    end_time: datetime | None = None

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        data = asdict(self)
        if self.start_time:
            data["start_time"] = self.start_time.isoformat()
        if self.end_time:
            data["end_time"] = self.end_time.isoformat()
        return data

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "ToolCall":
        """Create from dictionary."""
        if data.get("start_time"):
            data["start_time"] = datetime.fromisoformat(data["start_time"])
        if data.get("end_time"):
            data["end_time"] = datetime.fromisoformat(data["end_time"])
        return cls(**data)


@dataclass
class ACPSession:
    """ACP session information."""

    id: str
    created_at: datetime
    last_activity: datetime
    mode: AgentMode
    user_id: str | None = None
    metadata: dict[str, Any] | None = None

    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        data = asdict(self)
        data["created_at"] = self.created_at.isoformat()
        data["last_activity"] = self.last_activity.isoformat()
        return data

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "ACPSession":
        """Create from dictionary."""
        data["created_at"] = datetime.fromisoformat(data["created_at"])
        data["last_activity"] = datetime.fromisoformat(data["last_activity"])
        return cls(**data)


class ACPProtocol:
    """ACP protocol handler."""

    def __init__(self):
        self.sessions: dict[str, ACPSession] = {}
        self.pending_requests: dict[str, ACPRequest] = {}

    def create_session(
        self,
        mode: AgentMode = AgentMode.READ,
        user_id: str | None = None,
        metadata: dict[str, Any] | None = None,
    ) -> ACPSession:
        """Create a new ACP session."""
        session_id = str(uuid.uuid4())
        now = datetime.now()

        session = ACPSession(
            id=session_id,
            created_at=now,
            last_activity=now,
            mode=mode,
            user_id=user_id,
            metadata=metadata or {},
        )

        self.sessions[session_id] = session
        return session

    def get_session(self, session_id: str) -> ACPSession | None:
        """Get session by ID."""
        session = self.sessions.get(session_id)
        if session:
            session.last_activity = datetime.now()
        return session

    def create_request(
        self,
        session_id: str,
        method: str,
        params: dict[str, Any],
        mode: AgentMode | None = None,
    ) -> ACPRequest:
        """Create an ACP request."""
        session = self.get_session(session_id)
        if not session:
            raise ValueError(f"Session {session_id} not found")

        request_id = str(uuid.uuid4())
        request = ACPRequest(
            id=request_id,
            type=MessageType.REQUEST,
            timestamp=datetime.now(),
            session_id=session_id,
            method=method,
            params=params,
            mode=mode or session.mode,
        )

        self.pending_requests[request_id] = request
        return request

    def create_response(
        self,
        request: ACPRequest,
        result: Any | None = None,
        error: dict[str, Any] | None = None,
    ) -> ACPResponse:
        """Create an ACP response."""
        response = ACPResponse(
            id=str(uuid.uuid4()),
            type=MessageType.RESPONSE,
            timestamp=datetime.now(),
            session_id=request.session_id,
            request_id=request.id,
            result=result,
            error=error,
        )

        # Remove from pending requests
        self.pending_requests.pop(request.id, None)
        return response

    def create_notification(
        self, session_id: str, event: str, data: dict[str, Any]
    ) -> ACPNotification:
        """Create an ACP notification."""
        return ACPNotification(
            id=str(uuid.uuid4()),
            type=MessageType.NOTIFICATION,
            timestamp=datetime.now(),
            session_id=session_id,
            event=event,
            data=data,
        )

    def create_error(
        self,
        session_id: str,
        code: str,
        message: str,
        details: dict[str, Any] | None = None,
    ) -> ACPError:
        """Create an ACP error."""
        return ACPError(
            id=str(uuid.uuid4()),
            type=MessageType.ERROR,
            timestamp=datetime.now(),
            session_id=session_id,
            code=code,
            message=message,
            details=details,
        )

    def serialize_message(self, message: ACPMessage) -> str:
        """Serialize message to JSON string."""
        return json.dumps(message.to_dict())

    def deserialize_message(self, data: str) -> ACPMessage:
        """Deserialize message from JSON string."""
        message_data = json.loads(data)
        message_type = MessageType(message_data["type"])

        if message_type == MessageType.REQUEST:
            return ACPRequest.from_dict(message_data)
        elif message_type == MessageType.RESPONSE:
            return ACPResponse.from_dict(message_data)
        elif message_type == MessageType.NOTIFICATION:
            return ACPNotification.from_dict(message_data)
        elif message_type == MessageType.ERROR:
            return ACPError.from_dict(message_data)
        else:
            raise ValueError(f"Unknown message type: {message_type}")

    def validate_mode_permission(
        self, session: ACPSession, requested_mode: AgentMode
    ) -> bool:
        """Validate if requested mode is allowed for session."""
        # Admin can do anything
        if session.mode == AgentMode.ADMIN:
            return True

        # Read mode can only read
        if session.mode == AgentMode.READ:
            return requested_mode == AgentMode.READ

        # Write mode can read or write
        if session.mode == AgentMode.WRITE:
            return requested_mode in [AgentMode.READ, AgentMode.WRITE]

        return False
