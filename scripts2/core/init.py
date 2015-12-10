def init(*args):
    print('hello', args)

    r, = args
    print(r)
    print(r.test_method)
    for i in range(2):
        print(r.test_method())
    print(r.test_method)
    print(r)
    import _outpost_server
    print(_outpost_server)
    print(_outpost_server.RustRef)
    return

    import _outpost_server
    print(_outpost_server)
    print(_outpost_server.RustRef)
    for i in range(100):
        x = _outpost_server.test_func()

    print(_outpost_server.test_func)
    print(_outpost_server.RustRef)
    print(_outpost_server.test_func())
    return
