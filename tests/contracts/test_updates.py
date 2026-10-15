import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
import base64
from pathlib import Path
import tempfile
import unittest
from publish_release import existing_state
from update_feed import build_feed, version_key, channels_for, PLATFORMS

class Updates(unittest.TestCase):
    def test_published_release_is_verified_without_overwrite(self):
        import hashlib
        with tempfile.TemporaryDirectory() as d:
            file=Path(d)/'installer';file.write_bytes(b'verified installer')
            record={'body':'marker','draft':False,'assets':[{'name':file.name,'digest':'sha256:'+hashlib.sha256(file.read_bytes()).hexdigest()}]}
            self.assertEqual(existing_state(record,'marker',[file]),'published')
            file.write_bytes(b'changed')
            with self.assertRaises(ValueError):existing_state(record,'marker',[file])
            with self.assertRaises(ValueError):existing_state(record,'other commit',[file])
    def test_channel_ordering(self):
        self.assertEqual(channels_for('0.0.1'), ('stable', 'beta-public'))
        self.assertEqual(channels_for('0.0.2-beta.1'), ('beta-public',))
        self.assertNotIn('beta', channels_for('0.0.1'))
        versions=['0.1.0-beta.2','0.1.0-beta.10','0.1.0-rc.1','0.1.0','0.2.0-beta.1']
        self.assertEqual(sorted(versions,key=version_key),versions)
        with self.assertRaises(ValueError): version_key('../main')
    def test_feed_requires_every_platform_and_signature(self):
        with tempfile.TemporaryDirectory() as d:
            folder=Path(d); records={};version='0.1.0-beta.6'
            for platform in PLATFORMS:
                name=f'Local-Chat_{version}_{platform}'+{'macos-arm64':'.app.tar.gz','windows-x64':'.exe','linux-x64':'.AppImage'}[platform]
                (folder/name).write_bytes(b'Synthetic updater package')
                (folder/(name+'.sig')).write_text(base64.b64encode(b'x'*100).decode())
                records[platform]={'version':version,'updater':{'file':name,'signature':name+'.sig'}}
            feed=build_feed(version,records,folder,'Test notes','2026-10-15T00:00:00Z')
            self.assertEqual(set(feed['platforms']),set(PLATFORMS.values()))
            file=folder/records['linux-x64']['updater']['signature'];file.unlink()
            with self.assertRaises(FileNotFoundError): build_feed(version,records,folder,'','')
    def test_no_published_artifact_can_point_elsewhere(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(AssertionError): build_feed('0.1.0-beta.6',{'macos-arm64':{'version':'0.1.0-beta.6','updater':{'file':'../other','signature':'../other.sig'}}},Path(d),'','')

if __name__=='__main__': unittest.main()
