"""Command-line interface for Aircher."""

import asyncio

import click

from . import __version__


@click.group()
@click.version_option(version=__version__)
def main() -> None:
    """Aircher: Intelligent ACP-compatible coding agent backend."""
    pass


@main.command()
def status() -> None:
    """Show Aircher system status."""
    click.echo("Aircher: Intelligent ACP-compatible coding agent")
    click.echo(f"Version: {__version__}")
    click.echo("Status: Ready for Phase 3 implementation")


@main.command()
@click.option(
    "--model",
    default="gpt-4o-mini",
    help="LLM model to use (default: gpt-4o-mini)",
)
@click.option(
    "--enable-memory/--no-memory",
    default=True,
    help="Enable/disable memory systems (default: enabled)",
)
def serve(model: str, enable_memory: bool) -> None:
    """Run Aircher as an ACP server (JSON-RPC over stdio).

    This mode allows Aircher to be used by ACP-compatible editors like Zed.

    Example:
        aircher serve --model gpt-4o
    """
    click.echo(f"Starting Aircher ACP server with {model}...")
    click.echo(f"Memory systems: {'enabled' if enable_memory else 'disabled'}")

    from .agent import AircherAgent
    from .protocol.server import ACPServer

    # Create agent
    agent = AircherAgent(model_name=model, enable_memory=enable_memory)

    # Create and run ACP server
    server = ACPServer(agent=agent)

    try:
        asyncio.run(server.start())
    except KeyboardInterrupt:
        click.echo("\nShutting down ACP server...")
        server.stop()


if __name__ == "__main__":
    main()
