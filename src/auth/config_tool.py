import argparse
import sys

import nacl.secret
import nacl.signing
import nacl.utils
from nacl.encoding import RawEncoder, URLSafeBase64Encoder


def build_parser():
    args = argparse.ArgumentParser()
    args.add_argument('--config-key-file',
            help='path to symmetric key for encrypting config values (default: stdin')

    args.add_argument('command',
            choices=(
                'gen-config-key',
                'gen-signing-key',
                'gen-flask-secret-key',
                'encrypt-db-password',
                ))

    return args

def load_config_key(args):
    if args.config_key_file is None:
        x = input('config key: ')
    else:
        with open(args.config_key_file) as f:
            x = f.read()

    key = URLSafeBase64Encoder.decode(x.encode('ascii'))
    box = nacl.secret.SecretBox(key)
    return box

def nonce():
    return nacl.utils.random(nacl.secret.SecretBox.NONCE_SIZE)

def encode64(b):
    return URLSafeBase64Encoder.encode(b).decode('ascii')

def main(args):
    args = build_parser().parse_args(args)

    if args.command == 'gen-config-key':
        key = nacl.utils.random(nacl.secret.SecretBox.KEY_SIZE)
        print(encode64(key))
    elif args.command == 'gen-signing-key':
        config_box = load_config_key(args)

        # Generate and encrypt the signing key
        signing_key = nacl.signing.SigningKey.generate()
        signing_key_bin = signing_key.encode(RawEncoder)
        signing_key_enc = config_box.encrypt(signing_key_bin, nonce())
        print(encode64(signing_key_enc))
    elif args.command == 'gen-flask-secret-key':
        config_box = load_config_key(args)

        secret = nacl.utils.random(32)
        secret_enc = config_box.encrypt(secret, nonce())
        print(encode64(secret_enc))
    elif args.command == 'encrypt-db-password':
        config_box = load_config_key(args)

        password = input('database password: ')
        password_enc = config_box.encrypt(password.encode('utf-8'), nonce())
        print(encode64(password_enc))

if __name__ == '__main__':
    main(sys.argv[1:])
