import json
import sys
import time

if __name__ == '__main__':
    obj = {
            'url': 'ws://localhost:8888/ws',
            'version': 'dev',
            'pack': 'outpost.pack',
            }
    json.dump(obj, sys.stdout)
