import json
import sys

import tornado.ioloop
import tornado.web
import tornado.websocket
import tornado.httpclient
from tornado.web import asynchronous
from tornado.gen import coroutine


MODE, = sys.argv[1:]

if MODE == 'auth':
    BASE_URL = 'https://auth.everfree-outpost.com/'
    HOST = 'auth.everfree-outpost.com'
    PORT = 5001
elif MODE == 'play':
    BASE_URL = 'http://play.everfree-outpost.com/'
    HOST = 'play.everfree-outpost.com'
    PORT = 5002

class ProxyHandler(tornado.web.RequestHandler):
    async def get(self, path):
        print('\nproxying %s for %s' % (self.request.method, path))
        client = tornado.httpclient.AsyncHTTPClient()

        #print(' ** request headers: ', dict(self.request.headers.get_all()))
        headers = dict(self.request.headers.get_all())
        headers['Origin'] = 'http://play.everfree-outpost.com'
        headers['Host'] = HOST
        print(' ** sending request headers: ', headers)

        result = await client.fetch(BASE_URL + path,
                method=self.request.method,
                headers=headers,
                body=self.request.body or None,
                follow_redirects=False,
                raise_error=False)

        print('got %d bytes for %s' % (len(result.body), result.effective_url))
        if result.code == 200:
            body = result.body
            if MODE == 'play' and path == 'server/server.json':
                j = json.loads(body.decode())
                j['version'] = 'dev'
                body = json.dumps(j).encode()
                result.headers['Content-Length'] = len(body)
                print('rewrote server.json to %r' % body)
            self.write(body)
        print(' ** response headers: ', dict(result.headers.get_all()))

        self.set_status(result.code, result.reason)
        for k,v in result.headers.get_all():
            self.set_header(k, v)
        if 'Origin' in self.request.headers:
            self.set_header('Access-Control-Allow-Origin', self.request.headers['Origin'])

    post = get

if __name__ == "__main__":
    application = tornado.web.Application([
        (r"/(.*)", ProxyHandler),
    ])
    application.listen(PORT)
    tornado.ioloop.IOLoop.current().start()
