import configparser
import functools
import inspect
import os
import re
import struct

from flask import Flask, request, jsonify, render_template, \
        flash, redirect, url_for, abort, session, make_response

import bcrypt
import nacl.secret
import nacl.signing
from nacl.encoding import RawEncoder, URLSafeBase64Encoder
import psycopg2
import psycopg2.errorcodes


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

    def decrypt(k):
        value = get(k)
        return box.decrypt(URLSafeBase64Encoder.decode(value.encode('ascii')))

    return {
            'flask_debug': int(get('flask_debug')),
            'flask_secret_key': decrypt('flask_secret_key'),

            'signing_key': nacl.signing.SigningKey(decrypt('signing_key')),

            'db_name': get('db_name'),
            'db_user': get('db_user'),
            'db_pass': decrypt('db_pass').decode('utf-8'),

            'allowed_origin': get('allowed_origin'),
            }

cfg = read_config()

app = Flask(__name__)
app.debug = bool(cfg['flask_debug'])
app.secret_key = cfg['flask_secret_key']

db = psycopg2.connect(
        database=cfg['db_name'],
        user=cfg['db_user'],
        password=cfg['db_pass'],
        )


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

def user_dispatch(result, ok_url, err_url):
    print(result)
    if result['status'] == 'ok':
        for msg in result.get('msgs', ()):
            flash(msg)
        return redirect(ok_url)
    elif result['status'] == 'error':
        for msg in result.get('msgs', ()):
            flash(msg)
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

def start_session(uid, name):
    session['uid'] = uid
    session['name'] = name

def clear_session():
    if 'uid' in session:
        del session['uid']
    if 'name' in session:
        del session['name']

LOGIN_ERROR = 'Invalid username or password.'
@unpack_args
def do_login(name, password):
    name = normalize_name(name)

    with db, db.cursor() as curs:
        curs.execute('SELECT id, name, password FROM users '
                'WHERE name_lower = %s;',
                (name.lower(),))
        rows = curs.fetchall()

    if len(rows) == 0:
        return error(LOGIN_ERROR)
    assert len(rows) == 1, 'impossible to have more than one row with the same name_lower'

    uid, name, old_hash = rows[0]

    pass_hash = bcrypt.hashpw(password.encode('utf-8'), old_hash.encode('ascii')).decode('ascii')
    # According to the python bcrypt devs, this non-timing-safe comparison is
    # actually okay...
    if pass_hash != old_hash:
        return error(LOGIN_ERROR)

    start_session(uid, name)
    return ok('Logged in as %r.' % name)

NAME_RE = re.compile(r'^[a-zA-Z0-9- ]*$')
ALNUM_RE = re.compile(r'[a-zA-Z0-9]')
@unpack_args
def do_register(name, password, email):
    name = normalize_name(name)
    if len(name) == 0:
        return error('You must enter an account name.')
    if len(name) > 16:
        return error('Name is too long (must not exceed 16 characters).')
    if not NAME_RE.match(name):
        return error('Name may only contain letters, numbers, spaces, and hyphens.')
    if not ALNUM_RE.search(name):
        return error('Name must contain at least one letter or digit.')

    pass_hash = bcrypt.hashpw(password.encode('utf-8'), bcrypt.gensalt()).decode('ascii')

    try:
        with db, db.cursor() as curs:
            curs.execute('INSERT INTO users (name, name_lower, password, email) '
                'VALUES (%s, %s, %s, %s) RETURNING id',
                (name, name.lower(), pass_hash, email))
            uid, = curs.fetchone()
    except psycopg2.IntegrityError as e:
        if psycopg2.errorcodes.lookup(e.pgcode) == 'UNIQUE_VIOLATION':
            return error('Account name %r is already in use.' % name)
        else:
            raise

    start_session(uid, name)
    return ok('Registered as %r.' % name)


def do_get_verify_key():
    key = cfg['signing_key'].verify_key
    key_str = key.encode(URLSafeBase64Encoder).decode('ascii')
    return ok(key=key_str)

@unpack_args
def do_check_login(auto_guest=False):
    if 'name' in session:
        return ok(type='normal', name=session['name'])

    if auto_guest:
        # TODO: generate new guest account
        return ok(type='guest', name='Anon1234')

    return ok(type='none')


@unpack_args
def do_sign_challenge(challenge):
    if 'uid' not in session or 'name' not in session:
        return error(reason='not_logged_in')
    uid = session['uid']
    name = session['name']
    nonce_bytes = URLSafeBase64Encoder.decode(challenge.encode('ascii'))

    name_bytes = name.encode('utf-8')
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
                url_for('index'),
                url_for('login'))

@app.route('/register', methods=['GET', 'POST'])
def register():
    if request.method == 'GET':
        return render_template('register.html')
    else:
        result = do_register(request.form)
        return user_dispatch(result,
                url_for('index'),
                url_for('register'))

@app.route('/logout')
def logout():
    clear_session()
    flash('Logged out.')
    return redirect(url_for('index'))


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

