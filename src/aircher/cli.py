"""Command-line interface for Aircher."""

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


if __name__ == "__main__":
    main()
