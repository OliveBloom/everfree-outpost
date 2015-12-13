def init(*args):
    print('hello', args)

    r, = args
    print(r)
    print(r.script_dir)
    for i in range(2):
        print(r.script_dir())
    print(r.script_dir)
    print(r)
    import _outpost_server
    print(_outpost_server)
    print(_outpost_server.StorageRef)
    return
