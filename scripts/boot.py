"""Outpost server initialization script.

On server startup, this script runs (as `__main__`) to set up search paths,
custom importers, etc., so that the server binary will be able find the
`outpost.core.init` module.  That module handles the rest of the init process.
"""

import sys
import os

class FakePackage(object):
    def __init__(self, name, path):
        self.__name__ = name
        self.__package__ = name
        self.__path__ = [path]

if __name__ == '__main__':
    script_dir = os.path.dirname(__file__)
    sys.modules['outpost_server'] = FakePackage('outpost_server', script_dir)
