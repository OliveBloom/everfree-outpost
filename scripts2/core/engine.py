class EngineProxy(object):
    def __init__(self, eng):
        self._eng = eng

    def num_clients(self):
        return self._eng.messages_clients_len()

class ClientProxy(object):
    def __init__(self, eng, id):
        self._eng = eng
        self._engine = None
        self.id = id

    @property
    def engine(self):
        if self._engine is None:
            self._engine = EngineProxy(self._eng)
        return self._engine

    def send_message(self, msg):
        self._eng.messages_send_chat_update(self.id, '***\t' + msg)
