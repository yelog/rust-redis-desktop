import base64
import tempfile
import unittest
from pathlib import Path

from nacl.signing import SigningKey

from scripts.verify_sparkle_signature import verify_update


class VerifySparkleSignatureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp_dir.cleanup)
        self.update_path = Path(self.temp_dir.name) / "update.dmg"
        self.update_path.write_bytes(b"signed update contents")

        self.signing_key = SigningKey.generate()
        self.public_key = base64.b64encode(
            bytes(self.signing_key.verify_key)
        ).decode("ascii")
        self.signature = base64.b64encode(
            self.signing_key.sign(self.update_path.read_bytes()).signature
        ).decode("ascii")

    def test_accepts_valid_signature(self) -> None:
        verify_update(
            self.public_key,
            self.update_path,
            self.signature,
            self.update_path.stat().st_size,
        )

    def test_rejects_signature_without_base64_padding(self) -> None:
        with self.assertRaisesRegex(ValueError, "not valid Base64"):
            verify_update(
                self.public_key,
                self.update_path,
                self.signature.rstrip("="),
                self.update_path.stat().st_size,
            )

    def test_rejects_declared_length_mismatch(self) -> None:
        with self.assertRaisesRegex(ValueError, "File length mismatch"):
            verify_update(
                self.public_key,
                self.update_path,
                self.signature,
                self.update_path.stat().st_size + 1,
            )

    def test_rejects_tampered_update(self) -> None:
        self.update_path.write_bytes(b"tampered update contents")

        with self.assertRaisesRegex(ValueError, "Invalid signature"):
            verify_update(
                self.public_key,
                self.update_path,
                self.signature,
                self.update_path.stat().st_size,
            )


if __name__ == "__main__":
    unittest.main()
