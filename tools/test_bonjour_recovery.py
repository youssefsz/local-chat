"""Exercise fatal and interrupted poll calls through the real macOS Bonjour adapter."""
from pathlib import Path
import socket
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
interface = next(name for _, name in socket.if_nameindex() if name.startswith('en'))
with tempfile.TemporaryDirectory(prefix='local-chat-poll-') as folder:
    executable = str(Path(folder)/'bonjour-recovery')
    subprocess.run(['clang', '-std=c11', '-Wall', '-Wextra', '-Werror',
                    str(root/'tools/test_bonjour_recovery.c'), '-lpthread', '-o', executable], check=True)
    subprocess.run([executable, interface], check=True, timeout=15)
