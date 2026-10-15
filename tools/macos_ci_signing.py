#!/usr/bin/env python3
"""Import only the release identity into an ephemeral CI keychain. Never print secrets."""
import base64
import os
import secrets
import subprocess
from pathlib import Path

REQUIRED = ('APPLE_CERTIFICATE', 'APPLE_CERTIFICATE_PASSWORD', 'APPLE_SIGNING_IDENTITY', 'APPLE_ID', 'APPLE_PASSWORD', 'APPLE_TEAM_ID')
missing = [name for name in REQUIRED if not os.environ.get(name)]
if missing:
    raise SystemExit('Missing release secrets: ' + ', '.join(missing) + '. See docs/releases.md; signed releases never fall back to ad-hoc signing.')
folder = Path(os.environ['RUNNER_TEMP'])
keychain = folder / 'local-chat-release.keychain-db'
certificate = folder / 'local-chat-release.p12'
password = secrets.token_urlsafe(40)

def run(*args):
    # security errors can include arguments. Suppress them rather than leaking credentials into a build log.
    result = subprocess.run(['security', *args], capture_output=True)
    if result.returncode:
        raise SystemExit('The temporary signing keychain could not be prepared. Check the configured certificate and password.')
    return result.stdout.decode()

try:
    certificate.write_bytes(base64.b64decode(os.environ['APPLE_CERTIFICATE'], validate=True))
    certificate.chmod(0o600)
    run('create-keychain', '-p', password, str(keychain))
    run('set-keychain-settings', '-lut', '21600', str(keychain))
    run('unlock-keychain', '-p', password, str(keychain))
    run('import', str(certificate), '-k', str(keychain), '-P', os.environ['APPLE_CERTIFICATE_PASSWORD'], '-T', '/usr/bin/codesign', '-T', '/usr/bin/security')
    run('set-key-partition-list', '-S', 'apple-tool:,apple:,codesign:', '-s', '-k', password, str(keychain))
    previous = run('list-keychains', '-d', 'user').splitlines()
    search = [line.strip().strip('"') for line in previous if line.strip()]
    run('list-keychains', '-d', 'user', '-s', str(keychain), *search)
    identities = run('find-identity', '-v', '-p', 'codesigning', str(keychain))
    if os.environ['APPLE_SIGNING_IDENTITY'] not in identities:
        raise SystemExit('The imported certificate does not match APPLE_SIGNING_IDENTITY.')
    config = folder / 'local-chat-signing.json'
    import json
    config.write_text(json.dumps({'bundle': {'createUpdaterArtifacts': True, 'macOS': {'signingIdentity': os.environ['APPLE_SIGNING_IDENTITY']}}}))
    print('Release signing identity imported into temporary keychain.')
finally:
    certificate.unlink(missing_ok=True)
