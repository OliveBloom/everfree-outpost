from collections import namedtuple
import json
import sys

# Basic module contents

Module = namedtuple('Module', ('code', 'data', 'metadata'))

def read_emscripten_output(f):
    # Ignore everything before functions
    for line in f:
        if line.strip() == '// EMSCRIPTEN_START_FUNCTIONS':
            break
    else:
        assert False, "couldn't find EMSCRIPTEN_START_FUNCTIONS"

    # Read all functions into `code`
    code = ''
    for line in f:
        if line.strip() == '// EMSCRIPTEN_END_FUNCTIONS':
            break
        code += line
    else:
        assert False, "couldn't find EMSCRIPTEN_END_FUNCTIONS"

    # Read body of `allocate()` into `data`.
    for line in f:
        if line.startswith('/* memory initializer */'):
            start = line.index('[')
            end = line.index(']', start)
            data = line[start + 1 : end]
            break
    else:
        assert False, "couldn't find /* memory initializer */"

    # Ignore everything from `allocate` to start of metadata (there should be
    # nothing of interest there)
    for line in f:
        if line.strip() == '// EMSCRIPTEN_METADATA':
            break
    else:
        assert False, "couldn't find EMSCRIPTEN_METADATA"

    # Read metadata
    metadata_str = ''
    for line in f:
        metadata_str += line
    metadata = json.loads(metadata_str)

    return Module(code, data, metadata)

# Function pointer tables and related info

FuncTables = namedtuple('FuncTables', ('tables', 'aborts', 'sizes'))

def make_cast(val, ty):
    if ty == 'i':
        return '%s|0' % val;
    elif ty == 'd':
        return '+%s' % val
    elif ty == 'f':
        return 'fround(%s)' % val
    elif ty == 'v':
        return ''
    else:
        raise ValueError('unknown type: %s' % ty)

def make_abort_func(sig, name):
    ret_ty = sig[0]
    arg_tys = sig[1:]
    args = ', '.join('$%d' % i for i in range(len(arg_tys)))
    body = ' '.join('$%d = %s;' % (i, make_cast('$%d' % i, arg_tys[i]))
            for i in range(len(arg_tys)))
    ret = make_cast(0, ret_ty)
    return 'function %s(%s) { %s abort(); return %s; }' % (name, args, body, ret)

def build_tables(metadata):
    tables = ''
    aborts = ''
    sizes = {}

    for k, v in metadata['tables'].items():
        start = v.index('[')
        end = v.index(']')
        funcs = v[start + 1 : end].split(',')

        sizes[k] = len(funcs)

        abort_name = '__abort_%s' % k
        abort_func = make_abort_func(k, abort_name)
        aborts += abort_func + '\n'

        fn_table_body = ',\n    '.join(f if f != '0' else abort_name for f in funcs)
        tables += 'var FUNCTION_TABLE_%s = [\n    %s\n  ];\n' % (k, fn_table_body)

    return FuncTables(tables, aborts, sizes)

def substitute_table_sizes(code, sizes):
    for k,v in sizes.items():
        # Size is always a power of two, so the bit mask is size - 1.
        code = code.replace('#FM_%s#' % k, str(v - 1))
    return code


def main(template_path, asm_path, exports_path):
    with open(asm_path) as f:
        module = read_emscripten_output(f)
    tables = build_tables(module.metadata)

    combined_code = '\n\n'.join((
        substitute_table_sizes(module.code, tables.sizes),
        tables.tables,
        tables.aborts,
        ))

    exports = []
    with open(exports_path) as f:
        for line in f:
            line = line.strip()
            if line == '' or line[0] == '#':
                continue
            exports.append(line)


    parts = {
            'FUNCTIONS': combined_code,
            'EXPORTS': '\n'.join('%s: _%s,' % (x, x) for x in exports),
            'STATIC': module.data,
            'DATA_SIZE': str(module.metadata['staticBump']),
            }

    prefix = '// INSERT_EMSCRIPTEN_'
    with open(template_path) as f:
        for line in f:
            if line.strip().startswith(prefix):
                tail = line.strip()[len(prefix):]
                sys.stdout.write(parts[tail])
                sys.stdout.write('\n')
            else:
                sys.stdout.write(line)

if __name__ == '__main__':
    template_path, asm_path, exports_path = sys.argv[1:]
    main(template_path, asm_path, exports_path)
