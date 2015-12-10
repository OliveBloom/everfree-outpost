def init(*args):
    print('hello', args)
    import _outpost_server
    print(_outpost_server.RustRef)
    for i in range(100):
        x = _outpost_server.test_func()

    print(_outpost_server)
    print(_outpost_server.test_func)
    print(_outpost_server.RustRef)
    print(_outpost_server.test_func())
    return
