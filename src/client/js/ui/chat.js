var util = require('util/misc');
var Config = require('config').Config;

/** @constructor */
function ChatWindow() {
    var lines = Config.chat_lines.get();
    // Font size is 0.7rem.  Add a little bit extra to cover line spacing.
    var height = (lines * 0.85) + 'rem';

    var parts = util.templateParts('chat-panel');
    this.container = parts['top'];
    this._content = parts['content'];
    this._content.style.height = height;
    this._entry = parts['entry'];

    if (Config.chat_autohide.get()) {
        this.container.style.display = 'none';
    }

    this.count = 0;
}
exports.ChatWindow = ChatWindow;

ChatWindow.prototype.addMessage = function(msg) {
    var parts = msg.split('\t');
    if (parts.length != 3) {
        console.assert(false, 'msg is malformed', msg);
        return;
    }

    var channel = parts[0];
    var name = parts[1];
    var text = parts[2];
    if (Config.ignores.get()[name]) {
        return;
    }

    var parts = util.templateParts('chat-line');
    parts['name'].textContent = name;
    parts['text'].textContent = text;

    if (channel == '&s') {
        parts['top'].classList.add('server-message');
    } else if (channel == '&l') {
        parts['top'].classList.add('local-message');
    }


    var was_at_bottom =
        (this._content.scrollTop + this._content.clientHeight >= this._content.scrollHeight);

    this._content.appendChild(parts['top']);

    var limit = Config.chat_scrollback.get();
    if (this.count < limit) {
        this.count += 1;
    } else {
        // Remove the first line of the chat, but adjust the scrolling so the
        // view stays at the same spot.
        var old_h = this._content.scrollHeight;
        this._content.removeChild(this._content.firstChild);
        var new_h = this._content.scrollHeight;
        // new_h < old_h because the current viewport is closer to the top of
        // the entire content area.
        this._content.scrollTop -= old_h - new_h;
    }

    // If the chat box was scrolled to the bottom, automatically scroll with
    // new lines.  Otherwise, don't scroll, so the player can read the old
    // messages unhindered.
    if (was_at_bottom) {
        this._content.scrollTop = this._content.scrollHeight;
    }
};

ChatWindow.prototype.addIgnore = function(name) {
    var ignores = Config.ignores.get();
    ignores['<' + name + '>'] = true;
    Config.ignores.save();
};

ChatWindow.prototype.removeIgnore = function(name) {
    var ignores = Config.ignores.get();
    delete ignores['<' + name + '>'];
    Config.ignores.save();
};

ChatWindow.prototype.startTyping = function(keyboard, conn, init) {
    var this_ = this;

    if (Config.chat_autohide.get()) {
        this.container.style.display = 'flex';
    }

    this._entry.disabled = false;
    this._entry.value = init;
    this._entry.focus();
    this._entry.selectionStart = init.length;

    keyboard.pushHandler(function(down, evt) {
        if (document.activeElement !== this_._entry) {
            this_._entry.focus();
        }
        if (!down) {
            return false;
        }

        var binding = Config.chat_keybindings.get()[evt.keyCode];

        switch (binding) {
            case 'send':
                this_.finishTyping(keyboard, conn, true);
                return true;
            case 'cancel':
                this_.finishTyping(keyboard, conn, false);
                return true;
            default:
                return false;
        }
    });
};

ChatWindow.prototype.finishTyping = function(keyboard, conn, send) {
    keyboard.popHandler();

    var msg = this._entry.value;
    var handled = false;
    if (msg[0] == '/') {
        var idx = msg.indexOf(' ');
        if (idx != -1) {
            var cmd = msg.substring(1, idx);
            var arg = msg.substring(idx + 1);
            if (cmd == 'ignore') {
                this.addIgnore(arg);
                handled = true;
            } else if (cmd == 'unignore') {
                this.removeIgnore(arg);
                handled = true;
            }
        }
    }

    if (send && !handled && msg != '') {
        conn.sendChat(msg);
    }

    this._entry.blur();
    this._entry.value = '';
    this._entry.disabled = true;

    var this_ = this;
    if (Config.chat_autohide.get()) {
        window.setTimeout(function() {
            if (!this_._entry.disabled) {
                // User already started typing again.
                return;
            }
            this_.container.style.display = 'none';
        }, 3000);
    }
};
