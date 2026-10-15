"""Checks the release contract rather than generating a version or a Git tag."""
import unittest
import struct
from prepare_artifacts import verify_architecture
from release import validate, check

class ReleaseContract(unittest.TestCase):
    def test_allowed_versions(self):
        for version in ['0.1.0-beta.1', '0.1.0-beta.20', '0.1.0-rc.1', '0.1.0', '1.2.3']:
            self.assertEqual(validate(version), version)
    def test_invalid_versions(self):
        for version in ['0.01', '01.0.0', '0.1.0-beta.01', '0.1.0-beta.0', 'v0.1.0', '0.1.0;echo bad', '../main']:
            with self.assertRaises(ValueError): validate(version)
    def test_wrong_architecture_is_rejected(self):
        arm = bytearray(64); arm[:4] = b'\xcf\xfa\xed\xfe'; struct.pack_into('<I',arm,4,0x100000C)
        verify_architecture(arm,'macos-arm64')
        struct.pack_into('<I',arm,4,0x1000007)
        with self.assertRaises(AssertionError): verify_architecture(arm,'macos-arm64')
        linux = bytearray(64); linux[:5] = b'\x7fELF\x02'; struct.pack_into('<H',linux,18,62)
        verify_architecture(linux,'linux-x64')
        struct.pack_into('<H',linux,18,183)
        with self.assertRaises(AssertionError): verify_architecture(linux,'linux-x64')
        windows = bytearray(128); windows[:2] = b'MZ'; struct.pack_into('<I',windows,0x3c,64); windows[64:68] = b'PE\0\0'; struct.pack_into('<H',windows,68,0x8664)
        verify_architecture(windows,'windows-x64')
        struct.pack_into('<H',windows,68,0xaa64)
        with self.assertRaises(AssertionError): verify_architecture(windows,'windows-x64')
    def test_current_manifests_agree(self):
        version = check()
        self.assertEqual(check(f'v{version}'), version)
        with self.assertRaises(ValueError): check('v99.99.99')

if __name__ == '__main__': unittest.main()
