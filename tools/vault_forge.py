#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# ════════════════════════════════════════════════════════════════════════════
#  Realix · Vault Forge
#
#  Этот файл носит две шляпы сразу:
#    1. Эталон криптосистемы на Python. ASM-порт (source/security/*.asm)
#       обязан повторять его поведение байт в байт.
#    2. Инструмент сборки: пароль + данные → VAULT.BIN, который ОС читает
#       с дискеты и расшифровывает после ввода пароля.
#
#  Криптостек (только стандартные примитивы, ничего самодельного):
#    master       = PBKDF2-HMAC-SHA256(password, salt, iters, dkLen=64)
#    enc_key      = master[0:32]      mac_key = master[32:64]
#    keystream(i) = HMAC-SHA256(enc_key, nonce(12) || BE32(i))      (PRF-CTR)
#    ciphertext   = plaintext XOR keystream
#    tag          = HMAC-SHA256(mac_key, nonce(12) || ciphertext)   (Encrypt-then-MAC)
#
#  Формат VAULT.BIN (little-endian):
#    0   magic        4    b'RVLT'
#    4   version      1    = 1
#    5   reserved     3    = 0
#    8   iters        4    число итераций PBKDF2
#    12  salt         16
#    28  nonce        12
#    40  payload_len  4    длина открытого текста
#    44  tag          32   HMAC-SHA256(mac_key, nonce || ciphertext)
#    76  ciphertext   payload_len
#
#  Codded by Nixort <3
# ════════════════════════════════════════════════════════════════════════════

import argparse
import hashlib
import hmac
import os
import struct
import sys

MAGIC = b"RVLT"
VERSION = 1
HEADER_LEN = 76
DEFAULT_ITERS = 200_000


# ── Криптоядро · эталон, который повторяет ASM-порт ──────────────────────────

def _hmac_sha256(key: bytes, msg: bytes) -> bytes:
    return hmac.new(key, msg, hashlib.sha256).digest()


def pbkdf2(password: bytes, salt: bytes, iters: int, dklen: int) -> bytes:
    return hashlib.pbkdf2_hmac("sha256", password, salt, iters, dklen)


def keystream(enc_key: bytes, nonce: bytes, nbytes: int) -> bytes:
    out = bytearray()
    counter = 0
    while len(out) < nbytes:
        block = _hmac_sha256(enc_key, nonce + struct.pack(">I", counter))
        out.extend(block)
        counter += 1
    return bytes(out[:nbytes])


def seal(password: bytes, plaintext: bytes, salt: bytes, nonce: bytes,
         iters: int) -> bytes:
    master = pbkdf2(password, salt, iters, 64)
    enc_key, mac_key = master[:32], master[32:]
    ciphertext = bytes(p ^ k for p, k in
                       zip(plaintext, keystream(enc_key, nonce, len(plaintext))))
    tag = _hmac_sha256(mac_key, nonce + ciphertext)
    return ciphertext, tag


def open_vault(password: bytes, blob: bytes) -> bytes:
    if blob[:4] != MAGIC:
        raise ValueError("bad magic")
    iters = struct.unpack("<I", blob[8:12])[0]
    salt = blob[12:28]
    nonce = blob[28:40]
    plen = struct.unpack("<I", blob[40:44])[0]
    tag = blob[44:76]
    ciphertext = blob[76:76 + plen]

    master = pbkdf2(password, salt, iters, 64)
    enc_key, mac_key = master[:32], master[32:]
    expect = _hmac_sha256(mac_key, nonce + ciphertext)
    if not hmac.compare_digest(expect, tag):
        raise ValueError("AUTH FAIL: wrong password or tampered vault")
    return bytes(c ^ k for c, k in
                 zip(ciphertext, keystream(enc_key, nonce, plen)))


# ── Сборка контейнера VAULT.BIN ─────────────────────────────────────────

def forge(password: bytes, plaintext: bytes, iters: int) -> bytes:
    salt = os.urandom(16)
    nonce = os.urandom(12)
    ciphertext, tag = seal(password, plaintext, salt, nonce, iters)
    header = (MAGIC + bytes([VERSION]) + b"\x00\x00\x00"
              + struct.pack("<I", iters) + salt + nonce
              + struct.pack("<I", len(plaintext)) + tag)
    assert len(header) == HEADER_LEN, len(header)
    return header + ciphertext


# ── Самопроверка · своя SHA-256 против hashlib и контрольные векторы ─────────

def _pure_sha256(data: bytes) -> bytes:
    """Чистая реализация SHA-256: доказывает, что раундовая логика,
    которую копирует ASM, совпадает с эталоном."""
    h = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
         0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]
    k = [
        0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
        0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
        0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
        0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
        0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
        0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
        0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
        0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2]
    m = 0xffffffff
    def rotr(x, n): return ((x >> n) | (x << (32 - n))) & m
    msg = bytearray(data)
    ml = (8 * len(data)) & ((1 << 64) - 1)
    msg.append(0x80)
    while len(msg) % 64 != 56:
        msg.append(0)
    msg += struct.pack(">Q", ml)
    for off in range(0, len(msg), 64):
        w = list(struct.unpack(">16I", msg[off:off + 64])) + [0] * 48
        for i in range(16, 64):
            s0 = rotr(w[i-15], 7) ^ rotr(w[i-15], 18) ^ (w[i-15] >> 3)
            s1 = rotr(w[i-2], 17) ^ rotr(w[i-2], 19) ^ (w[i-2] >> 10)
            w[i] = (w[i-16] + s0 + w[i-7] + s1) & m
        a, b, c, d, e, f, g, hh = h
        for i in range(64):
            S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25)
            ch = (e & f) ^ (~e & g)
            t1 = (hh + S1 + ch + k[i] + w[i]) & m
            S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22)
            maj = (a & b) ^ (a & c) ^ (b & c)
            t2 = (S0 + maj) & m
            hh, g, f, e, d, c, b, a = g, f, e, (d + t1) & m, c, b, a, (t1 + t2) & m
        h = [(x + y) & m for x, y in zip(h, [a, b, c, d, e, f, g, hh])]
    return b"".join(struct.pack(">I", x) for x in h)


def selftest() -> None:
    # SHA-256 (NIST)
    assert hashlib.sha256(b"abc").hexdigest() == (
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
    assert _pure_sha256(b"abc") == hashlib.sha256(b"abc").digest()
    assert _pure_sha256(b"") == hashlib.sha256(b"").digest()
    big = b"The quick brown fox jumps over the lazy dog" * 7
    assert _pure_sha256(big) == hashlib.sha256(big).digest()
    # PBKDF2-HMAC-SHA256 (RFC 7914 / 6070-style)
    assert pbkdf2(b"passwd", b"salt", 1, 64).hex().startswith("55ac046e56e3089fec1691c22544b605")
    # round-trip
    pw = b"correct horse battery staple"
    pt = b"TOP SECRET: launch code 0xDEADBEEF\n"
    blob = forge(pw, pt, 4096)
    assert open_vault(pw, blob) == pt
    try:
        open_vault(b"wrong", blob)
        raise AssertionError("must reject wrong password")
    except ValueError:
        pass
    # tamper ciphertext -> MAC must reject
    bad = bytearray(blob); bad[-1] ^= 1
    try:
        open_vault(pw, bytes(bad)); raise AssertionError("must reject tamper")
    except ValueError:
        pass
    print("selftest OK: SHA-256, PBKDF2, seal/open, wrong-pw, tamper")


def main() -> int:
    ap = argparse.ArgumentParser(description="Realix Vault forge")
    sub = ap.add_subparsers(dest="cmd", required=True)

    f = sub.add_parser("forge", help="create VAULT.BIN")
    f.add_argument("--password", required=True)
    f.add_argument("--iters", type=int, default=DEFAULT_ITERS)
    g = f.add_mutually_exclusive_group(required=True)
    g.add_argument("--text", help="protected payload as text")
    g.add_argument("--in", dest="infile", help="protected payload from file")
    f.add_argument("--out", required=True)

    o = sub.add_parser("open", help="decrypt VAULT.BIN (verify)")
    o.add_argument("--password", required=True)
    o.add_argument("--in", dest="infile", required=True)

    sub.add_parser("selftest", help="run crypto self-test")

    args = ap.parse_args()
    if args.cmd == "selftest":
        selftest(); return 0
    if args.cmd == "forge":
        pt = args.text.encode() if args.text is not None else open(args.infile, "rb").read()
        blob = forge(args.password.encode(), pt, args.iters)
        with open(args.out, "wb") as fh:
            fh.write(blob)
        print(f"wrote {args.out} ({len(blob)} bytes, iters={args.iters}, payload={len(pt)})")
        return 0
    if args.cmd == "open":
        blob = open(args.infile, "rb").read()
        sys.stdout.buffer.write(open_vault(args.password.encode(), blob))
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
