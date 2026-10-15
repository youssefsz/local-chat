#!/usr/bin/env python3
"""Verify the actual macOS distribution bundle, including test-build exclusion."""
import argparse
import hashlib
import json
import pathlib
import plistlib
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('app', type=pathlib.Path)
parser.add_argument('--output', type=pathlib.Path)
args = parser.parse_args()
app = args.app.resolve()
info = plistlib.loads((app / 'Contents/Info.plist').read_bytes())
assert info['CFBundleIdentifier'] == 'app.localchat.desktop', 'Wrong bundle identifier'
assert info['NSBonjourServices'] == ['_localchat._tcp'], 'Wrong discovery declaration'
assert info.get('NSLocalNetworkUsageDescription'), 'Missing permission explanation'
programs = sorted(p.name for p in (app / 'Contents/MacOS').iterdir())
assert programs == ['local-chat'], f'Unexpected bundled programs: {programs}'
binary = app / 'Contents/MacOS/local-chat'
data = binary.read_bytes()
markers = ['LOCAL_CHAT_TEST_DATA', 'LOCAL_CHAT_TEST_SERVICE', '_localchat-test._tcp', 'window.json', 'startup.json', 'Dedicated test startup failure']
for marker in markers:
    assert marker.encode() not in data, f'Test-only marker present: {marker}'
verified = subprocess.run(['codesign', '--verify', '--deep', '--strict', '--verbose=2', str(app)], capture_output=True, text=True, check=True)
signature = subprocess.run(['codesign', '--display', '--verbose=4', str(app)], capture_output=True, text=True, check=True)
report = {
    'bundle_identifier': info['CFBundleIdentifier'],
    'version': info['CFBundleShortVersionString'],
    'executables': programs,
    'test_only_markers_absent': markers,
    'binary_sha256': hashlib.sha256(data).hexdigest(),
    'architecture': subprocess.check_output(['file', str(binary)], text=True).strip(),
    'signature_verification': verified.stderr.strip(),
    'signature': signature.stderr.strip(),
    'notarization': 'Not assessed by this script. Inspect the build log and stapled ticket separately.',
}
result = json.dumps(report, indent=2) + '\n'
if args.output:
    args.output.write_text(result)
print(result)
