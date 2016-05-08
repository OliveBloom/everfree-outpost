function unpack(arr) {
    return String.fromCharCode.apply(null, arr);
}

function pack(s) {
    var arr = new Uint8Array(s.length);
    packInto(s, arr);
    return arr;
}

function packInto(s, arr) {
    for (var i = 0; i < s.length; ++i) {
        arr[i] = s.charCodeAt(i);
    }
}

function base64(s) {
    return btoa(s).replace(/\+/g, '-').replace(/\//g, '_');
}

function unbase64(s) {
    return atob(s.replace(/-/g, '+').replace(/_/g, '/'));
}

function fromUTF8(s) {
    return decodeURIComponent(escape(s));
}


var OP_AUTH_RESPONSE =      0x0014;
var OP_AUTH_CHALLENGE =     0x801c;
var OP_AUTH_RESULT =        0x801d;

var MODE_SSO =      0;
var MODE_LOCAL =    1;

var SSO_ENDPOINT = 'http://localhost:5000/api/';


/** @constructor */
function Launcher(server_url) {
    this.current = 0;
    this.xhr = null;
    this.error = null;

    this.cur_files = null;
    this.total_files = null;

    this.cur_bytes = null;          // downloaded bytes of the current file
    this.total_bytes = null;        // total bytes of the current file
    this.finished_bytes = null;     // donloaded bytes of finished files
    this.all_bytes = null;          // total bytes of all files

    // 0 - connect to server
    this.server_url = new URL(server_url);
    this.server_info = null;
    this.version_info = null;
    this.conn = null;

    // 1 - log in
    this.login_name = null;

    // 2 - load code
    this.client = null;

    // 3 - load data
    this.data_array = null;

    // 4 - load world
    // (no state here, only in the client)
}

Launcher.prototype.start = function() {
    // TODO: set up canvas
    // TODO: set up resize handler

    this._getServerInfo();
};

Launcher.prototype._xhr = function(url, type, handlers, post, with_creds) {
    var xhr = new XMLHttpRequest();
    xhr.open(post != null ? 'POST' : 'GET', url, true);
    xhr.responseType = type;
    xhr.withCredentials = !!with_creds;

    xhr.onprogress = handlers.progress;
    xhr.onload = function(evt) {
        if (xhr.status == 200) {
            handlers.load(evt);
        } else {
            handlers.error(evt);
        }
    };
    xhr.onerror = handlers.error;

    if (post != null) {
        xhr.send(post);
    } else {
        xhr.send();
    }

    this.xhr = xhr;
}

Launcher.prototype._error = function(msg) {
    if (this.error == null) {
        console.error(msg);
        this.error = msg;
        if (this.xhr && this.xhr.readyState < XMLHttpRequest.DONE) {
            this.xhr.abort();
        }
        if (this.conn && this.conn.readyState <= XMLHttpRequest.OPEN) {
            this.conn.close();
        }
    }
};

Launcher.prototype._useDefaultOnMessage = function() {
    var this_ = this;
    this.conn.onmessage = function(evt) {
        this_._error('Unexpected message from server');
    };
};

//
// 0 - connect to server
//

Launcher.prototype._getServerInfo = function() {
    if (this.error) {
        return;
    }
    var this_ = this;

    this.current = 0;

    this._xhr(new URL('server.json', this.server_url).href, 'json', {
        load: function(evt) {
            this_.server_info = evt.target.response;
            this_._getVersionInfo();
        },
        error: function(evt) {
            this_._error((evt.target.statusText || 'Connection error') +
                    ' (while getting server info)');
        },
    });
};

Launcher.prototype._getVersionInfo = function() {
    if (this.error) {
        return;
    }
    var this_ = this;

    var version = this.server_info['version'];
    this._xhr(new URL('versions/' + version + '.json', document.location).href, 'json', {
        /*
        progress: function(evt) {
            this_.cur_bytes = evt.loaded;
            this_.total_bytes = evt.lengthComputable ? evt.total : null;
        },
        */
        load: function(evt) {
            this_.version_info = evt.target.response;
            this_._connectWebsocket();
        },
        error: function(evt) {
            this_._error((evt.target.statusText || 'Connection error') +
                    ' (while getting version info)');
        },
    });
};

Launcher.prototype._connectWebsocket = function() {
    if (this.error) {
        return;
    }
    var this_ = this;

    var conn = new WebSocket(this.server_info['url']);
    conn.binaryType = 'arraybuffer';
    conn.onopen = function(evt) {
        this_._authBegin();
    };
    conn.onerror = function(evt) {
        this_._error('Error connecting to game server');
    };
    conn.onclose = function(evt) {
        this_._error('Game server disconnected unexpectedly');
    };

    this.conn = conn;
    this._useDefaultOnMessage();
};

//
// 1 - authenticate
//

Launcher.prototype._authBegin = function() {
    if (this.error) {
        return;
    }
    var this_ = this;

    this.current = 1;

    this.conn.onmessage = function(evt) {
        var view = new DataView(evt.data);
        if (view.byteLength < 4) {
            this_._error('Unexpected message from server');
            return;
        }

        var opcode = view.getUint16(0, true);
        if (opcode == OP_AUTH_CHALLENGE) {
            var mode = view.getUint16(2, true);
            var challenge = new Uint8Array(evt.data).subarray(4);
            this_._authChallenge(mode, challenge);
        } else if (opcode == OP_AUTH_RESULT) {
            var flags = view.getUint16(2, true);
            var result = new Uint8Array(evt.data).subarray(4);
            this_._authResult(flags, result);
        } else {
            this_._error('Unexpected message from server');
        }
    };
};

Launcher.prototype._authChallenge = function(mode, challenge) {
    if (this.error) {
        return;
    }
    var this_ = this;

    if (mode == MODE_SSO) {
        var json = JSON.stringify({
            'challenge': base64(unpack(challenge)),
        });
        this._xhr(new URL('sign_challenge', SSO_ENDPOINT).href, 'json', {
            load: function(evt) {
                this_._authResponse(evt.target.response);
            },
            error: function(evt) {
                this_._error(evt.target.statusText || 'Connection error');
            },
        }, json, true);
    } else {
        this._error('Authentication mode ' + mode + ' is not supported');
    }
};

Launcher.prototype._authResponse = function(json_obj) {
    if (this.error) {
        return;
    }

    if (json_obj['status'] == 'ok') {
        var response = unbase64(json_obj['response']);
        var msg = new Uint8Array(2 + response.length);
        new DataView(msg.buffer).setUint16(0, OP_AUTH_RESPONSE, true);
        packInto(response, msg.subarray(2));
        this.conn.send(msg);
    } else {
        if (json_obj['reason'] == 'not_logged_in') {
            this._error('You are not logged in.')
        } else {
            this._error(json_obj['reason']);
        }
    }
};

Launcher.prototype._authResult = function(flags, result) {
    if (this.error) {
        return;
    }

    var s = fromUTF8(unpack(result));
    if ((flags & 1) == 0) {
        this._error(s);
        return;
    }

    this.login_name = result;

    this._loadCode();
};

//
// 2 - download code
//

Launcher.prototype._loadCode = function() {
    this.current = 2;
};

window.L = new Launcher('http://localhost:8889/');
window.L.start();
