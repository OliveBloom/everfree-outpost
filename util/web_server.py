import json
import sys

import tornado.ioloop
import tornado.web


BIND_ADDR = '127.0.0.1'
PORT = 8889


class DefaultHandler(tornado.web.RequestHandler):
    def get(self):
        self.redirect('/launcher/serverlist.html')

PATH_MAP = (
        ('/versions/dev', 'client'),
        ('/server', 'server/www'),
        ('/launcher', 'launcher'),
        ('/website', 'website'),
        )

if __name__ == "__main__":
    routes = [(url + r'/(.*)', tornado.web.StaticFileHandler, {'path': path})
            for url, path in PATH_MAP]
    routes.append(('/', DefaultHandler))
    application = tornado.web.Application(routes, debug=True)
    application.listen(PORT, BIND_ADDR)
    print('listening on %s:%d' % (BIND_ADDR, PORT))
    tornado.ioloop.IOLoop.current().start()

