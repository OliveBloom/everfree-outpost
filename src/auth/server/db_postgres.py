import psycopg2
import psycopg2.errorcodes

class Database:
    def __init__(self, cfg):
        self.db = psycopg2.connect(
                cfg['db_connstr'],
                host=cfg['db_host'],
                database=cfg['db_name'],
                user=cfg['db_user'],
                password=cfg['db_pass'],
                )

    def next_id(self):
        with self.db as db, db.cursor() as curs:
            curs.execute("SELECT nextval('counter')")
            return curs.fetchone()[0]

    def lookup_user(self, name):
        with self.db as db, db.cursor() as curs:
            curs.execute('SELECT id, name, password FROM users '
                    'WHERE name_lower = %s;',
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
            with self.db as db, db.cursor() as curs:
                curs.execute('INSERT INTO users (uid, name, name_lower, password, email) '
                    'VALUES (%s, %s, %s, %s, %s)',
                    (uid, name, name.lower(), pass_hash, email))
                return True
        except psycopg2.IntegrityError as e:
            if psycopg2.errorcodes.lookup(e.pgcode) == 'UNIQUE_VIOLATION':
                return False
            else:
                raise

