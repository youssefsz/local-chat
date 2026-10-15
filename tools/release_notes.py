#!/usr/bin/env python3
"""Preview release notes from reviewed highlights and verified GitHub history."""
import argparse
import json
import re
import subprocess
from pathlib import Path
from release import ROOT, validate
from update_feed import REPO, version_key


def api(path, body=None, paginate=False):
    args = ['gh', 'api', path]
    if body is not None:
        args += ['--method', 'POST', '--input', '-']
    if paginate:
        args += ['--paginate', '--slurp']
    result = subprocess.run(args, input=json.dumps(body) if body is not None else None,
                            text=True, encoding="utf-8", capture_output=True, check=True)
    return json.loads(result.stdout)


def previous_release(releases, version, commit, ancestor):
    """Ignore failed tags, drafts, newer releases, and releases from other branches."""
    eligible = []
    for release in releases:
        tag = release['tag_name']
        if not tag.startswith('v'):
            continue
        try:
            candidate = validate(tag.removeprefix('v'))
        except ValueError:
            continue
        if not release['draft'] and version_key(candidate) < version_key(version) and ancestor(tag, commit):
            eligible.append(candidate)
    return 'v' + max(eligible, key=version_key) if eligible else None


def text_name(name):
    # Names are public commit attribution, never inferred from an email address.
    return re.sub(r'[\[\]<>`*_\\\r\n]', '', name).strip()[:100]


def contributors(commits):
    people = set()
    for commit in commits:
        user = commit.get('author') or {}
        login = user.get('login', '')
        if login and re.fullmatch(r'[A-Za-z0-9-]+', login) and user.get('type') != 'Bot':
            people.add('@' + login)
        elif not login:
            name = text_name(commit['commit']['author']['name'])
            if name:
                people.add(name)
        for name in re.findall(r'^Co-authored-by:\s*(.*?)\s*<[^>]+>\s*$', commit['commit']['message'], re.M | re.I):
            name = text_name(name)
            if name:
                people.add(name)
    return sorted(people, key=str.casefold)


def render(version, highlights, generated, people, previous):
    validate(version)
    if not highlights.strip():
        raise ValueError('Reviewed release highlights are required')
    base = f'https://github.com/{REPO}/releases/download/v{version}/Local-Chat_{version}_'
    compare = f'https://github.com/{REPO}/compare/{previous}...v{version}' if previous else f'https://github.com/{REPO}/commits/v{version}'
    credits = ', '.join(people) if people else 'No linked contributor attribution was returned for this range.'
    details = generated.strip() + '\n\n' if generated.strip() else ''
    return f'''## Highlights

{highlights.strip()}

## Downloads

| Platform | Package |
| --- | --- |
| macOS Apple Silicon | [DMG]({base}macos-arm64.dmg) |
| Windows x64 | [Installer]({base}windows-x64.exe) |
| Linux x64 | [AppImage]({base}linux-x64.AppImage) · [Debian]({base}linux-x64.deb) |

The Mac app and DMG are Developer ID signed and notarized. Windows is unsigned and may show SmartScreen warnings.
Verify manual downloads with [SHA256SUMS](https://github.com/{REPO}/releases/download/v{version}/SHA256SUMS).
Updater archives and `.sig` files are used by Local Chat's update system.

## Upgrading and limitations

Choose **Check for Updates** in Settings. For the first public 0.0.1 release, users of the earlier 0.1.0-beta test series need a manual installation and must choose the Stable update channel. Updater downgrade protection remains enabled.
Linux AppImage supports in-app updates; Debian installations use downloaded packages. Saved chats stay on the computer.
Physical Windows and Linux desktop, LAN, and update-installation checks remain pending. See [platform status](https://github.com/{REPO}/blob/main/docs/platform-status.md).

## Contributors

Thanks to {credits} for contributions included in this release.

{details}[Full changelog]({compare})
'''


def generate(version, commit):
    validate(version)
    # Resolve references before using them in GitHub paths or comparisons.
    commit = subprocess.check_output(['git', 'rev-parse', '--verify', commit + '^{commit}'], cwd=ROOT, text=True, encoding="utf-8").strip()
    pages = api(f'repos/{REPO}/releases?per_page=100', paginate=True)
    def ancestor(tag, head):
        return subprocess.run(['git', 'merge-base', '--is-ancestor', tag, head], cwd=ROOT, capture_output=True).returncode == 0
    previous = previous_release([r for page in pages for r in page], version, commit, ancestor)
    body = {'tag_name': f'v{version}', 'target_commitish': commit}
    if previous:
        body['previous_tag_name'] = previous
    if subprocess.run(['git', 'cat-file', '-e', f'{commit}:.github/release.yml'], cwd=ROOT, capture_output=True).returncode == 0:
        body['configuration_file_path'] = '.github/release.yml'
    generated = api(f'repos/{REPO}/releases/generate-notes', body)['body']
    if previous:
        pages = api(f'repos/{REPO}/compare/{previous}...{commit}?per_page=100', paginate=True)
        commits = [c for page in pages for c in page['commits']]
    else:
        pages = api(f'repos/{REPO}/commits?sha={commit}&per_page=100', paginate=True)
        commits = [c for page in pages for c in page]
    highlights = (ROOT / 'docs/release-notes' / f'{version}.md').read_text(encoding='utf-8')
    # GitHub already returns a comparison link. Render one explicit link below.
    generated = re.sub(r'^\*\*Full Changelog\*\*:.*$', '', generated, flags=re.M).strip()
    return render(version, highlights, generated, contributors(commits), previous)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--version', required=True)
    parser.add_argument('--commit', default='HEAD')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(generate(args.version, args.commit), encoding="utf-8")
    print(f'Release-note preview written to {args.output}. No release was changed.')
