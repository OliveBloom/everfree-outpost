var Config = require('config').Config;

var TIMER_RANGE = 0x10000;
var TIMER_MASK = 0xffff;

/** @constructor */
function Timing(asm, conn) {
    this.asm = asm;
    this.conn = conn;

    this.client_send = 0;
    this.first_ping = true;

    var this_ = this;
    this.conn.onPong = function(cookie, s, cr) { this_._handlePong(s, cr); };
    this.update();
}
exports.Timing = Timing;

Timing.prototype.update = function() {
    this.client_send = Date.now();
    this.conn.sendPing(this.client_send & TIMER_MASK);
};

Timing.prototype.scheduleUpdates = function(delay, interval) {
    var this_ = this;
    function callback() {
        this_.update();
        setTimeout(callback, interval * 1000);
    }

    setTimeout(callback, delay * 1000);
};

Timing.prototype._handlePong = function(server_time, client_recv_raw) {
    // TODO: on firefox, event.timeStamp appears to be in microseconds instead
    // of milliseconds.  Just use Date.now() instead.
    var client_recv = Date.now();

    this.asm.handlePong(this.client_send, client_recv, server_time);
};

Timing.prototype.encodeSend = function(server) {
    return server & TIMER_MASK;
}
