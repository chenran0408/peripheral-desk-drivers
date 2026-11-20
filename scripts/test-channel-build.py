"""Verify that a new preview cannot be packaged as the prior stable version."""
import unittest

from build import select_source_versions


class ChannelVersionTest(unittest.TestCase):
    def test_workspace_check_uses_exact_source_version_and_channel(self):
        stable = {"id": "mouse", "version": "0.4.2", "channel": "stable"}
        preview = {"id": "mouse", "version": "0.4.13-preview.46", "channel": "preview"}
        other = {"id": "keyboard", "version": "0.4.1", "channel": "stable"}
        selected = select_source_versions(
            [stable, other], {"mouse": preview["version"], "keyboard": other["version"]}, [preview]
        )
        self.assertEqual(selected, [preview, other])

    def test_release_rejects_metadata_from_another_source_revision(self):
        previous = {"id": "mouse", "version": "0.4.2", "channel": "stable"}
        with self.assertRaisesRegex(ValueError, "catalog version 0.4.2 does not match crate"):
            select_source_versions([previous], {"mouse": "0.4.13-preview.46"})


if __name__ == "__main__":
    unittest.main()
