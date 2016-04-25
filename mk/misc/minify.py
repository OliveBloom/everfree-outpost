"""Minifier for emscripten-generated asm.js code.  This is NOT a
general-purpose minifier, and will likely break if used on non-asm code."""
from collections import namedtuple
import re
import sys

FUNCTION_RE = re.compile(r'''
        \b function \b
        (?: \s+ ([a-zA-Z0-9_$]+) )? # function name
        \s* \( ([^)]*) \)           # args
        \s* {
        | [{}]
        ''', re.VERBOSE)

Function = namedtuple('Function', ('name', 'code', 'locals'))

def parse_funcs(s):
    stack = [Function(None, [], set())]
    level = [0]
    i = 0
    while i < len(s):
        m = FUNCTION_RE.search(s, i)
        if m is None:
            stack[-1].code.append(s[i:])
            break

        j = m.end()

        is_func = False
        if m.group() == '{':
            level[-1] += 1
            stack[-1].code.append(s[i:j])
        elif m.group() == '}':
            level[-1] -= 1
            stack[-1].code.append(s[i:j])
            if level[-1] < 0:
                f = stack.pop()
                stack[-1].code.append(f)
                level.pop()
        else:
            stack[-1].code.append(s[i:m.start()])
            header = s[m.start():m.end()]

            name = m.group(1)
            if name is not None:
                stack[-1].locals.add(name)

            args = set(a.strip() for a in m.group(2).split(','))

            stack.append(Function(name, [header], args))
            level.append(0)

        i = j

    return stack[0]


VAR_RE = re.compile(r'\bvar\b([^;]*);')

def parse_vars(f):
    for c in f.code:
        if isinstance(c, Function):
            parse_vars(c)
            continue

        for m in VAR_RE.finditer(c):
            for decl in m.group(1).split(','):
                name = decl.partition('=')[0].strip()
                f.locals.add(name)

def raw_encode(i):
    def i2l(i):
        if i < 26:
            return chr(ord('a') + i)
        if i < 52:
            return chr(ord('A') + i - 26)
        if i < 54:
            return '_$'[i - 52]
        if i < 64:
            return chr(ord('0') + i - 54)

    s = i2l(i % 54)
    i //= 54

    while i > 0:
        i -= 1
        s += i2l(i % 64)
        i //= 64

    return s

def _decode_kw(s):
    x = 0
    for i, c in reversed(list(enumerate(s))):
        a = ord(c) - ord('a')
        if i == 0:
            x = a + 54 * x
        else:
            x = 1 + a + 64 * x
    #a = ord(s[0]) - ord('a')
    #b = ord(s[1]) - ord('a')
    #i = a + 54 * (1 + b)
    assert raw_encode(x) == s
    return x

KEYWORD_CODES = sorted(_decode_kw(kw) for kw in
        # NB: this list must be sorted
        ('do', 'if', 'in')
        )

def encode(i):
    for c in KEYWORD_CODES:
        if i >= c:
            i += 1

    return raw_encode(i)

test_set = {}
for i in range(0, 10000):
    s = encode(i)
    assert s not in test_set, '%d / %d = %s' % (i, test_set[s], s)
    test_set[s] = i

# Can't use word-boundary (\b) anchors here because $ is not a word character.
# Rely on greedy matching instead.
#
# Don't match `.foo` (attribute access) or `foo:` (entry in an object literal)
NAME_RE = re.compile(r'(?<!\.)[a-zA-Z0-9_$]+(?!:)')

def rename(f, name_map={}, depth=0):
    name_map = name_map.copy()
    for l in sorted(f.locals):
        if l not in name_map:
            name_map[l] = encode(len(name_map))

    def repl(m):
        s = m.group()
        return name_map.get(s, s)

    for i, c in enumerate(f.code):
        if isinstance(c, str):
            f.code[i] = NAME_RE.sub(repl, c)
        else:
            rename(c, name_map, depth + 1)

WHITE_RE = re.compile(r'\s+')
COMMENT_RE = re.compile(r'//.*')
BREAK_RE = re.compile(r'[,;{}]')

def remove_whitespace(s):
    def repl(m):
        nonlocal last

        if m.start() == 0 or m.end() == len(s):
            return ''

        a = s[m.start() - 1]
        b = s[m.end()]
        # Avoid turning 'x + +y' into 'x++y'.
        if (NAME_RE.match(a) and NAME_RE.match(b)) or \
                (a + b in ('++', '--')):
            return ' '
        else:
            return ''

    last = 0
    def repl_break(m):
        nonlocal last
        s = m.group()
        if m.end() - last > 180:
            last = m.end()
            return s + '\n'
        else:
            return s

    s = COMMENT_RE.sub('', s)
    s = WHITE_RE.sub(repl, s)
    s = BREAK_RE.sub(repl_break, s)
    return s

def print_func(f):
    if isinstance(f, str):
        return f
    return ''.join(print_func(c) for c in f.code)

s = sys.stdin.read()
f = parse_funcs(s)
parse_vars(f)
rename(f)
t = print_func(f)
t = remove_whitespace(t)
sys.stdout.write(t)
