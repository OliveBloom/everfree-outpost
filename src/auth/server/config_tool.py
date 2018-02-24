import argparse
import sys

import nacl.secret
import nacl.signing
import nacl.utils
from nacl.encoding import RawEncoder, URLSafeBase64Encoder


def build_parser():
    args = argparse.ArgumentParser()
    args.add_argument('--config-key-file',
            help='path to symmetric key for encrypting config values (default: stdin)')
    args.add_argument('--dont-encrypt', action='store_true',
            help='use plain base64 instead of encrypting secret values (not recommended)')

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

def encode_bytes(bs, args):
    '''Convert `bytes` to `str` using the settings provided in `args`.'''
    if args.dont_encrypt:
        return 'b64!' + encode64(bs)
    else:
        config_box = load_config_key(args)
        enc = config_box.encrypt(bs, nonce())
        return 'enc!' + encode64(enc)

def main(args):
    args = build_parser().parse_args(args)

    if args.command == 'gen-config-key':
        key = nacl.utils.random(nacl.secret.SecretBox.KEY_SIZE)
        # Doesn't make sense to encrypt this one
        print(encode64(key))
    elif args.command == 'gen-signing-key':
        signing_key = nacl.signing.SigningKey.generate()
        print(encode_bytes(signing_key.encode(RawEncoder), args))
    elif args.command == 'gen-flask-secret-key':
        secret = nacl.utils.random(32)
        print(encode_bytes(secret, args))
    elif args.command == 'encrypt-db-password':
        config_box = load_config_key(args)

        password = input('database password: ')
        print(encode_bytes(password.encode('utf-8'), args))

if __name__ == '__main__':
    main(sys.argv[1:])
