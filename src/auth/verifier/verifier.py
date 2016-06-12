import configparser
import os
import struct
import sys

import nacl.bindings
import nacl.signing
import nacl.utils
from nacl.encoding import RawEncoder, URLSafeBase64Encoder
import requests


OP_AUTH_RESPONSE =      0x0014
OP_AUTH_CHALLENGE =     0x8021
OP_AUTH_RESULT =        0x8022
OP_ADD_CLIENT =         0xff00
OP_REMOVE_CLIENT =      0xff01
OP_CLIENT_REMOVED =     0xff02
OP_AUTH_DONE =          0xff09

MODE_SSO =      0
MODE_LOCAL =    1


def read_config():
    config = configparser.ConfigParser(allow_no_value=True)
    config.optionxform = str
    config.read_file(open('outpost.ini'))

    return {
            'auth_server': config['auth'].get('sso_endpoint'),
            #'superusers': set(config['auth.superusers'].keys()),
            #'bans': set(config['auth.bans'].keys()),
            }

def get_verify_key(url):
    r = requests.get(url)
    key_str = r.json()['key']
    key = nacl.signing.VerifyKey(key_str.encode('ascii'), URLSafeBase64Encoder)
    return key


def build_raw(cid, opcode, body):
    data = struct.pack('<H', opcode) + body
    header = struct.pack('<HH', cid, len(data))
    return header + data

def build_msg(cid, opcode, fmt, *args):
    return build_raw(cid, opcode, struct.pack(fmt, *args))

def check_response(key, data, expected_nonce):
    try:
        key.verify(data)
    except nacl.signing.BadSignatureError:
        print('bad sig', file=sys.stderr)
        return None

    body = data[nacl.bindings.crypto_sign_BYTES:]

    nonce_len, name_len, _, _, uid = struct.unpack('<BBBBI', body[:8])
    nonce_start = 8
    name_start = nonce_start + nonce_len
    nonce = body[nonce_start : nonce_start + nonce_len]
    name = body[name_start : name_start + name_len]

    if nonce != expected_nonce:
        print('bad nonce', file=sys.stderr)
        return None

    print('response ok', file=sys.stderr)
    return uid, name.decode()

def main():
    cfg = read_config()
    if not cfg['auth_server'].startswith('https://'):
        sys.stderr.write('warning: Auth server URL does not use HTTPS!  '
                'This configuration is not secure.\n')
    key = get_verify_key(cfg['auth_server'] + 'get_verify_key')

    b_in = sys.stdin.buffer
    b_out = sys.stdout.buffer

    pending_nonces = {}

    while True:
        cid, data_len, opcode = struct.unpack('<HHH', b_in.read(6))
        # data_len includes the length of `opcode`
        data = b_in.read(data_len - 2)

        try:
            if cid == 0:
                user_cid, = struct.unpack('<H', data[:2])
                data = data[2:]
                print('control message for %d' % user_cid, file=sys.stderr)
                if opcode == OP_ADD_CLIENT:
                    nonce = nacl.utils.random(16)
                    pending_nonces[user_cid] = nonce
                    challenge = struct.pack('<H', MODE_SSO) + nonce
                    b_out.write(build_raw(user_cid, OP_AUTH_CHALLENGE, challenge))
                    print('ADD client %d, mode %d, nonce %s' %
                            (user_cid, MODE_SSO, ''.join('%02x' % b for b in nonce)),
                            file=sys.stderr)
                elif opcode == OP_REMOVE_CLIENT:
                    del pending_nonces[user_cid]
                    b_out.write(build_msg(0, OP_CLIENT_REMOVED, '<H', user_cid))
                    print('REMOVE client %d' % user_cid, file=sys.stderr)
                else:
                    assert False, 'bad opcode: %x' % opcode
            else:
                if opcode == OP_AUTH_RESPONSE:
                    uid, name = check_response(key, data, pending_nonces.pop(cid))
                    if name is None:
                        msg = struct.pack('<H', 0) + 'Login failed'.encode()
                        b_out.write(build_raw(cid, OP_AUTH_RESULT, msg))
                        b_out.write(build_msg(0, OP_CLIENT_REMOVED, '<H', cid))
                        print('FAILED client %d' % cid, file=sys.stderr)
                    else:
                        msg = struct.pack('<H', 1) + name.encode()
                        b_out.write(build_raw(cid, OP_AUTH_RESULT, msg))
                        name_b = name.encode()
                        msg = struct.pack('<HIH', cid, uid, len(name_b)) + name_b
                        b_out.write(build_raw(0, OP_AUTH_DONE, msg))
                        print('DONE client %d (name = %r)' % (cid, name),
                                file=sys.stderr)
                else:
                    assert False, 'bad opcode: %x' % opcode
        except:
            sys.stderr.write('Exception while handling %x from %d' % (opcode, cid))
            traceback.print_exc()

        b_out.flush()
        sys.stderr.flush()






if __name__ == '__main__':
    main()
