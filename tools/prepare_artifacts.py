#!/usr/bin/env python3
"""Verify real release binaries and stage exactly the supported installers."""
import argparse
import json
import shutil
import struct
import subprocess
from pathlib import Path
from release import ROOT, check

TARGETS = {
    'macos-arm64': ('aarch64-apple-darwin', 'local-chat', [('dmg', '*.dmg', '.dmg')]),
    'windows-x64': ('x86_64-pc-windows-msvc', 'local-chat.exe', [('nsis', '*-setup.exe', '.exe')]),
    'linux-x64': ('x86_64-unknown-linux-gnu', 'local-chat', [('appimage', '*.AppImage', '.AppImage'), ('deb', '*.deb', '.deb')]),
}
MARKERS = (b'LOCAL_CHAT_TEST_DATA', b'LOCAL_CHAT_TEST_SERVICE', b'_localchat-test._tcp', b'startup.json', b'window.json')

def verify_architecture(data, platform):
    if platform == 'macos-arm64':
        assert data[:4] == b'\xcf\xfa\xed\xfe' and struct.unpack_from('<I', data, 4)[0] == 0x100000C, 'Expected thin ARM64 Mach-O'
    elif platform == 'windows-x64':
        assert data[:2] == b'MZ', 'Expected PE executable'
        offset = struct.unpack_from('<I', data, 0x3c)[0]
        assert data[offset:offset+4] == b'PE\0\0' and struct.unpack_from('<H', data, offset+4)[0] == 0x8664, 'Expected x64 PE'
    else:
        assert data[:5] == b'\x7fELF\x02' and struct.unpack_from('<H', data, 18)[0] == 62, 'Expected x86-64 ELF'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('platform', choices=TARGETS)
    parser.add_argument('--signed-macos', action='store_true')
    parser.add_argument('--clean', action='store_true', help='Remove only generated bundle output before rebuilding from a cache')
    args = parser.parse_args()
    version = check()
    target, executable, packages = TARGETS[args.platform]
    release = ROOT / 'src-tauri/target' / target / 'release'
    if args.clean:
        bundle = release / 'bundle'
        if bundle.exists(): shutil.rmtree(bundle)
        print('Removed generated bundle output for ' + args.platform)
        return
    data = (release / executable).read_bytes()
    verify_architecture(data, args.platform)
    for marker in MARKERS:
        assert marker not in data, f'Test control compiled into production: {marker.decode()}'
    output = ROOT / 'release-assets'
    output.mkdir(exist_ok=True)
    if args.platform == 'macos-arm64':
        app = release / 'bundle/macos/Local Chat.app'
        subprocess.run(['python3', str(ROOT/'tools/verify_macos_bundle.py'), str(app)], check=True)
        if args.signed_macos:
            subprocess.run(['xcrun', 'stapler', 'validate', str(app)], check=True)
            subprocess.run(['spctl', '--assess', '--type', 'execute', '--verbose=4', str(app)], check=True)
    if args.platform == 'windows-x64':
        # Unsigned Windows is an explicit product decision, not a fallback from failed signing.
        signature = subprocess.check_output(['powershell', '-NoProfile', '-Command', f"(Get-AuthenticodeSignature -LiteralPath '{release / executable}').Status"], text=True).strip()
        assert signature == 'NotSigned', f'Expected unsigned Windows beta, got {signature}'
    names=[]
    for folder, pattern, extension in packages:
        files = list((release / 'bundle' / folder).glob(pattern))
        assert len(files) == 1, f'Expected one {folder} installer, found {len(files)}'
        name = f'Local-Chat_{version}_{args.platform}{extension}'
        shutil.copy2(files[0], output/name)
        names.append(name)
    (output / f'{args.platform}.json').write_text(json.dumps({'version':version,'target':target,'files':names,'signing':'Developer ID + notarized' if args.signed_macos else 'unsigned Windows' if args.platform=='windows-x64' else 'CI test build' if args.platform=='macos-arm64' else 'Linux package'},indent=2)+'\n')
    print('Verified and staged: '+', '.join(names))

if __name__ == '__main__': main()
