"""ACP server implementation."""

import asyncio
from typing import Any

from loguru import logger

from ..agent import AircherAgent
from ..modes import AgentMode
from . import ACPProtocol, ACPSession, AgentMode as ACPAgentMode
from .transport import ErrorCode, JSONRPCError, StdioTransport


class ACPServer:
    """ACP protocol server for Aircher agent."""

    def __init__(self, agent: AircherAgent | None = None):
        self.protocol = ACPProtocol()
        self.transport = StdioTransport()
        self.agent = agent or AircherAgent()
        self.request_handlers = self._build_handlers()

    def _build_handlers(self) -> dict[str, Any]:
        """Build JSON-RPC method handlers."""
        return {
            "initialize": self._handle_initialize,
            "session.create": self._handle_session_create,
            "session.get": self._handle_session_get,
            "session.end": self._handle_session_end,
            "agent.prompt": self._handle_agent_prompt,
            "agent.cancel": self._handle_agent_cancel,
            "tool.execute": self._handle_tool_execute,
        }

    async def _handle_message(self, message: dict[str, Any]) -> dict[str, Any] | None:
        """Handle incoming JSON-RPC message."""
        # Validate JSON-RPC 2.0 message
        if message.get("jsonrpc") != "2.0":
            raise JSONRPCError(
                ErrorCode.INVALID_REQUEST,
                "Invalid JSON-RPC version (must be 2.0)",
            )

        # Check if it's a request (has method) or response
        method = message.get("method")
        if not method:
            # It's a response or notification, log it
            logger.debug(f"Received response/notification: {message}")
            return None

        request_id = message.get("id")
        params = message.get("params", {})

        try:
            # Find handler for method
            handler = self.request_handlers.get(method)
            if not handler:
                raise JSONRPCError(
                    ErrorCode.METHOD_NOT_FOUND,
                    f"Method not found: {method}",
                )

            # Execute handler
            result = await handler(params)

            # Return response (only if request has id)
            if request_id is not None:
                return {
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "result": result,
                }
            else:
                # Notification, no response needed
                return None

        except JSONRPCError as e:
            # JSON-RPC error
            if request_id is not None:
                return {
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "error": e.to_dict(),
                }
            else:
                logger.error(f"Error in notification: {e}")
                return None

        except Exception as e:
            # Internal error
            logger.error(f"Internal error handling {method}: {e}")
            if request_id is not None:
                return {
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "error": {
                        "code": ErrorCode.INTERNAL_ERROR,
                        "message": "Internal error",
                        "data": str(e),
                    },
                }
            else:
                return None

    async def _handle_initialize(self, params: dict[str, Any]) -> dict[str, Any]:
        """Handle initialize request."""
        logger.info("Initializing ACP server")

        return {
            "protocolVersion": "0.1.0",
            "serverInfo": {
                "name": "Aircher",
                "version": "0.1.0",
            },
            "capabilities": {
                "sessions": True,
                "streaming": False,  # Not implemented yet
                "tools": {
                    "read_file": True,
                    "write_file": True,
                    "list_directory": True,
                    "search_files": True,
                    "bash": True,
                },
            },
        }

    async def _handle_session_create(self, params: dict[str, Any]) -> dict[str, Any]:
        """Handle session creation request."""
        mode_str = params.get("mode", "read")

        # Map mode string to AgentMode
        mode_map = {
            "read": ACPAgentMode.READ,
            "write": ACPAgentMode.WRITE,
            "admin": ACPAgentMode.ADMIN,
        }
        mode = mode_map.get(mode_str.lower(), ACPAgentMode.READ)

        user_id = params.get("user_id")
        metadata = params.get("metadata", {})

        session = self.protocol.create_session(
            mode=mode,
            user_id=user_id,
            metadata=metadata,
        )

        logger.info(f"Created session {session.id} in {mode.value} mode")

        return session.to_dict()

    async def _handle_session_get(self, params: dict[str, Any]) -> dict[str, Any]:
        """Handle get session request."""
        session_id = params.get("session_id")
        if not session_id:
            raise JSONRPCError(
                ErrorCode.INVALID_PARAMS,
                "Missing required parameter: session_id",
            )

        session = self.protocol.get_session(session_id)
        if not session:
            raise JSONRPCError(
                ErrorCode.SESSION_NOT_FOUND,
                f"Session not found: {session_id}",
            )

        return session.to_dict()

    async def _handle_session_end(self, params: dict[str, Any]) -> dict[str, Any]:
        """Handle session end request."""
        session_id = params.get("session_id")
        if not session_id:
            raise JSONRPCError(
                ErrorCode.INVALID_PARAMS,
                "Missing required parameter: session_id",
            )

        session = self.protocol.sessions.pop(session_id, None)
        if not session:
            raise JSONRPCError(
                ErrorCode.SESSION_NOT_FOUND,
                f"Session not found: {session_id}",
            )

        logger.info(f"Ended session {session_id}")

        return {"status": "ended", "session_id": session_id}

    async def _handle_agent_prompt(self, params: dict[str, Any]) -> dict[str, Any]:
        """Handle agent prompt request."""
        session_id = params.get("session_id")
        message = params.get("message")

        if not session_id or not message:
            raise JSONRPCError(
                ErrorCode.INVALID_PARAMS,
                "Missing required parameters: session_id, message",
            )

        # Get session
        session = self.protocol.get_session(session_id)
        if not session:
            raise JSONRPCError(
                ErrorCode.SESSION_NOT_FOUND,
                f"Session not found: {session_id}",
            )

        # Map ACP mode to Agent mode
        mode_map = {
            ACPAgentMode.READ: AgentMode.READ,
            ACPAgentMode.WRITE: AgentMode.WRITE,
            ACPAgentMode.ADMIN: AgentMode.WRITE,  # Map ADMIN to WRITE for now
        }
        agent_mode = mode_map.get(session.mode, AgentMode.READ)

        logger.info(f"Processing prompt for session {session_id}")

        try:
            # Run agent
            result = await self.agent.run(
                message=message,
                mode=agent_mode,
                session_id=session_id,
            )

            return {
                "session_id": session_id,
                "response": result.get("response", ""),
                "tool_calls": result.get("tool_calls", []),
                "cost_summary": result.get("cost_summary", {}),
            }

        except Exception as e:
            logger.error(f"Agent execution failed: {e}")
            raise JSONRPCError(
                ErrorCode.INTERNAL_ERROR,
                f"Agent execution failed: {str(e)}",
            )

    async def _handle_agent_cancel(self, params: dict[str, Any]) -> dict[str, Any]:
        """Handle agent cancel request."""
        session_id = params.get("session_id")

        if not session_id:
            raise JSONRPCError(
                ErrorCode.INVALID_PARAMS,
                "Missing required parameter: session_id",
            )

        # TODO: Implement actual cancellation
        logger.warning(f"Cancel requested for session {session_id} (not implemented)")

        return {"status": "cancelled", "session_id": session_id}

    async def _handle_tool_execute(self, params: dict[str, Any]) -> dict[str, Any]:
        """Handle tool execution request."""
        tool_name = params.get("tool")
        tool_params = params.get("parameters", {})

        if not tool_name:
            raise JSONRPCError(
                ErrorCode.INVALID_PARAMS,
                "Missing required parameter: tool",
            )

        logger.info(f"Executing tool: {tool_name}")

        try:
            # Find tool in agent's tool list
            tool = None
            for t in self.agent.tools:
                if t.name == tool_name:
                    tool = t
                    break

            if not tool:
                raise JSONRPCError(
                    ErrorCode.METHOD_NOT_FOUND,
                    f"Tool not found: {tool_name}",
                )

            # Execute tool
            result = await tool.execute(tool_params)

            return {
                "tool": tool_name,
                "status": "success",
                "result": result,
            }

        except Exception as e:
            logger.error(f"Tool execution failed: {e}")
            raise JSONRPCError(
                ErrorCode.TOOL_EXECUTION_FAILED,
                f"Tool execution failed: {str(e)}",
            )

    async def start(self):
        """Start the ACP server."""
        logger.info("Starting ACP server...")

        # Set up message handler
        self.transport.set_message_handler(self._handle_message)

        # Start transport
        await self.transport.start()

    def stop(self):
        """Stop the ACP server."""
        logger.info("Stopping ACP server...")
        self.transport.stop()


async def run_acp_server():
    """Run ACP server in stdio mode."""
    server = ACPServer()
    try:
        await server.start()
    except KeyboardInterrupt:
        logger.info("Interrupted by user")
    finally:
        server.stop()


if __name__ == "__main__":
    asyncio.run(run_acp_server())
