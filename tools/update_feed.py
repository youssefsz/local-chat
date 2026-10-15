#!/usr/bin/env python3
"""Build deterministic updater metadata and publish monotonic channel feeds."""
import base64
import json
import subprocess
from pathlib import Path
from release import validate

REPO = 'youssefsz/local-chat'
PLATFORMS = {'macos-arm64':'darwin-aarch64','windows-x64':'windows-x86_64','linux-x64':'linux-x86_64'}

def version_key(version):
    validate(version)
    core, _, pre = version.partition('-')
    stage, _, number = pre.partition('.')
    return (*map(int,core.split('.')), {'':2,'rc':1,'beta':0}[stage], int(number or 0))

def build_feed(version, records, assets, notes, date):
    validate(version)
    platforms = {}
    for platform, target in PLATFORMS.items():
        record=records[platform]
        assert record['version']==version
        update=record['updater']
        name=update['file']; signature=update['signature']
        expected=f'Local-Chat_{version}_{platform}'+{ 'macos-arm64':'.app.tar.gz','windows-x64':'.exe','linux-x64':'.AppImage'}[platform]
        assert name==expected and signature==name+'.sig'
        assert (assets/name).is_file() and 0<(assets/name).stat().st_size<=256*1024*1024
        signed=(assets/signature).read_text().strip()
        assert len(base64.b64decode(signed,validate=True))>64
        platforms[target]={'url':f'https://github.com/{REPO}/releases/download/v{version}/{name}','signature':signed}
    return {'version':version,'notes':notes,'pub_date':date,'platforms':platforms}

def gh(path, data=None):
    args=['gh','api',path]
    if data is not None: args+=['--method','PUT','--input','-']
    result=subprocess.run(args,input=json.dumps(data) if data is not None else None,capture_output=True,text=True)
    if result.returncode: raise RuntimeError(f'GitHub channel operation failed: {result.stderr.strip()}')
    return json.loads(result.stdout)

def channel_file(name):
    result=subprocess.run(['gh','api',f'repos/{REPO}/contents/{name}.json?ref=updates'],capture_output=True,text=True)
    if result.returncode:
        if '404' in result.stderr: return None,None
        raise RuntimeError('Could not read update channel')
    record=json.loads(result.stdout)
    return record['sha'],json.loads(base64.b64decode(record['content']))

def publish_channels(feed):
    ref=subprocess.run(['gh','api',f'repos/{REPO}/git/ref/heads/updates'],capture_output=True,text=True)
    if ref.returncode:
        if '404' not in ref.stderr: raise RuntimeError('Could not read update feed branch')
        commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
        subprocess.run(['gh','api',f'repos/{REPO}/git/refs','--method','POST','-f','ref=refs/heads/updates','-f',f'sha={commit}'],check=True,capture_output=True)
    # Static JSON has no HTTP 204 response. Version zero with no targets is an
    # empty stable channel, never a downloadable release or an installer version.
    stable_sha, _ = channel_file('stable')
    if stable_sha is None:
        empty={'version':'0.0.0','notes':'No stable release has been published.','platforms':{}}
        gh(f'repos/{REPO}/contents/stable.json', {'message':'Initialize empty stable update channel','branch':'updates','content':base64.b64encode((json.dumps(empty)+'\n').encode()).decode()})
    names=['beta'] if '-' in feed['version'] else ['stable','beta']
    for name in names:
        sha,existing=channel_file(name)
        if existing and version_key(existing['version'])>=version_key(feed['version']):
            if existing['version']==feed['version'] and existing!=feed:
                raise RuntimeError('This channel version already has different metadata')
            continue
        body={'message':f'Publish {name} update {feed["version"]}','branch':'updates','content':base64.b64encode((json.dumps(feed,indent=2)+'\n').encode()).decode()}
        if sha: body['sha']=sha
        gh(f'repos/{REPO}/contents/{name}.json',body)
    print('Update channels published after complete release; existing newer versions retained.')
