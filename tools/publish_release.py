#!/usr/bin/env python3
"""Publish complete immutable releases, then advance signed update channels."""
import hashlib
import json
import os
import subprocess
from pathlib import Path
from release import check
from update_feed import REPO, build_feed, publish_channels
from release_notes import generate


def existing_state(release, marker, files):
    if marker not in release['body']:
        raise ValueError('This release belongs to another commit. Use a new version.')
    expected={p.name for p in files}
    if release['draft']:
        if any(a['name'] not in expected for a in release['assets']):
            raise ValueError('The draft contains unexpected assets.')
        return 'draft'
    actual={a['name']:a.get('digest') for a in release['assets']}
    if set(actual)!=expected or any(actual[p.name]!='sha256:'+hashlib.sha256(p.read_bytes()).hexdigest() for p in files):
        raise ValueError('Published assets differ. Published releases are never overwritten.')
    return 'published'


def main():
    version=check(os.environ['GITHUB_REF_NAME'],on_main=True)
    repo=os.environ['GITHUB_REPOSITORY'];assert repo==REPO
    assets=Path('release-assets');files=[];records={}
    for platform in ('macos-arm64','windows-x64','linux-x64'):
        record=json.loads((assets/f'{platform}.json').read_text());records[platform]=record
        assert record['version']==version and record['updater']
        if platform=='macos-arm64': assert record['signing']=='Developer ID + notarized' and record['disk_image_notarized']
        for name in record['files']:
            path=assets/name;assert path.is_file() and path.parent==assets;files.append(path)
    changes=Path('docs/release-notes')/f'{version}.md'
    description=changes.read_text().strip() if changes.exists() else ''
    date=subprocess.check_output(['git','show','-s','--format=%cI','HEAD'],text=True).strip()
    feed=build_feed(version,records,assets,description,date)
    metadata=assets/'update.json';metadata.write_text(json.dumps(feed,indent=2)+'\n');files.append(metadata)
    checksum=assets/'SHA256SUMS';checksum.write_text(''.join(f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n' for p in sorted(files)));files.append(checksum)
    tag=f'v{version}';marker='<!-- local-chat-release:'+os.environ['GITHUB_SHA']+' -->'
    existing=subprocess.run(['gh','api',f'repos/{repo}/releases/tags/{tag}'],capture_output=True,text=True)
    state='new'
    if existing.returncode==0: state=existing_state(json.loads(existing.stdout),marker,files)
    elif '404' not in existing.stderr: raise RuntimeError('Could not check existing release')
    if state!='published':
        notes=assets/'release-notes.md'
        # Freeze draft notes at creation so retries do not change attribution or text.
        body = marker + '\n' + generate(version, os.environ['GITHUB_SHA']) if state == 'new' else json.loads(existing.stdout)['body']
        notes.write_text(body)
        if state=='new':
            args=['gh','release','create',tag,'--repo',repo,'--verify-tag','--draft','--title',f'Local Chat {version}','--notes-file',str(notes)]
            if '-' in version: args.append('--prerelease')
            subprocess.run(args,check=True)
        upload=['gh','release','upload',tag,'--repo',repo,*map(str,files)]
        if state=='draft': upload.append('--clobber')
        subprocess.run(upload,check=True)
        subprocess.run(['gh','release','edit',tag,'--repo',repo,'--draft=false','--latest=false' if '-' in version else '--latest'],check=True)
    # Resuming after publication only verifies existing immutable bytes and repairs
    # the feed. It never reuploads or modifies published assets.
    publish_channels(feed)

if __name__=='__main__':main()
