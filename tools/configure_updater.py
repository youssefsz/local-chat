#!/usr/bin/env python3
"""Create/reuse one private updater identity outside Git and configure CI securely."""
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
folder = Path.home()/'.codex/local-chat-signing'
folder.mkdir(parents=True, exist_ok=True, mode=0o700)
key = folder/'LocalChat-Updater.key'
if not key.exists():
    subprocess.run([str(ROOT/'node_modules/.bin/tauri'), 'signer', 'generate', '--ci', '-p', '', '-w', str(key)], check=True, capture_output=True)
key.chmod(0o600)
public = key.with_suffix('.key.pub').read_text().strip()
target = ROOT/'src-tauri/updater.pub'
if target.exists() and target.read_text().strip() != public:
    raise SystemExit('Existing public key differs. Do not rotate an installed updater identity accidentally.')
config_path=ROOT/'src-tauri/tauri.conf.json'
config=json.loads(config_path.read_text())
configured=config.get('plugins',{}).get('updater',{}).get('pubkey')
if configured and configured.strip()!=public:
    raise SystemExit('Configured public key differs. Restore the matching updater identity before changing CI secrets.')
subprocess.run(['gh', 'secret', 'set', 'TAURI_SIGNING_PRIVATE_KEY', '--repo', 'youssefsz/local-chat'], input=key.read_bytes(), check=True, capture_output=True)
target.write_text(public+'\n')
config.setdefault('plugins',{}).setdefault('updater',{})['pubkey']=public
config_path.write_text(json.dumps(config,indent=2)+'\n')
print('Updater identity configured. Only the public key is in the repository; the private key is retained outside Git.')
