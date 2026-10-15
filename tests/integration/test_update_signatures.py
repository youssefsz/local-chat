#!/usr/bin/env python3
"""Exercise valid, tampered, wrong-key and wrong-version packages with real keys."""
from pathlib import Path
import subprocess
import tempfile
ROOT=Path(__file__).resolve().parents[2]
cli=['node',str(ROOT/'node_modules/@tauri-apps/cli/tauri.js')]
with tempfile.TemporaryDirectory(prefix='local-chat-update-signature-') as d:
    folder=Path(d);package=folder/'package.bin';package.write_bytes(b'Synthetic updater signature verification\n'*100)
    keys=[]
    for n in ['correct','wrong']:
        key=folder/(n+'.key');subprocess.run([*cli,'signer','generate','--ci','-p','','-w',str(key)],check=True,capture_output=True);keys.append(key)
    subprocess.run([*cli,'signer','sign','-f',str(keys[0]),'-p','','--app-version','0.1.0-beta.6',str(package)],check=True,capture_output=True)
    command=['cargo','run','--quiet','--manifest-path',str(ROOT/'src-tauri/Cargo.toml'),'--no-default-features','--features','release-tools','--example','local-chat-update-verify','--']
    def verify(key, version='0.1.0-beta.6'):
        return subprocess.run([*command,str(key)+'.pub',str(package)+'.sig',str(package),version],capture_output=True).returncode
    assert verify(keys[0])==0,'Valid update signature rejected'
    assert verify(keys[1])!=0,'Wrong signing identity accepted'
    assert verify(keys[0],'0.1.0-beta.7')!=0,'Manifest version substitution accepted'
    package.write_bytes(package.read_bytes()+b'tampered')
    assert verify(keys[0])!=0,'Tampered update accepted'
print('Update signatures: valid, wrong key, wrong version, tampering all passed')
