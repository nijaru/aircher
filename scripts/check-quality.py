#!/usr/bin/env python3
"""Run all code quality checks."""

import subprocess
import sys
from pathlib import Path


def run_command(cmd: list[str], description: str) -> bool:
    """Run a command and return success status."""
    print(f"Running {description}...")
    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print(f"✅ {description} passed")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ {description} failed")
        print(e.stdout)
        print(e.stderr)
        return False


def main() -> None:
    """Run all quality checks."""
    project_root = Path(__file__).parent.parent

    checks = [
        (["uv", "run", "ruff", "check", "."], "Ruff linting"),
        (["uv", "run", "ruff", "format", "--check", "."], "Ruff formatting"),
        (["uv", "run", "ty", "src/", "--strict"], "Type checking"),
        (["uv", "run", "vulture", "src/"], "Dead code detection"),
        (["uv", "run", "pytest", "tests/", "-q"], "Tests"),
    ]

    failed = []
    for cmd, desc in checks:
        if not run_command(cmd, desc):
            failed.append(desc)

    if failed:
        print(f"\n❌ Failed checks: {', '.join(failed)}")
        sys.exit(1)
    else:
        print("\n✅ All checks passed!")
        sys.exit(0)


if __name__ == "__main__":
    main()
