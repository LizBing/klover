"""Regression checks for the harness's trust boundary, without a JVM dependency."""
from contextlib import redirect_stderr, redirect_stdout
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import run as runner


CASE = {"name": "arithmetic/add", "class": "Example", "method": "test",
        "descriptor": "(II)I", "args": ["1", "2"], "requires": ["int_arithmetic"]}


def process(stdout="KLOVER_DIFF_V1\tI\t3\n", error=None, status=0):
    return {"command": ["fake vm"], "exit_status": status, "stdout": stdout,
            "stderr": "", "error": error}


class ProtocolTest(unittest.TestCase):
    def test_exact_integer_and_void(self):
        for kind, value in [("I", "-2147483648"), ("J", "9223372036854775807"), ("V", "void")]:
            record = process(f"KLOVER_DIFF_V1\t{kind}\t{value}\n")
            runner.normalize(record, kind)
            self.assertIsNone(record["error"])
            self.assertEqual(record["result"], {"type": kind, "value": value})

    def test_signed_zero_is_not_equal_and_nan_is_canonical(self):
        records = []
        for value in ["00000000", "80000000", "nan"]:
            record = process(f"KLOVER_DIFF_V1\tF\t{value}\n")
            runner.normalize(record, "F")
            self.assertIsNone(record["error"])
            records.append(record["result"])
        self.assertNotEqual(records[0], records[1])
        self.assertEqual(records[2]["value"], "nan")

    def test_malformed_or_ambiguous_output_is_failure(self):
        for output in ["", "3", "KLOVER_DIFF_V1\tJ\t3\n", "KLOVER_DIFF_V1\tI\t03\n",
                       "KLOVER_DIFF_V1\tI\t2147483648\n", "KLOVER_DIFF_V1\tI\t3\nnoise\n",
                       "KLOVER_DIFF_V1\tI\t3\nKLOVER_DIFF_V1\tI\t3\n"]:
            with self.subTest(output=output):
                record = process(output)
                runner.normalize(record, "I")
                self.assertIsNone(record["result"])
                self.assertIn("invalid result protocol", record["error"])

    def test_nonzero_exit_with_valid_result_is_failure(self):
        record = process(error="process exited with status 1", status=1)
        runner.normalize(record, "I")
        self.assertIsNone(record["result"])

    def test_argument_ranges_bits_and_descriptor(self):
        for kind, value in [("I", "2147483648"), ("B", "128"), ("C", "-1"),
                            ("Z", "true"), ("F", "1.0"), ("D", "nan"), ("I", 3)]:
            with self.subTest(kind=kind, value=value), self.assertRaises(ValueError):
                runner.scalar(kind, value)
        self.assertEqual(runner.scalar("F", "7fc00001"), "7fc00001")
        for desc in ["(V)I", "(I)II", "([I)I", "()Ljava/lang/Object;", None]:
            with self.assertRaises(ValueError):
                runner.signature(desc)


class ExecutionTest(unittest.TestCase):
    def invoke(self, features=None):
        return runner.run_case(CASE, features or {"int_arithmetic": True},
                               "java", Path("classes with spaces"), "klover", 100, 1, 5)

    def test_skip_before_starting_processes(self):
        with patch.object(runner, "execute") as execute:
            record = self.invoke({"int_arithmetic": False})
            execute.assert_not_called()
        self.assertEqual(record["status"], "SKIP")
        self.assertIn("int_arithmetic", record["reason"])

    def test_equal_returns_pass_and_commands_preserve_arguments(self):
        with patch.object(runner, "execute", side_effect=[process(), process()]) as execute:
            record = self.invoke()
        self.assertEqual(record["status"], "PASS")
        self.assertIn("classes with spaces", execute.call_args_list[0].args[0])
        self.assertIn("(II)I", execute.call_args_list[1].args[0])

    def test_mismatch_and_identical_errors_fail(self):
        for responses in [
            [process(), process("KLOVER_DIFF_V1\tI\t4\n")],
            [process(error="unsupported", status=2), process(error="unsupported", status=2)],
            [process(), process(error="process timed out", status=None)],
            [process(""), process("")],
        ]:
            with self.subTest(responses=responses), patch.object(runner, "execute", side_effect=responses):
                self.assertEqual(self.invoke()["status"], "FAIL")

    def test_real_exit_stderr_timeout_and_missing_executable(self):
        record = runner.execute([sys.executable, "-c", "import sys; print('diagnostic', file=sys.stderr); sys.exit(7)"], 5)
        self.assertEqual(record["exit_status"], 7)
        self.assertIn("diagnostic", record["stderr"])
        record = runner.execute([sys.executable, "-c", "import time; time.sleep(10)"], 0.1)
        self.assertIn("timed out", record["error"])
        self.assertIsNone(record["exit_status"])
        with tempfile.TemporaryDirectory() as tmp:
            record = runner.execute([str(Path(tmp) / "missing")], 1)
        self.assertIsNotNone(record["error"])


class RunnerTest(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.report = Path(tmp.name) / "report.json"
        for name, value in [("CASES", [CASE]), ("FEATURES", {"int_arithmetic": True}),
                            ("REPORT", self.report)]:
            patcher = patch.object(runner, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)

    def run_main(self, *args):
        with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            status = runner.main(args)
        return status, json.loads(self.report.read_text())

    def test_unknown_requirement_is_error_not_skip(self):
        runner.FEATURES = {"typo": True}
        status, report = self.run_main()
        self.assertEqual(status, 2)
        self.assertIn("unknown features", report["infrastructure_error"])

    def test_invalid_selection_tool_and_limits_are_errors(self):
        for args in [("--filter", "nothing/*"), ("--java", "/missing/java"),
                     ("--timeout", "nan"), ("--fuel", "0")]:
            status, report = self.run_main(*args)
            self.assertEqual(status, 2)
            self.assertTrue(report["infrastructure_error"])

    def test_all_skipped_needs_no_vm(self):
        runner.FEATURES = {"int_arithmetic": False}
        with patch.object(runner, "execute") as execute:
            status, report = self.run_main("--java", "/missing/java")
            execute.assert_not_called()
        self.assertEqual(status, 0)
        self.assertEqual(report["summary"], {"PASS": 0, "FAIL": 0, "SKIP": 1})

    def test_failure_exit_code_and_reproduction_report(self):
        info = process("OpenJDK 64-Bit Server VM\nversion\nversion\n")
        with patch.object(runner.shutil, "which", return_value=sys.executable), \
             patch.object(runner, "execute", side_effect=[info, process(), process("KLOVER_DIFF_V1\tI\t4\n")]):
            status, report = self.run_main()
        self.assertEqual(status, 1)
        self.assertEqual(report["summary"]["FAIL"], 1)
        self.assertEqual(report["cases"][0]["case"]["descriptor"], "(II)I")
        self.assertEqual(report["cases"][0]["klover"]["result"]["value"], "4")
        self.assertEqual(report["cases"][0]["klover"]["command"], ["fake vm"])

    def test_non_hotspot_reference_is_rejected(self):
        with patch.object(runner.shutil, "which", return_value=sys.executable), \
             patch.object(runner, "execute", return_value=process("Eclipse OpenJ9 VM\n")):
            status, report = self.run_main()
        self.assertEqual(status, 2)
        self.assertIn("HotSpot probe failed", report["infrastructure_error"])


if __name__ == "__main__":
    unittest.main()
