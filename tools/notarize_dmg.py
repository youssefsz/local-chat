#!/usr/bin/env python3
"""Notarize and staple the actual signed download container before publication."""
import json
import os
import subprocess
from pathlib import Path
from release import ROOT, check

check()
folder = ROOT / 'src-tauri/target/aarch64-apple-darwin/release/bundle/dmg'
images = list(folder.glob('*.dmg'))
if len(images) != 1:
    raise SystemExit('Expected exactly one freshly built DMG.')
image = images[0]
subprocess.run(['codesign', '--verify', '--strict', str(image)], check=True)
print('Submitting the signed DMG to Apple for notarization.', flush=True)
result = subprocess.run([
    'xcrun', 'notarytool', 'submit', str(image),
    '--apple-id', os.environ['APPLE_ID'], '--password', os.environ['APPLE_PASSWORD'],
    '--team-id', os.environ['APPLE_TEAM_ID'], '--wait', '--output-format', 'json',
], capture_output=True, text=True)
if result.returncode:
    # Avoid printing authentication arguments or server output containing account data.
    raise SystemExit('DMG notarization failed. Inspect the Apple account credentials or notarization service status; no release was published.')
record = json.loads(result.stdout)
if record.get('status') != 'Accepted':
    raise SystemExit(f"Apple did not accept the DMG. Submission ID: {record.get('id', 'unavailable')}")
subprocess.run(['xcrun', 'stapler', 'staple', str(image)], check=True)
subprocess.run(['xcrun', 'stapler', 'validate', str(image)], check=True)
subprocess.run(['spctl', '--assess', '--type', 'open', '--context', 'context:primary-signature', '--verbose=4', str(image)], check=True)
output = ROOT / 'release-assets'
output.mkdir(exist_ok=True)
(output/'macos-notarization.json').write_text(json.dumps({
    'submission_id': record['id'], 'status': 'Accepted', 'disk_image_ticket_stapled': True,
    'disk_image_gatekeeper': 'accepted',
}, indent=2)+'\n')
print('DMG accepted by Apple, stapled, and accepted by Gatekeeper.')
