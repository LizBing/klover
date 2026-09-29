"""Fixture lifecycle and Make dependencies, using disposable projects."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class BuildToolsTest(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory(prefix="klover build test ")
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        (self.root / "scripts").mkdir()
        shutil.copy2(ROOT / "scripts/build-test-classes.py", self.root / "scripts")
        shutil.copy2(ROOT / "Makefile", self.root)
        for name, source in {
            "test_data/classes/Example.java": "public class Example { class Nested {} }",
            "test_data/classes/Extra.java": "public class Extra {}",
            "java/java.base/java/lang/Object.java": "package java.lang; public class Object {}",
            "tests/differential/HotSpotRunner.java": "public class HotSpotRunner {}",
        }.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source + "\n")
        self.output = self.root / "build/test-classes"

    def compile(self):
        result = subprocess.run([sys.executable, "scripts/build-test-classes.py"],
                                cwd=self.root, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    @unittest.skipUnless(shutil.which("javac"), "requires javac with --release 8")
    def test_recompile_removes_deleted_and_nested_classes(self):
        self.compile()
        self.assertTrue((self.output / "Example$Nested.class").exists())
        (self.root / "test_data/classes/Example.java").write_text("public class Example {}\n")
        (self.root / "test_data/classes/Extra.java").unlink()
        self.compile()
        self.assertTrue((self.output / "Example.class").exists())
        self.assertTrue((self.output / "HotSpotRunner.class").exists())
        self.assertFalse((self.output / "Example$Nested.class").exists())
        self.assertFalse((self.output / "Extra.class").exists())

    @unittest.skipUnless(shutil.which("javac"), "requires javac with --release 8")
    def test_compilation_failure_removes_output_and_stops_dependent_tests(self):
        # A marker command would be called if Cargo ran after a failed compile.
        marker = self.root / "cargo-called"
        probe = self.root / "probe.sh"
        probe.write_text(f'#!/bin/sh\ntouch "{marker}"\n')
        probe.chmod(0o755)
        for source in ["test_data/classes/Example.java", "tests/differential/HotSpotRunner.java"]:
            with self.subTest(source=source):
                path = self.root / source
                original = path.read_text()
                self.compile()
                path.write_text("invalid java source\n")
                result = subprocess.run(
                    ["make", "test-rust", "CMAKE=true", f"CARGO={probe}", f"PYTHON={sys.executable}"],
                    cwd=self.root, capture_output=True, text=True,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("Java compilation failed", result.stderr)
                self.assertFalse(self.output.exists())
                self.assertFalse(marker.exists())
                path.write_text(original)

    def test_make_shares_compilation_and_separates_builds_from_tests(self):
        def dry_run(*targets):
            result = subprocess.run(["make", "-n", "-j4", *targets], cwd=self.root,
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            return result.stdout
        self.assertEqual(dry_run("test-all").count("build-test-classes.py"), 1)
        self.assertNotIn("build-test-classes.py", dry_run("test-core"))
        self.assertNotIn("build-test-classes.py", dry_run("build-rust"))
        for target in [(), ("build-all",)]:
            output = dry_run(*target)
            self.assertIn("--example differential_invoke", output)
            self.assertNotIn("unittest", output)
            self.assertNotIn("ctest", output)
            self.assertNotIn("cargo test", output)


if __name__ == "__main__":
    unittest.main()
