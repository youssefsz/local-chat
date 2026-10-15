#!/usr/bin/env python3
"""Store macOS signing credentials directly in GitHub Secrets, without echoing them."""
import argparse
import base64
import getpass
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--certificate', type=Path, required=True, help='Developer ID Application identity exported as a password-protected .p12')
parser.add_argument('--repo', default='youssefsz/local-chat')
args = parser.parse_args()
certificate = args.certificate.expanduser().resolve()
if certificate.suffix.lower() != '.p12' or not certificate.is_file():
    raise SystemExit('Select the exported Developer ID Application .p12 file.')
password = getpass.getpass('Certificate export password: ')
apple_id = input('Apple ID email for notarization: ').strip()
apple_password = getpass.getpass('Apple app-specific password, not your account password: ')
if not all((password, apple_id, apple_password)):
    raise SystemExit('All values are required. Nothing was uploaded.')
values = {
    'APPLE_CERTIFICATE': base64.b64encode(certificate.read_bytes()).decode(),
    'APPLE_CERTIFICATE_PASSWORD': password,
    'APPLE_SIGNING_IDENTITY': 'Developer ID Application: Youssef Dhibi (5S4ZT4SRV9)',
    'APPLE_TEAM_ID': '5S4ZT4SRV9',
    'APPLE_ID': apple_id,
    'APPLE_PASSWORD': apple_password,
}
for name, value in values.items():
    result = subprocess.run(['gh', 'secret', 'set', name, '--repo', args.repo], input=value.encode(), capture_output=True)
    if result.returncode:
        raise SystemExit(f'Could not store {name}. Rerun the setup to finish; no values were printed.')
print('Six macOS release secrets configured. No credential files were written into the repository.')
