#!/usr/bin/env python3
"""Run named test suites and optionally save a data-free CI result summary."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def commands(suite):
    python = sys.executable
    if suite == 'rust':
        return [['cargo', 'test', '--locked', '--manifest-path', 'src-tauri/Cargo.toml', '--no-default-features', '--lib']]
    if suite == 'contracts':
        return [[python, '-m', 'unittest', 'discover', '-s', 'tests/contracts', '-p', 'test_*.py']]
    if suite == 'signatures':
        return [[python, 'tests/integration/test_update_signatures.py']]
    if suite == 'bonjour':
        return [[python, 'tests/integration/test_bonjour_recovery.py']] if sys.platform == 'darwin' else []
    if suite == 'integration':
        if sys.platform != 'darwin':
            raise SystemExit('The real-process discovery harness currently supports macOS. See docs/testing.md for other platform checks.')
        return [['cargo', 'build', '--locked', '--manifest-path', 'src-tauri/Cargo.toml', '--no-default-features', '--features', 'test-profile', '--example', 'local-chat-peer'],
                [python, 'tests/integration/peer_integration.py', '--counts', '2', '5', '20'],
                [python, 'tests/integration/reconnect_integration.py']]
    return [command for name in ('rust', 'contracts', 'signatures', 'bonjour') for command in commands(name)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('suite', choices=['rust', 'contracts', 'signatures', 'bonjour', 'integration', 'all'])
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    outcomes = []
    code = 0
    for command in commands(args.suite):
        started = time.monotonic()
        result = subprocess.run(command, cwd=ROOT)
        outcomes.append({'command': command[1:] if command[0] == sys.executable else command,
                         'exit_code': result.returncode, 'seconds': round(time.monotonic() - started, 2)})
        if result.returncode:
            code = result.returncode
            break
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps({'suite': args.suite, 'status': 'passed' if code == 0 else 'failed', 'checks': outcomes}, indent=2) + '\n')
    return code


if __name__ == '__main__':
    raise SystemExit(main())
