#!/bin/bash
# Generate self-signed certificates for Redis SSL testing

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CERT_DIR="$SCRIPT_DIR/redis-ssl-certs"

mkdir -p "$CERT_DIR"

# Generate CA key and certificate
openssl genrsa -out "$CERT_DIR/ca.key" 4096
openssl req -x509 -new -nodes -key "$CERT_DIR/ca.key" -sha256 -days 365 -out "$CERT_DIR/ca.crt" -subj "/CN=Redis-Test-CA"

# Generate server key and certificate
openssl genrsa -out "$CERT_DIR/redis.key" 2048
openssl req -new -key "$CERT_DIR/redis.key" -out "$CERT_DIR/redis.csr" -subj "/CN=redis-ssl"

# Create extensions file for server cert
cat > "$CERT_DIR/redis.ext" << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
extendedKeyUsage = serverAuth, clientAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
DNS.2 = redis-ssl
IP.1 = 127.0.0.1
EOF

openssl x509 -req -in "$CERT_DIR/redis.csr" -CA "$CERT_DIR/ca.crt" -CAkey "$CERT_DIR/ca.key" -CAcreateserial -out "$CERT_DIR/redis.crt" -days 365 -sha256 -extfile "$CERT_DIR/redis.ext"

# Set permissions
chmod 644 "$CERT_DIR"/*.crt
chmod 600 "$CERT_DIR"/*.key

echo "Certificates generated in $CERT_DIR"
echo "Files:"
ls -la "$CERT_DIR"