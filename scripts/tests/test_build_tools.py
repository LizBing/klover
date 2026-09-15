"""Build-tool regression tests using disposable projects and a real javac."""

from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class BuildToolsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="klover build test ")
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        (self.root / "scripts").mkdir()
        for name in ("build-test-classes.py", "verify-class-major.py", "clean-build.py"):
            shutil.copy2(ROOT / "scripts" / name, self.root / "scripts" / name)
        self.sources = self.root / "test_data/classes"
        self.sources.mkdir(parents=True)
        object_file = self.root / "java/java.base/java/lang/Object.java"
        object_file.parent.mkdir(parents=True)
        object_file.write_text("package java.lang; public class Object {}\n")
        (self.sources / "Example.java").write_text("public class Example { class Nested {} }\n")
        self.output = self.root / "build/test classes"

    def run_tool(self, name, *args, ok=True):
        result = subprocess.run(
            [sys.executable, str(self.root / "scripts" / name), *map(str, args)],
            capture_output=True, text=True,
        )
        if ok:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    @unittest.skipUnless(shutil.which("javac"), "requires javac with --release 8")
    def test_fixture_lifecycle(self):
        def build(ok=True):
            return self.run_tool("build-test-classes.py", "--output", self.output, ok=ok)

        build()
        stamp = self.output / ".build-state.json"
        before = stamp.stat().st_mtime_ns
        self.assertTrue((self.output / "Example$Nested.class").exists())
        build()
        self.assertEqual(stamp.stat().st_mtime_ns, before, "no-op build rewrote outputs")

        # Source edits remove obsolete nested outputs; source additions/deletions
        # must be detected even when no surviving source timestamp changes.
        (self.sources / "Example.java").write_text("public class Example {}\n")
        extra = self.sources / "Extra.java"
        extra.write_text("public class Extra {}\n")
        build()
        self.assertFalse((self.output / "Example$Nested.class").exists())
        self.assertTrue((self.output / "Extra.class").exists())
        extra.unlink()
        build()
        self.assertFalse((self.output / "Extra.class").exists())

        saved = (self.output / "Example.class").read_bytes()
        (self.output / "Example.class").write_bytes(b"corrupt")
        build()
        self.assertEqual((self.output / "Example.class").read_bytes(), saved)
        (self.output / "Example.class").unlink()
        build()
        self.assertEqual((self.output / "Example.class").read_bytes(), saved)

        (self.sources / "Example.java").write_text("invalid java source\n")
        build(ok=False)
        self.assertEqual((self.output / "Example.class").read_bytes(), saved)

    def test_make_preserves_build_directory_spaces(self):
        result = subprocess.run(
            ["make", "-f", str(ROOT / "Makefile"), "-n", "classes",
             f"BUILD_DIR={self.root / 'custom build'}"],
            capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(f'--output "{self.root / "custom build/test-classes"}"', result.stdout)

    def test_verify_rejects_empty_or_wrong_version(self):
        self.output.mkdir(parents=True)
        self.run_tool("verify-class-major.py", 52, self.output, ok=False)
        (self.output / "Bad.class").write_bytes(bytes.fromhex("cafebabe00000035"))
        self.run_tool("verify-class-major.py", 52, self.output, ok=False)

    def test_clean_validates_all_roots_before_removing_any(self):
        self.output.mkdir(parents=True)
        self.run_tool("clean-build.py", self.root / "build", self.sources, ok=False)
        self.assertTrue(self.output.exists())
        link = self.root / "compile_commands.json"
        link.symlink_to(self.output / "compile_commands.json")
        self.run_tool("clean-build.py", self.root / "build", self.root / "cargo-output")
        self.assertFalse(self.output.exists())
        self.assertFalse(link.is_symlink())
        self.assertTrue((self.sources / "Example.java").exists())


if __name__ == "__main__":
    unittest.main()
