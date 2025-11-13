"""Tool management for Aircher."""

import json
import os
import platform
import shutil
import tarfile
from pathlib import Path

import httpx
from loguru import logger


class ToolManager:
    """Manage bundled and external tools for Aircher."""

    def __init__(self):
        self.aircher_dir = Path.home() / ".aircher"
        self.tools_dir = self.aircher_dir / "tools" / "bin"
        self.tools_dir.mkdir(parents=True, exist_ok=True)

        self.versions_file = self.aircher_dir / "tools" / "versions.json"
        self.versions = self._load_versions()

        # Add to PATH for this session
        os.environ["PATH"] = f"{self.tools_dir}:{os.environ['PATH']}"

    def _load_versions(self) -> dict[str, str]:
        """Load tool versions from file."""
        if self.versions_file.exists():
            return json.loads(self.versions_file.read_text())
        return {}

    def _save_versions(self):
        """Save tool versions to file."""
        self.versions_file.parent.mkdir(parents=True, exist_ok=True)
        self.versions_file.write_text(json.dumps(self.versions, indent=2))

    def get_platform(self) -> str:
        """Get platform identifier for downloads."""
        system = platform.system().lower()
        machine = platform.machine().lower()

        if system == "darwin":
            return "darwin-arm64" if machine == "arm64" else "darwin-x86_64"
        elif system == "linux":
            return "linux-x86_64" if machine in ["x86_64", "amd64"] else "linux-arm64"
        else:
            raise ValueError(f"Unsupported platform: {system}-{machine}")

    def get_tool(self, preferred: str, fallback: str | None = None) -> str:
        """Get best available tool for purpose."""
        # Check if tool is available in PATH or bundled
        if shutil.which(preferred):
            return preferred

        # Check if we have it bundled
        bundled_tool = self.tools_dir / preferred
        if bundled_tool.exists():
            return str(bundled_tool)

        # Use fallback if provided
        if fallback and shutil.which(fallback):
            return fallback

        # Raise error if no tool available
        raise RuntimeError(f"Tool {preferred} not available and no fallback provided")

    async def ensure_bundled_tools(self):
        """Download and extract bundled tools."""
        platform = self.get_platform()

        # Tools to bundle with versions
        tools = {
            "rg": {
                "version": "14.1.0",
                "url_template": "https://github.com/BurntSushi/ripgrep/releases/download/{version}/ripgrep_{version}_{platform}.tar.gz",
            },
            "ast-grep": {
                "version": "0.25.0",
                "url_template": "https://github.com/ast-grep/ast-grep/releases/download/{version}/ast-grep-{platform}.tar.gz",
            },
        }

        platform_map = {
            "darwin-arm64": "aarch64-apple-darwin",
            "darwin-x86_64": "x86_64-apple-darwin",
            "linux-x86_64": "x86_64-unknown-linux-musl",
            "linux-arm64": "aarch64-unknown-linux-musl",
        }

        for tool_name, config in tools.items():
            current_version = self.versions.get(tool_name)
            target_version = config["version"]

            # Skip if we have the latest version
            if (
                current_version == target_version
                and (self.tools_dir / tool_name).exists()
            ):
                logger.info(f"{tool_name} {target_version} already installed")
                continue

            # Download and extract
            platform_name = platform_map[platform]
            url = config["url_template"].format(
                version=target_version, platform=platform_name
            )

            logger.info(f"Downloading {tool_name} {target_version}...")
            await self._download_and_extract(url, tool_name)

            # Update version
            self.versions[tool_name] = target_version
            self._save_versions()

            logger.info(f"Installed {tool_name} {target_version}")

    async def _download_and_extract(self, url: str, tool_name: str):
        """Download and extract a tool from URL."""
        async with httpx.AsyncClient() as client:
            response = await client.get(url)
            response.raise_for_status()

            # Download to temporary file
            temp_file = self.tools_dir / f"{tool_name}.tar.gz"
            temp_file.write_bytes(response.content)

            # Extract tar.gz
            with tarfile.open(temp_file, "r:gz") as tar:
                for member in tar:
                    if member.isreg():
                        # Find the actual binary
                        if member.name.endswith(tool_name) or (
                            tool_name == "rg" and member.name.endswith("rg")
                        ):
                            content = tar.extractfile(member)
                            if content:
                                tool_path = self.tools_dir / tool_name
                                tool_path.write_bytes(content.read())
                                tool_path.chmod(0o755)  # Make executable
                                break

            # Clean up temp file
            temp_file.unlink()

    def get_tool_status(self) -> dict[str, str]:
        """Get status of all tools."""
        status = {}

        # Check bundled tools
        bundled_tools = ["rg", "ast-grep"]
        for tool in bundled_tools:
            tool_path = self.tools_dir / tool
            if tool_path.exists():
                version = self.versions.get(tool, "unknown")
                status[tool] = f"bundled ({version})"
            else:
                status[tool] = "not installed"

        # Check external tools
        external_tools = ["fd", "jq", "sd", "git"]
        for tool in external_tools:
            if shutil.which(tool):
                status[tool] = "system"
            else:
                status[tool] = "not found"

        return status


# Global instance
_tool_manager = None


def get_tool_manager() -> ToolManager:
    """Get global tool manager instance."""
    global _tool_manager
    if _tool_manager is None:
        _tool_manager = ToolManager()
    return _tool_manager


def get_tool(preferred: str, fallback: str | None = None) -> str:
    """Get best available tool (convenience function)."""
    return get_tool_manager().get_tool(preferred, fallback)
