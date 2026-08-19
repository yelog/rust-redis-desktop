#!/usr/bin/env python3

import argparse
import base64
import binascii
from pathlib import Path

from nacl.exceptions import BadSignatureError
from nacl.signing import VerifyKey


def decode_base64(value: str, name: str, expected_length: int) -> bytes:
    try:
        decoded = base64.b64decode(value, validate=True)
    except (binascii.Error, ValueError) as error:
        raise ValueError(f"{name} is not valid Base64: {error}") from error

    if len(decoded) != expected_length:
        raise ValueError(
            f"{name} must decode to {expected_length} bytes, got {len(decoded)}"
        )
    return decoded


def verify_update(
    public_key: str, update_path: Path, signature: str, declared_length: int
) -> None:
    actual_length = update_path.stat().st_size
    if actual_length != declared_length:
        raise ValueError(
            f"File length mismatch for {update_path}: "
            f"declared {declared_length}, actual {actual_length}"
        )

    public_key_bytes = decode_base64(public_key, "Public key", 32)
    signature_bytes = decode_base64(signature, "Signature", 64)

    try:
        VerifyKey(public_key_bytes).verify(update_path.read_bytes(), signature_bytes)
    except BadSignatureError as error:
        raise ValueError(f"Invalid signature for {update_path}") from error


def main() -> int:
    parser = argparse.ArgumentParser(description="Verify a Sparkle Ed25519 update")
    parser.add_argument("public_key", help="Base64-encoded Ed25519 public key")
    parser.add_argument("update_path", type=Path)
    parser.add_argument("signature", help="Base64-encoded Ed25519 signature")
    parser.add_argument("declared_length", type=int)
    args = parser.parse_args()

    try:
        verify_update(
            args.public_key, args.update_path, args.signature, args.declared_length
        )
    except (OSError, ValueError) as error:
        parser.exit(1, f"Sparkle verification failed: {error}\n")

    print(f"Sparkle signature verified: {args.update_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
