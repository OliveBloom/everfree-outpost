import configparser
import functools
import inspect
import json
import os
import re
import struct

from flask import Flask, request, jsonify, render_template, \
        flash, redirect, url_for, abort, session, make_response

import bcrypt
import nacl.secret
import nacl.signing
from nacl.encoding import RawEncoder, URLSafeBase64Encoder


def get_config_key():
    key = os.environ.get('OUTPOST_AUTH_CONFIG_KEY') or input('config key: ')
    box = nacl.secret.SecretBox(URLSafeBase64Encoder.decode(key.encode('ascii')))
    return box

def read_config():
    config = configparser.ConfigParser()
    config.read_file(open('outpost_auth.ini'))

    box = get_config_key()

    def get(k):
        return config['DEFAULT'].get(k)

    def get_default(k, default):
        return config['DEFAULT'].get(k, default)

    def decrypt(k):
        value = get(k)
        return box.decrypt(URLSafeBase64Encoder.decode(value.encode('ascii')))


    reserved_names_path = get_default('reserved_names', None)
    if reserved_names_path is not None:
        with open(reserved_names_path) as f:
            reserved_names = json.load(f)
    else:
        reserved_names = {}


    return {
            'flask_debug': int(get_default('flask_debug', 0)),
            'flask_secret_key': decrypt('flask_secret_key'),

            'signing_key': nacl.signing.SigningKey(decrypt('signing_key')),

            'db_type': get_default('db_type', 'postgres'),
            'db_name': get('db_name'),
            'db_user': get('db_user'),
            'db_pass': decrypt('db_pass').decode('utf-8'),
            'db_host': get_default('db_host', None),
            'db_connstr': get_default('db_connstr', None),

            'allowed_origin': get('allowed_origin'),
            'redir_url': get_default('redir_url', None),

            'reserved_names': reserved_names,
            }

cfg = read_config()

if cfg['db_type'] == 'postgres':
    from db_postgres import Database
elif cfg['db_type'] == 'mysql':
    from db_mysql import Database
else:
    raise ValueError('db_type must be "postgres" or "mysql" (got %r)' % cfg['db_type'])
db = Database(cfg)

app = Flask(__name__)
app.debug = bool(cfg['flask_debug'])
app.secret_key = cfg['flask_secret_key']


# Misc. helper functions

def unpack_args(f):
    sig = inspect.signature(f)
    params = sig.parameters

    @functools.wraps(f)
    def g(args):
        dct = {}
        for k,v in args.items():
            if k in dct:
                return bug('Duplicate argument: %r' % k)
            dct[k] = v
        args = dct

        for param in params.values():
            if param.name not in args and param.default is inspect.Parameter.empty:
                return bug('Missing argument: %r' % param.name)
        for k in args:
            if k not in params:
                return bug('Extra argument: %r' % k)

        return f(**args)
    return g

def check_origin(f):
    @functools.wraps(f)
    def g(*args, **kwargs):
        if request.method == 'OPTIONS':
            resp = make_response('')
        else:
            if request.headers.get('Origin') == cfg['allowed_origin']:
                resp = make_response(f(*args, **kwargs))
            else:
                resp = make_response(('', 403))

        resp.headers['Access-Control-Allow-Origin'] = cfg['allowed_origin']
        resp.headers['Access-Control-Allow-Credentials'] = 'true'
        return resp

    return g

NORM_RE = re.compile(r'\s+')
def normalize_name(name):
    return NORM_RE.sub(' ', name.strip())

def build_result(*msgs, **kwargs):
    dct = kwargs
    if msgs:
        dct['msgs'] = msgs
    return dct

def ok(*msgs, **kwargs):
    return build_result(*msgs, status='ok', **kwargs)

def error(*msgs, **kwargs):
    return build_result(*msgs, status='error', **kwargs)

def bug(*msgs, **kwargs):
    return build_result(*msgs, status='bug', **kwargs)

def user_dispatch(result, ok_url, err_url, flash_ok=True, flash_category=None):
    print(result)
    if result['status'] == 'ok':
        if flash_ok:
            for msg in result.get('msgs', ()):
                flash(msg, flash_category)
        return redirect(ok_url)
    elif result['status'] == 'error':
        for msg in result.get('msgs', ()):
            flash(msg, flash_category)
        return redirect(err_url)
    elif result['status'] == 'bug':
        abort(400)
    assert False, 'bad status in result: %r' % result['status']

def api_dispatch(result):
    if result['status'] == 'ok' or result['status'] == 'error':
        return jsonify(result)
    elif  result['status'] == 'bug':
        return (jsonify(result), 400)
    assert False, 'bad status in result: %r' % result['status']


# Common actions

# The session has two fields:
#  uid      - A unique ID for this user.  For registered users, there will be a
#             corresponding row in the `users` table.
#  name     - The user's name.  Only set for registered users.

LOGIN_ERROR = 'Invalid username or password.'
@unpack_args
def do_login(name, password):
    name = normalize_name(name)

    result = db.lookup_user(name)
    if result is None:
        return error(LOGIN_ERROR)
    uid, name, old_hash = result

    pass_hash = bcrypt.hashpw(password.encode('utf-8'), old_hash.encode('ascii')).decode('ascii')
    # According to the python bcrypt devs, this non-timing-safe comparison is
    # actually okay...
    if pass_hash != old_hash:
        return error(LOGIN_ERROR)

    session['uid'] = uid
    session['name'] = name
    return ok('Logged in as %r.' % name)

NAME_RE = re.compile(r'^[a-zA-Z0-9- ]*$')
ALNUM_RE = re.compile(r'[a-zA-Z0-9]')
@unpack_args
def do_register(name, password, email):
    if 'name' in session:
        return error('You must log out first.')

    name = normalize_name(name)
    if len(name) == 0:
        return error('You must enter an account name.')
    if len(name) > 16:
        return error('Name is too long (must not exceed 16 characters).')
    if not NAME_RE.match(name):
        return error('Name may only contain letters, numbers, spaces, and hyphens.')
    if not ALNUM_RE.search(name):
        return error('Name must contain at least one letter or digit.')

    if len(password) < 8:
        return error('Password must be at least 8 characters long')

    if name in cfg['reserved_names']:
        if request.remote_addr != cfg['reserved_names'][name]:
            return error('That name is reserved for someone else.')

    pass_hash = bcrypt.hashpw(password.encode('utf-8'), bcrypt.gensalt()).decode('ascii')

    if 'uid' not in session:
        session['uid'] = db.next_id()

    reg_ok = db.register(session['uid'], name, pass_hash, email)
    if not reg_ok:
        return error('Account name %r is already in use.' % name)

    session['name'] = name
    return ok('Registered as %r.' % name)

def do_get_verify_key():
    key = cfg['signing_key'].verify_key
    key_str = key.encode(URLSafeBase64Encoder).decode('ascii')
    return ok(key=key_str)

@unpack_args
def do_check_login(auto_guest=False):
    if 'name' in session:
        return ok(type='normal', uid=session['uid'], name=session['name'])

    return ok(type='none', uid=session.get('uid'))

@unpack_args
def do_sign_challenge(challenge):
    if 'uid' not in session:
        session['uid'] = db.next_id()
    uid = session['uid']

    nonce_bytes = URLSafeBase64Encoder.decode(challenge.encode('ascii'))

    if 'name' in session:
        name = session['name']
        name_bytes = name.encode('utf-8')
    else:
        name_bytes = b''

    header = struct.pack('<BBBBI', len(nonce_bytes), len(name_bytes), 0, 0, uid)
    b = header + nonce_bytes + name_bytes
    signed = cfg['signing_key'].sign(b)
    signed_str = URLSafeBase64Encoder.encode(signed).decode('ascii')

    return ok(response=signed_str)


# User-facing routes

@app.route('/')
def index():
    return render_template('index.html',
            username=session.get('name'))

@app.route('/login', methods=['GET', 'POST'])
def login():
    if request.method == 'GET':
        return render_template('login.html')
    else:
        result = do_login(request.form)
        return user_dispatch(result,
                cfg['redir_url'] or url_for('index'),
                url_for('login'),
                flash_ok=cfg['redir_url'] is None,
                flash_category='login')

@app.route('/register', methods=['GET', 'POST'])
def register():
    if request.method == 'GET':
        return render_template('register.html')
    else:
        result = do_register(request.form)
        return user_dispatch(result,
                cfg['redir_url'] or url_for('index'),
                url_for('register'),
                flash_ok=cfg['redir_url'] is None,
                flash_category='register')

@app.route('/logout')
def logout():
    if 'uid' in session:
        del session['uid']
    if 'name' in session:
        del session['name']
    if cfg['redir_url'] is None:
        flash('Logged out.')
    return redirect(cfg['redir_url'] or url_for('index'))


@app.route('/api/get_verify_key')
def api_get_verify_key():
    result = do_get_verify_key()
    return api_dispatch(result)

@app.route('/api/check_login', methods=['GET', 'POST', 'OPTIONS'])
@check_origin
def api_check_login():
    if request.method == 'POST':
        args = request.get_json(force=True)
    else:
        args = {}

    result = do_check_login(args)
    return api_dispatch(result)

@app.route('/api/sign_challenge', methods=['POST', 'OPTIONS'])
@check_origin
def api_sign_challenge():
    result = do_sign_challenge(request.get_json(force=True))
    return api_dispatch(result)


if __name__ == '__main__':
    app.run()

