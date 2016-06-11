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


/** @constructor */
function Launcher(server_url) {
    this.current = 0;
    this.xhr = null;
    this.error = null;

    this.onload = null;

    this.cur_progress = null;
    this.cur_total = null;
    this.completed_progress = null;
    this.grand_total = null;

    // 0 - connect to server
    this.server_url = new URL(server_url);
    this.server_info = null;
    this.base_url = null;
    this.version_info = null;
    this.conn = null;

    // 1 - log in
    this.login_name = null;

    // 2 - load data
    this.data_blob = null;

    // 3 - load code
    this.client = null;

    // 4 - load world
    // (no state here, only in the client)
}

function kb(b) {
    return (b / 1024)|0;
}

function byteProgress(cur, total) {
    var pct = (cur / total * 100)|0;
    return kb(cur) + 'k/' + kb(total) + 'k bytes (' + pct + '%)';
}

Launcher.prototype.getMessage = function() {
    if (this.error != null) {
        return 'Error: ' + this.error;
    }

    switch (this.current) {
        case 0: 
            if (!this.server_info) return 'Retrieving server info';
            if (!this.version_info) return 'Retrieving version info';
            return 'Connecting to server';

        case 1:
            return 'Logging in';

        case 2:
        case 3:
            // grand_total may be null prior to the first xhr progress event
            if (this.grand_total == null) return 'Loading';
            return 'Loading: ' +
                byteProgress(this.cur_progress + this.completed_progress, this.grand_total);

        case 4:
            return 'Loading world';

        default:
            return '???';
    }
};

Launcher.prototype.getProgress = function() {
    switch (this.current) {
        case 0:
            return 0;

        case 1:
            return 0;

        case 2:
        case 3:
            if (this.grand_total == null) return 0;
            return (this.cur_progress + this.completed_progress) / this.grand_total;

        case 4:
            return 1;

        default:
            return 0;
    }
};


Launcher.prototype.start = function() {
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
    this.base_url = new URL('' + version + '/', VERSIONS_BASE).href;
    this._xhr(new URL('manifest.json', this.base_url).href, 'json', {
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

    this.login_name = s;

    this._loadData();
};

//
// 2 - download data
//

Launcher.prototype._loadData = function() {
    if (this.error) {
        return;
    }
    var this_ = this;

    this.current = 2;

    this.cur_progress = 0;
    this.completed_progress = 0;
    this._xhr(new URL('outpost.pack', this.server_url).href, 'blob', {
        progress: function(evt) {
            this_.cur_progress = evt.loaded;
            // Assume the pack is no more than a megabyte
            this_.cur_total = evt.lengthComputable ? evt.total : 1024 * 1024;
            if (this_.grand_total == null) {
                this_.grand_total = this_.cur_total + this_.version_info['total_size'];
            }
        },
        load: function(evt) {
            // In case the progress event never fires
            if (this_.cur_total == null) {
                this_.cur_total = evt.lengthComputable ? evt.total : 1024 * 1024;
                if (this_.grand_total == null) {
                    this_.grand_total = this_.cur_total + this_.version_info['total_size'];
                }
            }

            this_.completed_progress += this_.cur_total;
            this_.cur_progress = 0;
            this_.data_blob = evt.target.response;
            this_._loadCode();
        },
        error: function(evt) {
            this_._error(evt.target.statusText || 'Connection error');
        },
    });
};

/*
 * TODO
Launcher.prototype._processData = function(blob) {
};
*/

//
// 3 - download code
//

Launcher.prototype._loadCode = function() {
    if (this.error) {
        return;
    }

    this.current = 3;

    // From now on, resolve paths relative to this.base_url.  Note this only
    // applies to URLs in the document (such as CSS image URLs), not to XHRs.
    var base = document.createElement('base');
    base.setAttribute('href', this.base_url);
    document.body.appendChild(base);

    // Hacky module implementation
    window.exports = {};
    window.require = function() { return window.exports; };

    this._loadCodeIndexed(0);
};

Launcher.prototype._loadCodeIndexed = function(idx) {
    if (this.error) {
        return;
    }
    var this_ = this;

    if (idx >= this.version_info['files'].length) {
        this._prepareClient();
        return;
    }

    var path = this.version_info['files'][idx];
    this.cur_progress = 0;
    this.cur_total = null;
    var type = path.endsWith('.html') ? 'document' : 'blob';
    this._xhr(new URL(path, this.base_url).href, type, {
        progress: function(evt) {
            this_.cur_progress = evt.loaded;
            this_.cur_total = evt.lengthComputable ? evt.total : 1024 * 1024;
        },
        load: function(evt) {
            if (this_.cur_total == null) {
                this_.cur_total = evt.lengthComputable ? evt.total : 1024 * 1024;
            }

            this_.completed_progress += this_.cur_total;
            this_.cur_progress = 0;
            this_._addCode(path, evt.target.response, function() {
                this_._loadCodeIndexed(idx + 1);
            });
        },
        error: function(evt) {
            this_._error((evt.target.statusText || 'Connection error') +
                    ' (while loading ' + path + ')');
        },
    });
};

Launcher.prototype._addCode = function(path, obj, next) {
    if (path.endsWith('.html')) {
        while (obj.body.firstChild) {
            document.body.appendChild(obj.body.firstChild);
        }
        next();
    } else {
        var script = document.createElement('script');
        script.setAttribute('src', URL.createObjectURL(obj) + '#_/' + path);
        // Don't proceed until the script has been executed.
        script.onload = next;
        document.body.appendChild(script);
    }
};

//
// 4 - handoff to OutpostClient
//

Launcher.prototype._prepareClient = function() {
    if (this.error) {
        return;
    }

    this.current = 4;

    var this_ = this;
    window.onerror = function(msg, url, line, col, err) {
        this_.error = msg;
    };

    this.client = new window.OutpostClient();
    this.client.loadData(this.data_blob, function() {
        console.log('loaded data', this_.onload);
        if (this_.onload != null) {
            this_.onload();
        }
    });
};



/** @constructor */
function LauncherUI(launcher) {
    this.launcher = launcher;

    this.img_banner = null;
    this.img_font = null;
    this.metrics = null;
    this.pending = 0;

    this.canvas = null;
    this.ctx = null;
    this.start_time = null;
}

function autoResize(canvas) {
    function handleResize() {
        if (canvas.parentNode == null) {
            window.removeEventListener('resize', handleResize);
            return;
        }
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
    }
    window.addEventListener('resize', handleResize);
    handleResize();
}

LauncherUI.prototype.init = function() {
    var this_ = this;

    this.pending = 3;

    this.img_banner = new Image();
    this.img_banner.onload = function() { this_._finishedPending(); };
    this.img_banner.src = LOGO_BAR_URL;

    this.img_font = new Image();
    this.img_font.onload = function() { this_._finishedPending(); };
    this.img_font.src = 'font.png';

    var xhr = new XMLHttpRequest();
    xhr.open('GET', 'font_metrics.json', true);
    xhr.responseType = 'json';
    xhr.onload = function(evt) {
        this_.metrics = evt.target.response;
        this_._finishedPending();
    };
    xhr.send();
};

LauncherUI.prototype._finishedPending = function() {
    --this.pending;
    if (this.pending == 0) {
        this.initCanvas();
    }
};

LauncherUI.prototype.initCanvas = function() {
    this.canvas = document.createElement('canvas');
    this.canvas.style.position = 'absolute';
    this.canvas.style.top = '0px';
    this.canvas.style.left = '0px';
    document.body.insertBefore(this.canvas, document.body.firstChild);
    autoResize(this.canvas);

    var i = document.getElementById('i');
    i.parentNode.removeChild(i);

    this.ctx = this.canvas.getContext('2d');
    this.ctx.imageSmoothingEnabled = false;
    this.ctx.mozImageSmoothingEnabled = false;
    this.ctx.webkitImageSmoothingEnabled = false;

    this.start_time = Date.now();
    this.renderFrame();
};

function calcBannerPos() {
    var w = window.innerWidth;
    var h = window.innerHeight;
    var px = Math.max(w, h);
    var scale = Math.max(1, Math.round(px / 1024));
    var x = Math.floor(w / 2 / scale) - 80;
    var y = Math.floor(h / 2 / scale) - 60;
    // NB: x and y are pre-scaling coordinates
    return {x: x, y: y, scale: scale};
}

function findGlyph(fm, gm, c) {
    if (gm[c] == null) {
        var code = c.charCodeAt(0);
        var glyph = -1;
        for (var j = 0; j < fm['spans'].length; ++j) {
            var span = fm['spans'][j];
            if (span[0] <= code && code < span[1]) {
                glyph = span[2] + (code - span[0]);
                break;
            }
        }
        gm[c] = glyph;
    }

    return gm[c];
}

function measureString(fm, s, gm) {
    var glyph_map = gm || {};
    var x = 0;
    for (var i = 0; i < s.length; ++i) {
        var c = s.charAt(i);

        var glyph = findGlyph(fm, glyph_map, c);
        if (glyph == -1) {
            x += fm['space_width'];
        } else {
            x += fm['widths2'][glyph] + (i > 0 ? fm['spacing'] : 0);
        }
    }

    return x;
}

function drawString(ctx, fm, img, s, base_x, base_y, gm) {
    var glyph_map = gm || {};
    var dest_x = 0;
    var height = fm['height'];

    for (var i = 0; i < s.length; ++i) {
        var c = s.charAt(i);

        var glyph = findGlyph(fm, glyph_map, c);
        if (glyph == -1) {
            dest_x += fm['space_width'];
        } else {
            var src_x = fm['xs2'][glyph];
            var width = fm['widths2'][glyph];

            ctx.drawImage(img,
                    src_x, 0, width, height,
                    dest_x + base_x, base_y, width, height);

            dest_x += width + (i > 0 ? fm['spacing'] : 0);
        }
    }
}

LauncherUI.prototype.renderFrame = function() {
    if (this.ctx == null) {
        // The canvas has been handed off to the OutpostClient.
        return;
    }

    var this_ = this;
    window.requestAnimationFrame(function() { this_.renderFrame(); });

    var ctx = this.ctx;

    var banner = calcBannerPos();
    ctx.save();
    ctx.scale(banner.scale, banner.scale);
    ctx.translate(banner.x, banner.y);
    ctx.clearRect(0, 0, 160, 120);
    ctx.drawImage(this.img_banner, 0, 0);

    var bar_x0 = 19;
    var bar_x1 = 141;
    var bar_y = 105;

    var frac = this.launcher.getProgress();
    var width = ((bar_x1 - bar_x0) * frac)|0;
    ctx.fillStyle = '#dad45e';
    ctx.fillRect(bar_x0, bar_y, width, 2);
    ctx.fillStyle = '#6daa2c';
    ctx.fillRect(bar_x0, bar_y + 2, width, 2);

    var gm = {};
    var msg = this.launcher.getMessage();
    var width = measureString(this.metrics, msg, gm);
    var msg_y = 118;
    ctx.clearRect(-160, msg_y, 480, this.metrics['height']);
    drawString(ctx, this.metrics, this.img_font,
            msg, 80 - (width / 2)|0, msg_y, gm);

    ctx.restore();
};



function launch(url) {
    var l = new Launcher(url);
    var ui = new LauncherUI(l);

    ui.init();

    l.onload = function() {
        delete window['L'];
        window['C'] = l.client;
        l.client.handoff(ui.canvas, l.conn);
    };

    l.start();
    window['L'] = l;
}

// TODO: read #s=... part of URL to find the server
if (!location.hash.startsWith('#s=')) {
    location.replace('serverlist.html');
}

launch(location.hash.substr(3));
