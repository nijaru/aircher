"""Stdio transport for ACP protocol (JSON-RPC over stdin/stdout)."""

import asyncio
import json
import sys
from typing import Any, Callable

from loguru import logger


class StdioTransport:
    """JSON-RPC transport over stdin/stdout for ACP protocol."""

    def __init__(self):
        self.running = False
        self.message_handler: Callable[[dict[str, Any]], Any] | None = None

    def set_message_handler(self, handler: Callable[[dict[str, Any]], Any]):
        """Set the message handler callback."""
        self.message_handler = handler

    async def start(self):
        """Start the stdio transport loop."""
        self.running = True
        logger.info("Stdio transport started, listening on stdin...")

        try:
            while self.running:
                # Read from stdin
                line = await asyncio.get_event_loop().run_in_executor(
                    None, sys.stdin.readline
                )

                if not line:
                    # EOF reached
                    logger.info("Stdin EOF reached, stopping transport")
                    break

                line = line.strip()
                if not line:
                    continue

                try:
                    # Parse JSON-RPC message
                    message = json.loads(line)
                    logger.debug(f"Received message: {message.get('method', message.get('id', 'notification'))}")

                    # Handle message
                    if self.message_handler:
                        response = await self.message_handler(message)
                        if response:
                            await self.send_message(response)
                    else:
                        logger.warning("No message handler registered")

                except json.JSONDecodeError as e:
                    logger.error(f"Failed to parse JSON: {e}")
                    error_response = {
                        "jsonrpc": "2.0",
                        "id": None,
                        "error": {
                            "code": -32700,
                            "message": "Parse error",
                            "data": str(e),
                        },
                    }
                    await self.send_message(error_response)

                except Exception as e:
                    logger.error(f"Error handling message: {e}")
                    error_response = {
                        "jsonrpc": "2.0",
                        "id": None,
                        "error": {
                            "code": -32603,
                            "message": "Internal error",
                            "data": str(e),
                        },
                    }
                    await self.send_message(error_response)

        except Exception as e:
            logger.error(f"Transport error: {e}")
        finally:
            self.running = False
            logger.info("Stdio transport stopped")

    async def send_message(self, message: dict[str, Any]):
        """Send a JSON-RPC message to stdout."""
        try:
            json_str = json.dumps(message)
            sys.stdout.write(json_str + "\n")
            sys.stdout.flush()
            logger.debug(f"Sent message: {message.get('method', message.get('id', 'response'))}")
        except Exception as e:
            logger.error(f"Failed to send message: {e}")

    def stop(self):
        """Stop the transport."""
        self.running = False
        logger.info("Stopping stdio transport")


class JSONRPCError(Exception):
    """JSON-RPC error."""

    def __init__(self, code: int, message: str, data: Any | None = None):
        self.code = code
        self.message = message
        self.data = data
        super().__init__(f"JSON-RPC Error {code}: {message}")

    def to_dict(self) -> dict[str, Any]:
        """Convert to JSON-RPC error dict."""
        error = {
            "code": self.code,
            "message": self.message,
        }
        if self.data is not None:
            error["data"] = self.data
        return error


# JSON-RPC error codes
class ErrorCode:
    """Standard JSON-RPC error codes."""

    PARSE_ERROR = -32700
    INVALID_REQUEST = -32600
    METHOD_NOT_FOUND = -32601
    INVALID_PARAMS = -32602
    INTERNAL_ERROR = -32603

    # ACP-specific error codes
    SESSION_NOT_FOUND = -32001
    PERMISSION_DENIED = -32002
    TOOL_EXECUTION_FAILED = -32003
    AGENT_BUSY = -32004
