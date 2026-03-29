#!/usr/bin/env python3
"""
Sparkle DMG 签名脚本 (Python 版本)
使用 EdDSA 私钥对 DMG 文件进行签名

依赖: pynacl (pip install pynacl)
"""

import base64
import hashlib
import sys
import os

try:
    from nacl.signing import SigningKey
    from nacl.encoding import RawEncoder
except ImportError:
    print("请先安装 pynacl: pip install pynacl")
    sys.exit(1)

def sign_file(file_path: str, private_key_b64: str) -> str:
    """
    使用 EdDSA 私钥签名文件
    
    返回 Base64 编码的签名
    """
    # 解码私钥
    private_key_bytes = base64.b64decode(private_key_b64)
    signing_key = SigningKey(private_key_bytes)
    
    # 计算文件的 SHA256 哈希 (Sparkle 签名文件内容)
    with open(file_path, 'rb') as f:
        file_content = f.read()
    
    # 签名
    signed = signing_key.sign(file_content)
    signature_bytes = signed.signature
    
    # Base64 编码签名
    signature_b64 = base64.b64encode(bytes(signature_bytes)).decode('utf-8')
    
    return signature_b64

def main():
    if len(sys.argv) < 3:
        print("用法: python sign_update_python.py <文件路径> <私钥(Base64)>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    private_key = sys.argv[2]
    
    if not os.path.exists(file_path):
        print(f"错误: 文件不存在: {file_path}")
        sys.exit(1)
    
    # 获取文件大小
    file_size = os.path.getsize(file_path)
    
    # 签名
    signature = sign_file(file_path, private_key)
    
    # 输出 Sparkle 格式的签名信息
    print(f"sparkle:edSignature=\"{signature}\"")
    print(f"length=\"{file_size}\"")
    
    return signature

if __name__ == "__main__":
    main()