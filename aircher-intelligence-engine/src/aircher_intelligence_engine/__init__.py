"""
Aircher Intelligence Engine - Universal MCP Server

A sophisticated Model Context Protocol server providing intelligent context management,
cross-project learning, and advanced AI tool integration.
"""

__version__ = "0.1.0"

async def main():
    """Main entry point for the MCP server."""
    from .mcp.server import AircherIntelligenceServer
    
    server = AircherIntelligenceServer()
    await server.run()

def sync_main() -> None:
    """Synchronous wrapper for main entry point."""
    import asyncio
    asyncio.run(main())
