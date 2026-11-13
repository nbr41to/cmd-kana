"""Release metadata mismatches must fail before signing or uploading artifacts."""

import importlib.util
import json
from pathlib import Path
import plistlib
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location("release_metadata", ROOT / "scripts/check-release.py")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class ReleaseMetadataTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        for name in ("Cargo.toml", "Cargo.lock", "CHANGELOG.md", "app/Info.plist", ".github/release.json"):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, path)
        shutil.copytree(ROOT / "docs/releases", self.root / "docs/releases")
        self.version = plistlib.loads((ROOT / "app/Info.plist").read_bytes())["CFBundleShortVersionString"]

    def test_current_release_metadata_is_consistent(self):
        self.assertEqual(MODULE.validate(self.root), self.version)

    def test_mismatched_bundle_version_is_rejected(self):
        path = self.root / "app/Info.plist"
        info = plistlib.loads(path.read_bytes())
        info["CFBundleShortVersionString"] = "999.999.999"
        path.write_bytes(plistlib.dumps(info))
        with self.assertRaisesRegex(ValueError, "versions differ"):
            MODULE.validate(self.root)

    def test_stale_lockfile_is_rejected(self):
        path = self.root / "Cargo.lock"
        path.write_text(path.read_text().replace(f'version = "{self.version}"', 'version = "999.999.999"'))
        with self.assertRaisesRegex(ValueError, "Cargo.lock"):
            MODULE.validate(self.root)

    def test_missing_release_notes_is_rejected(self):
        (self.root / "docs/releases" / f"v{self.version}.md").unlink()
        with self.assertRaisesRegex(ValueError, "Release notes"):
            MODULE.validate(self.root)

    def test_different_publication_version_is_rejected(self):
        path = self.root / ".github/release.json"
        plan = json.loads(path.read_text())
        plan["version"] = "999.999.999"
        path.write_text(json.dumps(plan))
        with self.assertRaisesRegex(ValueError, "publication plan version"):
            MODULE.validate(self.root)

    def test_string_publish_flag_cannot_enable_publication(self):
        path = self.root / ".github/release.json"
        plan = json.loads(path.read_text())
        plan["publish"] = "true"
        path.write_text(json.dumps(plan))
        with self.assertRaisesRegex(ValueError, "boolean publish flag"):
            MODULE.validate(self.root)


if __name__ == "__main__":
    unittest.main()
