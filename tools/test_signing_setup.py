"""Prove a rejected Apple login cannot replace the existing GitHub secrets."""
import contextlib
import io
from pathlib import Path
import runpy
import subprocess
import unittest
from unittest.mock import patch


class SigningSetup(unittest.TestCase):
    def exercise(self, accepted):
        calls = []

        def run(command, **options):
            calls.append((command, options))
            if command[0] == 'xcrun':
                return subprocess.CompletedProcess(command, 0 if accepted else 1, b'{}', b'' if accepted else b'HTTP 401')
            return subprocess.CompletedProcess(command, 0, b'', b'')

        output = io.StringIO()
        with patch('sys.argv', ['configure_signing.py', '--notarization-only']), patch('builtins.input', return_value='fixture@example.invalid'), patch('getpass.getpass', return_value='fixture-password'), patch('subprocess.run', side_effect=run), contextlib.redirect_stdout(output):
            if accepted:
                runpy.run_path(str(Path(__file__).with_name('configure_signing.py')), run_name='__main__')
            else:
                with self.assertRaisesRegex(SystemExit, 'No GitHub secrets were changed'):
                    runpy.run_path(str(Path(__file__).with_name('configure_signing.py')), run_name='__main__')
        self.assertNotIn('fixture-password', output.getvalue())
        return calls

    def test_rejected_login_preserves_secrets(self):
        calls = self.exercise(False)
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][0][:3], ['xcrun', 'notarytool', 'history'])

    def test_verified_login_updates_only_authentication(self):
        calls = self.exercise(True)
        writes = calls[1:]
        self.assertEqual([command[3] for command, _ in writes], ['APPLE_ID', 'APPLE_PASSWORD'])
        for command, options in writes:
            self.assertNotIn('fixture-password', command)
            self.assertIn('input', options)


if __name__ == '__main__':
    unittest.main()
