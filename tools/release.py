#!/usr/bin/env python3
"""One version across Cargo, npm, Tauri, lockfiles, and immutable release tags."""
import argparse
import json
import re
import subprocess
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = re.compile(r'(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-(beta|rc)\.([1-9]\d*))?\Z')

def validate(version):
    if not VERSION.fullmatch(version):
        raise ValueError('Use X.Y.Z, X.Y.Z-beta.N, or X.Y.Z-rc.N without leading zeroes')
    return version

def versions():
    package = json.loads((ROOT / 'package.json').read_text())
    npm_lock = json.loads((ROOT / 'package-lock.json').read_text())
    cargo = tomllib.loads((ROOT / 'src-tauri/Cargo.toml').read_text())
    cargo_lock = tomllib.loads((ROOT / 'src-tauri/Cargo.lock').read_text())
    return {
        'package.json': package['version'],
        'package-lock.json': npm_lock['version'],
        'package-lock.json root': npm_lock['packages']['']['version'],
        'Cargo.toml': cargo['package']['version'],
        'Cargo.lock': next(p['version'] for p in cargo_lock['package'] if p['name'] == 'local-chat'),
        'tauri.conf.json': json.loads((ROOT / 'src-tauri/tauri.conf.json').read_text())['version'],
    }

def check(tag=None, on_main=False):
    values = versions()
    version = validate(values['package.json'])
    if any(v != version for v in values.values()):
        raise ValueError(f'Version mismatch: {values}')
    if tag is not None and tag != f'v{version}':
        raise ValueError(f'Tag {tag!r} must equal v{version}')
    if on_main:
        subprocess.run(['git', 'merge-base', '--is-ancestor', 'HEAD', 'origin/main'], cwd=ROOT, check=True)
    return version

def set_version(version):
    validate(version)
    for name in ['package.json', 'package-lock.json', 'src-tauri/tauri.conf.json']:
        path = ROOT / name
        data = json.loads(path.read_text())
        data['version'] = version
        if name == 'package-lock.json':
            data['packages']['']['version'] = version
        path.write_text(json.dumps(data, indent=2) + '\n')
    path = ROOT / 'src-tauri/Cargo.toml'
    text, count = re.subn(r'(?m)^version = "[^"]+"', f'version = "{version}"', path.read_text(), count=1)
    assert count == 1
    path.write_text(text)
    path = ROOT / 'src-tauri/Cargo.lock'
    text, count = re.subn(r'(\[\[package\]\]\nname = "local-chat"\nversion = ")[^"]+', lambda m: m[1] + version, path.read_text())
    assert count == 1
    path.write_text(text)
    return check()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    p = commands.add_parser('check'); p.add_argument('--tag'); p.add_argument('--on-main', action='store_true')
    p = commands.add_parser('set'); p.add_argument('version')
    args = parser.parse_args()
    version = set_version(args.version) if args.command == 'set' else check(args.tag, args.on_main)
    print(version)

if __name__ == '__main__':
    main()
