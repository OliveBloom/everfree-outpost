def do_eval(eng, code):
    try:
        if '\n' in code[:-1]:
            exec(code)
            return ''
        else:
            return str(eval(code))
    except Exception as e:
        return repr(e)

    hooks.set_eval(do_eval)

def init(hooks):
    hooks.set_eval(do_eval)
