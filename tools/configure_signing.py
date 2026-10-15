#!/usr/bin/env python3
"""Validate Apple login and store release credentials directly in GitHub Secrets."""
import argparse
import base64
import getpass
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--certificate', type=Path, help='Developer ID Application identity exported as a password-protected .p12')
parser.add_argument('--notarization-only', action='store_true', help='Update only Apple login; keep the existing signing certificate')
parser.add_argument('--repo', default='youssefsz/local-chat')
args = parser.parse_args()
values = {}
if not args.notarization_only:
    if args.certificate is None:
        parser.error('--certificate is required unless --notarization-only is selected')
    certificate = args.certificate.expanduser().resolve()
    if certificate.suffix.lower() != '.p12' or not certificate.is_file():
        raise SystemExit('Select the exported Developer ID Application .p12 file.')
    password = getpass.getpass('Certificate export password: ')
    if not password:
        raise SystemExit('Certificate password is required. Nothing was uploaded.')
    values.update({
        'APPLE_CERTIFICATE': base64.b64encode(certificate.read_bytes()).decode(),
        'APPLE_CERTIFICATE_PASSWORD': password,
        'APPLE_SIGNING_IDENTITY': 'Developer ID Application: Youssef Dhibi (5S4ZT4SRV9)',
        'APPLE_TEAM_ID': '5S4ZT4SRV9',
    })
apple_id = input('Apple ID email belonging to Developer team 5S4ZT4SRV9: ').strip()
apple_password = getpass.getpass('New Apple app-specific password, not your account password: ').strip()
if not all((apple_id, apple_password)):
    raise SystemExit('Both values are required. Nothing was uploaded.')
print('Checking login with Apple’s notarization service. No app is being uploaded.', flush=True)
try:
    login = subprocess.run([
        'xcrun', 'notarytool', 'history', '--apple-id', apple_id,
        '--password', apple_password, '--team-id', '5S4ZT4SRV9', '--output-format', 'json',
    ], capture_output=True, timeout=120)
except (OSError, subprocess.TimeoutExpired):
    raise SystemExit('Apple login could not be checked. Use a Mac with Xcode command-line tools and retry. No secrets were changed.')
if login.returncode:
    unauthorized = b'401' in login.stderr or b'401' in login.stdout
    raise SystemExit(('Apple returned HTTP 401. ' if unauthorized else 'Apple login check failed. ') + 'Verify the Apple ID and generate an app-specific password at account.apple.com, then retry. No GitHub secrets were changed.')
values.update({'APPLE_ID': apple_id, 'APPLE_PASSWORD': apple_password})
for name, value in values.items():
    result = subprocess.run(['gh', 'secret', 'set', name, '--repo', args.repo], input=value.encode(), capture_output=True)
    if result.returncode:
        raise SystemExit(f'Could not store {name}. Rerun setup to finish; no values were printed.')
print(f'Apple login verified. {len(values)} GitHub release secrets configured. No credentials were written into the repository.')
