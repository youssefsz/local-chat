#!/usr/bin/env python3
"""Serve signed local packages to a dedicated test-profile app. Never production."""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import plistlib
import subprocess
import tarfile
import time
from urllib.parse import urlparse

ROOT=Path(__file__).resolve().parents[2]

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app',type=Path,required=True)
    parser.add_argument('--folder',type=Path,required=True)
    parser.add_argument('--data',type=Path,required=True)
    args=parser.parse_args();folder=args.folder.resolve();folder.mkdir(parents=True,exist_ok=True,mode=0o700);folder.chmod(0o700)
    info=plistlib.loads((args.app/'Contents/Info.plist').read_bytes())
    assert info['CFBundleIdentifier']=='app.localchat.desktop.qa','Only a dedicated QA bundle is allowed'
    version=info['CFBundleShortVersionString'];package=folder/'update.app.tar.gz'
    with tarfile.open(package,'w:gz') as archive:archive.add(args.app,arcname='Local Chat Test.app')
    cli=['node',str(ROOT/'node_modules/@tauri-apps/cli/tauri.js')]
    key=folder/'fixture.key'
    if not key.exists():subprocess.run([*cli,'signer','generate','--ci','-p','','-w',str(key)],check=True,capture_output=True)
    key.chmod(0o600)
    subprocess.run([*cli,'signer','sign','-f',str(key),'-p','','--app-version',version,str(package)],check=True,capture_output=True)
    signature=Path(str(package)+'.sig').read_text().strip();control=folder/'mode.json'
    if not control.exists():control.write_text('{"mode":"current"}')
    class Handler(BaseHTTPRequestHandler):
        def log_message(self,*args):pass
        def do_GET(self):
            mode=json.loads(control.read_text())['mode'];path=urlparse(self.path).path
            if path=='/manifest.json':
                if mode=='unavailable':self.send_error(503);return
                candidate=version if mode!='current' else '0.1.0-beta.5'
                if mode=='wrong-version':candidate='0.1.0-beta.999'
                sig=signature
                if mode=='bad-signature':sig=signature[:-8]+'AAAAAAAA'
                data=json.dumps({'version':candidate,'notes':'Synthetic update fixture. Your real conversations are not used.\nChecks download verification and restart.','pub_date':'2026-10-15T00:00:00Z','platforms':{'darwin-aarch64':{'url':f'http://127.0.0.1:{server.server_port}/package','signature':sig}}}).encode()
                self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(data)));self.end_headers();self.wfile.write(data)
            elif path=='/package':
                size=package.stat().st_size
                self.send_response(200);self.send_header('Content-Length',str(size));self.end_headers()
                try:
                    with package.open('rb') as file:
                        while chunk:=file.read(65536):self.wfile.write(chunk);time.sleep(0.005)
                except (BrokenPipeError,ConnectionResetError):pass
            else:self.send_error(404)
    server=ThreadingHTTPServer(('127.0.0.1',0),Handler)
    args.data.mkdir(parents=True,exist_ok=True)
    (args.data/'update-test.json').write_text(json.dumps({'endpoint':f'http://127.0.0.1:{server.server_port}/manifest.json','pubkey':Path(str(key)+'.pub').read_text().strip()}))
    print(json.dumps({'fixture':'ready','version':version,'port':server.server_port,'control':str(control)}),flush=True)
    server.serve_forever()

if __name__=='__main__':main()
