import MySQLdb
import MySQLdb.constants.ER

class Database:
    def __init__(self, cfg):
        args = ()
        if cfg['db_connstr'] is not None:
            args += (cfg['db_connstr'],)

        kwargs = {}
        if cfg['db_host'] is not None:
            kwargs['host'] = cfg['db_host']

        self.db = MySQLdb.connect(
                *args,
                db=cfg['db_name'],
                user=cfg['db_user'],
                passwd=cfg['db_pass'],
                **kwargs
                )

    def next_id(self):
        with self.db as curs:
            curs.execute('SELECT next_counter();')
            return curs.fetchone()[0]

    def lookup_user(self, name):
        with self.db as curs:
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
            with self.db as curs:
                curs.execute('INSERT INTO users (id, name, name_lower, password, email) '
                    'VALUES (%s, %s, %s, %s, %s)',
                    (uid, name, name.lower(), pass_hash, email))
                return True
        except MySQLdb.IntegrityError as e:
            if e.args[0] == MySQLdb.constants.ER.DUP_ENTRY:
                return False
            else:
                raise

