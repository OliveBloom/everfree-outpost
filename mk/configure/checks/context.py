import os
import platform
import subprocess
import sys
import tempfile

win32 = platform.system() == 'Windows'

if not win32:
    from shlex import quote
else:
    def quote(path):
        return '"%s"' % path.replace('"', '""')


class Info:
    def __init__(self):
        pass

    def __getattr__(self, k):
        return None

class ConfigError(Exception):
    pass

class Context:
    def __init__(self, args, temp_dir, log_file):
        self.info = Info()
        self.args = args

        self.temp_dir = temp_dir
        self.counter = 0
        self.log_file = log_file

    # Utility functions
    def file(self, ext):
        name = os.path.join(self.temp_dir, 'tmp%06d.%s' % (self.counter, ext))
        self.counter += 1
        return name

    def write(self, ext, content, mode='w'):
        name = self.file(ext)
        with open(name, mode) as f:
            f.write(content)
        self.log('Created file %s with contents:' % name)
        self.trace(content)
        return name

    def log(self, msg, level='INFO'):
        for line in msg.splitlines():
            self.log_file.write(' [%s] %s\n' % (level.center(4), line))

    def warn(self, msg):
        self.log(msg, level='WARN')

    def err(self, msg):
        self.log(msg, level='ERR')

    def trace(self, msg):
        self.log(msg, level='TRC')

    def out(self, msg, level='MSG'):
        self.log(msg, level=level)
        print(msg)

    def out_part(self, msg, level='MSG'):
        self.log(msg, level=level)
        print(msg, end='')

    def out_skip(self, what, why, level='MSG'):
        self.out('Skipping check for %s because %s is missing' % (what, why), level=level)

    # Command running
    def run(self, prog, args=[], expect_ret=0):
        if prog is None:
            self.warn('Skipping check because a needed program was not found')
            return None

        cmd = prog + ''.join(' ' + quote(a) for a in args)
        self.log('Execute: %r' % cmd)
        self.log_file.flush()
        ret = subprocess.call(cmd, shell=True,
                stdin=subprocess.DEVNULL, stdout=self.log_file, stderr=subprocess.STDOUT)
        self.log_file.flush()
        if expect_ret is None or ret == expect_ret:
            self.log('Process %r returned %d (ok)' % (prog, ret))
            return True
        else:
            self.warn('Process %r returned %d (expected %d)' % (prog, ret, expect_ret),)
            return None

    def run_output(self, prog, args=[], expect_ret=0):
        cmd = prog + ''.join(' ' + quote(a) for a in args)
        self.log('Execute: %r' % cmd)
        p = subprocess.Popen(cmd, shell=True,
                stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        output, _ = p.communicate()

        if expect_ret is not None and p.returncode != expect_ret:
            self.warn('Process %r returned %d (expected %d)' %
                    (prog, p.returncode, expect_ret))
            return None

        output = output.decode()
        self.log('Process %r output:' % prog)
        self.trace(output)
        # If it returns nonzero, we get an exception from check_output.
        self.log('Process %r returned %d (ok)' % (prog, p.returncode))
        return output

    # Collect candidates, considering both args and a default
    def get_candidates(self, arg_name, default=()):
        val = getattr(self.args, arg_name, None)
        if val is None:
            return default
        else:
            return (val,)

    # Run check
    def check_present(self, desc, x, chk):
        self.out_part('Checking for %s %r: ' % (desc, x))
        try:
            ok = chk(self, x)
            if ok:
                self.out('ok')
                return True
            else:
                raise ConfigError('error')
        except ConfigError as e:
            self.out(str(e))
            return False

    def check_all(self, desc, candidates, chk):
        if len(candidates) == 0:
            self.out('Checking %s: (no candidates to check)' % desc, level='WARN')
            return None

        for c in candidates:
            self.out_part('Checking %s %r: ' % (desc, c))
            try:
                ok = chk(self, c)
                if ok:
                    self.out('ok')
                    return c
                else:
                    raise ConfigError('error')
            except ConfigError as e:
                self.out(str(e))

        return None
