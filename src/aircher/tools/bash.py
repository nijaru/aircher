"""Bash tool execution for Aircher."""

import os
import shlex
import subprocess
from dataclasses import dataclass
from pathlib import Path

from loguru import logger


@dataclass
class BashResult:
    """Result of bash command execution."""

    stdout: str
    stderr: str
    returncode: int
    success: bool
    command: str


class BashTool:
    """Safe bash tool execution with fallbacks."""

    def __init__(self, tool_manager):
        self.tool_manager = tool_manager
        self.timeout = 30  # Default timeout in seconds

    def execute(
        self,
        command: str,
        timeout: int | None = None,
        cwd: str | Path | None = None,
        env: dict[str, str] | None = None,
        capture_output: bool = True,
    ) -> BashResult:
        """Execute bash command safely."""
        if timeout is None:
            timeout = self.timeout

        # Prepare environment
        exec_env = os.environ.copy()
        if env:
            exec_env.update(env)

        # Ensure our bundled tools are in PATH
        exec_env["PATH"] = f"{self.tool_manager.tools_dir}:{exec_env['PATH']}"

        # Prepare working directory
        work_dir = Path(cwd) if cwd else Path.cwd()

        try:
            logger.debug(f"Executing: {command} in {work_dir}")

            # Execute command
            result = subprocess.run(
                command,
                shell=True,
                timeout=timeout,
                cwd=work_dir,
                env=exec_env,
                capture_output=capture_output,
                text=True,
            )

            return BashResult(
                stdout=result.stdout or "",
                stderr=result.stderr or "",
                returncode=result.returncode,
                success=result.returncode == 0,
                command=command,
            )

        except subprocess.TimeoutExpired:
            return BashResult(
                stdout="",
                stderr=f"Command timed out after {timeout} seconds",
                returncode=124,
                success=False,
                command=command,
            )
        except Exception as e:
            return BashResult(
                stdout="",
                stderr=f"Command execution failed: {str(e)}",
                returncode=1,
                success=False,
                command=command,
            )

    def get_tool_command(
        self,
        preferred: str,
        fallback: str | None = None,
        args: list[str] | None = None,
    ) -> str:
        """Get command with available tool and arguments."""
        tool_path = self.tool_manager.get_tool(preferred, fallback)

        if args:
            return f"{tool_path} {' '.join(shlex.quote(arg) for arg in args)}"
        return tool_path

    # Common tool commands
    def ripgrep(
        self, pattern: str, path: str = ".", args: list[str] | None = None
    ) -> BashResult:
        """Execute ripgrep search."""
        rg_args = [pattern, path]
        if args:
            rg_args.extend(args)

        command = self.get_tool_command("rg", "grep", rg_args)
        return self.execute(command)

    def ast_grep(
        self, pattern: str, path: str = ".", args: list[str] | None = None
    ) -> BashResult:
        """Execute ast-grep search."""
        sg_args = ["run", pattern, path]
        if args:
            sg_args.extend(args)

        command = self.get_tool_command("ast-grep", fallback=None, args=sg_args)
        return self.execute(command)

    def find_files(
        self, pattern: str, path: str = ".", args: list[str] | None = None
    ) -> BashResult:
        """Find files using fd or find."""
        fd_args = [pattern, path]
        if args:
            fd_args.extend(args)

        command = self.get_tool_command("fd", "find", fd_args)
        return self.execute(command)

    def list_directory(
        self, path: str = ".", args: list[str] | None = None
    ) -> BashResult:
        """List directory contents."""
        ls_args = [path]
        if args:
            ls_args.extend(args)

        command = self.get_tool_command("ls", fallback=None, args=ls_args)
        return self.execute(command)

    def read_file(self, path: str, args: list[str] | None = None) -> BashResult:
        """Read file contents."""
        cat_args = [path]
        if args:
            cat_args.extend(args)

        command = self.get_tool_command("cat", fallback=None, args=cat_args)
        return self.execute(command)

    def git_command(self, subcommand: str, args: list[str] | None = None) -> BashResult:
        """Execute git command."""
        git_args = [subcommand]
        if args:
            git_args.extend(args)

        command = self.get_tool_command("git", fallback=None, args=git_args)
        return self.execute(command)

    def jq_filter(
        self,
        filter_expr: str,
        input_file: str | None = None,
        args: list[str] | None = None,
    ) -> BashResult:
        """Execute jq JSON filter."""
        jq_args = [filter_expr]
        if input_file:
            jq_args.append(input_file)
        if args:
            jq_args.extend(args)

        command = self.get_tool_command("jq", fallback=None, args=jq_args)
        return self.execute(command)

    def sd_substitute(
        self,
        pattern: str,
        replacement: str,
        input_file: str | None = None,
        args: list[str] | None = None,
    ) -> BashResult:
        """Execute sd substitute."""
        sd_args = [pattern, replacement]
        if input_file:
            sd_args.append(input_file)
        if args:
            sd_args.extend(args)

        command = self.get_tool_command("sd", "sed", sd_args)
        return self.execute(command)
