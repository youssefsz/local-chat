#!/usr/bin/env python3
"""Publish only after all platform packages exist and match the tagged version."""
import hashlib
import json
import os
import subprocess
from pathlib import Path
from release import check

version=check(os.environ['GITHUB_REF_NAME'], on_main=True)
assets=Path('release-assets')
files=[]
for platform in ('macos-arm64','windows-x64','linux-x64'):
    record=json.loads((assets/f'{platform}.json').read_text())
    assert record['version']==version
    if platform=='macos-arm64': assert record['signing']=='Developer ID + notarized'
    for name in record['files']:
        path=assets/name
        assert path.is_file() and path.parent==assets
        files.append(path)
checksum=assets/'SHA256SUMS'
checksum.write_text(''.join(f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n' for p in sorted(files)))
files.append(checksum)
tag=f'v{version}'
repo=os.environ['GITHUB_REPOSITORY']
if subprocess.run(['gh','release','view',tag,'--repo',repo],capture_output=True).returncode==0:
    raise SystemExit('This release already exists. Use a new version; published assets are never overwritten.')
notes=assets/'release-notes.md'
notes.write_text(f'''Local Chat {version}

Downloads: Apple Silicon macOS, Windows x64, Linux x64. Windows is unsigned and may display SmartScreen warnings. The macOS app is Developer ID signed and notarized.

Windows/Linux networking adapters are implemented and build-tested. Physical cross-device discovery, firewall behavior, and desktop integration still need validation on those operating systems. See docs/platform-status.md.

Use SHA256SUMS to verify downloads. No automatic updater is included in this release.
''')
args=['gh','release','create',tag,'--repo',repo,'--verify-tag','--draft','--title',f'Local Chat {version}','--notes-file',str(notes)]
if '-' in version: args.append('--prerelease')
subprocess.run(args,check=True)
subprocess.run(['gh','release','upload',tag,'--repo',repo,*map(str,files)],check=True)
# A failed upload leaves a draft, never a partially populated published release.
subprocess.run(['gh','release','edit',tag,'--repo',repo,'--draft=false','--latest=false' if '-' in version else '--latest'],check=True)
