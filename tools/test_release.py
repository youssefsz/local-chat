"""Checks the release contract rather than generating a version or a Git tag."""
import unittest
from release import validate, check

class ReleaseContract(unittest.TestCase):
    def test_allowed_versions(self):
        for version in ['0.1.0-beta.1', '0.1.0-beta.20', '0.1.0-rc.1', '0.1.0', '1.2.3']:
            self.assertEqual(validate(version), version)
    def test_invalid_versions(self):
        for version in ['0.01', '01.0.0', '0.1.0-beta.01', '0.1.0-beta.0', 'v0.1.0', '0.1.0;echo bad', '../main']:
            with self.assertRaises(ValueError): validate(version)
    def test_current_manifests_agree(self):
        version = check()
        self.assertEqual(check(f'v{version}'), version)
        with self.assertRaises(ValueError): check('v99.99.99')

if __name__ == '__main__': unittest.main()
