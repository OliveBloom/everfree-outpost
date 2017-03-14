var Config = require('config').Config;


function LogServer(url) {
    this.ws = new WebSocket(url);
    this.buffer = [];

    var this_ = this;
    this.ws.onopen = function() { this_._handleOpen(); };
    this.ws.onerror = function(evt) { this_._handleError(); };
}

LogServer.prototype._handleOpen = function() {
    for (var i = 0; i < this.buffer.length; ++i) {
        this.ws.send(this.buffer[i]);
    }
    this.buffer = [];
};

LogServer.prototype._handleError = function() {
    throw 'failed to connect to logging server: ' + this.ws.url;
};

LogServer.prototype.send = function(msg) {
    if (this.ws.readyState == WebSocket.OPEN) {
        this.ws.send(msg);
    } else {
        this.buffer.push(msg);
    }
};


function DummyLogServer() {
}

DummyLogServer.prototype.send = function(msg) {};


function makeLogServer(url) {
    console.log('make server with url ', url);
    if (url) {
        return new LogServer(url);
    } else {
        return new DummyLogServer();
    }
}

exports.LogServer = makeLogServer(Config.debug_log_server.get());
