#!/usr/bin/env python3
"""Reject signing material, local data, and recognizable credentials before publishing."""
import re
import subprocess
from pathlib import Path

patterns = [
    re.compile(rb'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----'),
    re.compile(rb'gh[pousr]_[A-Za-z0-9]{30,}'),
    re.compile(rb'github_pat_[A-Za-z0-9_]{40,}'),
    re.compile(rb'AKIA[0-9A-Z]{16}'),
]
files = subprocess.check_output(['git','ls-files','-z']).decode().split('\0')
problems=[]
for name in filter(None,files):
    path=Path(name)
    if path.suffix in ('.p12','.pfx','.p8','.key','.keychain','.keychain-db') or path.name.startswith('.env') or 'local-chat.sqlite3' in path.name or path.name=='test-identity.json' or name.startswith(('.test-data/','artifacts/','node_modules/','src-tauri/target/','.impeccable/','docs/evidence/')):
        problems.append(f'Private or generated file is tracked: {name}')
    if path.is_file() and re.search(rb'/Users/[A-Za-z0-9._-]+/', path.read_bytes()):
        problems.append(f'Machine-specific home path is tracked: {name}')
    if path.is_file() and any(pattern.search(path.read_bytes()) for pattern in patterns):
        problems.append(f'Possible credential in {name}')
if problems: raise SystemExit('\n'.join(problems))
print(f'Public-tree check passed for {len(files)-1} tracked files. This is a targeted credential/data check, not a complete security audit.')
