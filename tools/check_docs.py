#!/usr/bin/env python3
"""Check tracked Markdown links and README image paths without network access."""
from html import unescape
from pathlib import Path
import re
import subprocess
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
names = subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT, text=True, encoding="utf-8").split('\0')
failures = []
checked = 0
for name in names:
    file = ROOT / name
    if file.suffix != '.md' or not file.is_file():
        continue
    text = re.sub(r'```.*?```', '', file.read_text(encoding="utf-8"), flags=re.S)
    links = re.findall(r'\[[^\]]*\]\(([^\s)]+)\)', text)
    links += re.findall(r'(?:src|srcset|href)="([^"]+)"', text)
    for link in links:
        parsed = urlsplit(unescape(link))
        if parsed.scheme or parsed.netloc or not parsed.path:
            continue
        path = (file.parent / unquote(parsed.path)).resolve()
        checked += 1
        if not path.is_relative_to(ROOT) or not path.exists():
            failures.append(f'{name}: missing local target {link}')
if failures:
    raise SystemExit('\n'.join(failures))
print(f'Documentation links passed: {checked} local targets.')
