import os
import sqlite3
import threading

def _load_schema():
    d = os.path.dirname(__file__)
    with open(os.path.join(d, 'schema-sqlite.sql')) as f:
        return f.read()

class Database:
    def __init__(self, cfg):
        self.connstr = cfg['db_connstr']
        self.dbs = {}

        if not os.path.exists(self.connstr):
            print('initializing %s' % self.connstr)
            with self.db:
                self.db.executescript(_load_schema())

    @property
    def db(self):
        tid = threading.get_ident()
        if tid not in self.dbs:
            self.dbs[tid] = sqlite3.connect(self.connstr)
        return self.dbs[tid]

    def next_id(self):
        with self.db:
            curs = self.db.cursor()
            curs.execute("SELECT value FROM counter;")
            uid, = curs.fetchone()
            curs.execute("UPDATE counter SET value = ?;", (uid + 1,))
            return uid

    def lookup_user(self, name):
        with self.db:
            curs = self.db.cursor()
            curs.execute('SELECT id, name, password FROM users '
                    'WHERE name_lower = ?;',
                    (name.lower(),))
            rows = curs.fetchall()
        if len(rows) == 0:
            return None
        elif len(rows) == 1:
            return rows[0]
        else:
            assert False, 'UNIQUE constraint should forbid >1 row in result'

    def register(self, uid, name, pass_hash, email):
        try:
            with self.db:
                curs = self.db.cursor()
                curs.execute('INSERT INTO users (id, name, name_lower, password, email) '
                    'VALUES (?, ?, ?, ?, ?)',
                    (uid, name, name.lower(), pass_hash, email))
                return True
        except sqlite3.IntegrityError as e:
            if str(e).startswith('UNIQUE constraint failed:'):
                return False
            else:
                raise

