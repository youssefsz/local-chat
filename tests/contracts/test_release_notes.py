import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools'))
import unittest
from release_notes import previous_release, contributors, render


class ReleaseNotes(unittest.TestCase):
    def test_only_older_published_ancestors_define_the_range(self):
        records = [{'tag_name': tag, 'draft': draft} for tag, draft in [
            ('v0.1.0-beta.5', False), ('v0.1.0-beta.6', False),
            ('v0.1.0-beta.99', False), ('v0.1.0-beta.4', True), ('invalid', False)]]
        self.assertEqual(previous_release(records, '0.1.0-beta.7', 'head', lambda tag, _: tag != 'v0.1.0-beta.6'), 'v0.1.0-beta.5')
        self.assertIsNone(previous_release([], '0.1.0', 'head', lambda *_: True))

    def test_attribution_uses_verified_authors_without_publishing_emails(self):
        commits = [{'author': {'login': 'contributor', 'type': 'User'}, 'commit': {
            'author': {'name': 'Ignored email guess'},
            'message': 'Fix\n\nCo-authored-by: Second Author <private@example.invalid>'}},
            {'author': None, 'commit': {'author': {'name': 'Unlinked Author'}, 'message': 'Fix'}},
            {'author': {'login': 'automation[bot]', 'type': 'Bot'}, 'commit': {'author': {'name': 'Bot'}, 'message': 'Update'}}]
        self.assertEqual(contributors(commits), ['@contributor', 'Second Author', 'Unlinked Author'])

    def test_downloads_and_comparison_are_bound_to_the_release(self):
        note = render('0.1.0-beta.7', 'Preserves drafts.', '## Fixes\n- A real PR by @contributor in #7', ['@contributor'], 'v0.1.0-beta.6')
        self.assertIn('/v0.1.0-beta.7/Local-Chat_0.1.0-beta.7_windows-x64.exe', note)
        self.assertIn('/compare/v0.1.0-beta.6...v0.1.0-beta.7', note)
        self.assertIn('## Contributors', note)
        self.assertIn('## Fixes', note)
        with self.assertRaises(ValueError): render('../main', 'Text', '', [], None)
        with self.assertRaises(ValueError): render('0.1.0', '', '', [], None)


if __name__ == '__main__':
    unittest.main()
