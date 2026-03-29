#!/usr/bin/env python3
"""
Sparkle EdDSA 密钥生成脚本 (Python 版本)
用于生成 Ed25519 密钥对，兼容 Sparkle 更新签名

依赖: pynacl (pip install pynacl)
"""

import base64
import sys

try:
    from nacl.signing import SigningKey
    from nacl.encoding import RawEncoder
except ImportError:
    print("请先安装 pynacl: pip install pynacl")
    sys.exit(1)

def generate_sparkle_keys():
    print("=== Sparkle EdDSA 密钥生成 (Python) ===\n")
    
    # 生成 Ed25519 密钥对
    signing_key = SigningKey.generate()
    verify_key = signing_key.verify_key
    
    # 获取原始字节
    private_key_bytes = bytes(signing_key)
    public_key_bytes = bytes(verify_key)
    
    # Base64 编码 (Sparkle 使用 Base64 编码)
    private_key_b64 = base64.b64encode(private_key_bytes).decode('utf-8')
    public_key_b64 = base64.b64encode(public_key_bytes).decode('utf-8')
    
    print("密钥已生成!\n")
    print("=" * 60)
    print("私钥 (SPARKLE_PRIVATE_KEY)")
    print("=" * 60)
    print(private_key_b64)
    print("=" * 60)
    print("\n请将上述私钥保存到 GitHub Secrets:")
    print("  Repository -> Settings -> Secrets -> Actions -> New repository secret")
    print("  Name: SPARKLE_PRIVATE_KEY")
    print("  Value: 上述 Base64 编码的字符串")
    print()
    print("=" * 60)
    print("公钥 (SPARKLE_PUBLIC_KEY)")
    print("=" * 60)
    print(public_key_b64)
    print("=" * 60)
    print("\n请将上述公钥配置到 Info.plist:")
    print("  <key>SUPublicEDKey</key>")
    print(f"  <string>{public_key_b64}</string>")
    print()
    print("⚠️  安全提示:")
    print("  - 私钥必须保密，只存储在 GitHub Secrets 中")
    print("  - 公钥可以公开，嵌入到应用中")
    print("  - 如果私钥泄露，请立即重新生成")
    print()
    
    # 保存到文件 (可选)
    try:
        with open("sparkle_keys.txt", "w") as f:
            f.write(f"SPARKLE_PRIVATE_KEY={private_key_b64}\n")
            f.write(f"SPARKLE_PUBLIC_KEY={public_key_b64}\n")
        print("密钥已保存到 sparkle_keys.txt (请删除此文件或安全保存)")
    except:
        pass

if __name__ == "__main__":
    generate_sparkle_keys()