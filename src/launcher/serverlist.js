
function buildServerEntry(info) {
    var div = document.createElement('div');
    var img = document.createElement('img');
    var link = document.createElement('a');

    div.classList.add('server-entry');
    img.src = ARROW_URL;
    img.classList.add('server-arrow');
    link.textContent = info.name;
    link.href = 'launcher.html#s=' + info.url;
    link.classList.add('server-link');

    div.appendChild(img);
    div.appendChild(link);
    return div;
}

function initServerList() {
    var list = document.getElementById('server-list');

    var h1 = document.createElement('h1');
    h1.textContent = 'Server List';
    list.appendChild(h1);

    for (var i = 0; i < BUILTIN_SERVERS.length; ++i) {
        var info = BUILTIN_SERVERS[i];
        list.appendChild(buildServerEntry(info));
    }

    // TODO: add custom servers
}

function startLoginCheck() {
    var xhr = new XMLHttpRequest();
    xhr.open('POST', new URL('api/check_login', AUTH_URL).href, true);
    xhr.responseType = 'json';
    xhr.withCredentials = true;

    xhr.onload = function(evt) {
        if (xhr.status == 200) {
            finishLoginCheck(xhr.response);
        } else {
            finishLoginCheck(null);
        }
    };
    xhr.onerror = function(evt) {
        finishLoginCheck(null);
    };

    xhr.send(JSON.stringify({'auto_guest': true}));
}

function finishLoginCheck(result) {
    var stat = document.getElementById('login-status');
    if (result == null) {
        stat.textContent = 'Error checking login';
        return;
    }

    var link = '<a href="' + new URL('login', AUTH_URL).href + '">Log In</a>'
    if (result.type == 'none') {
        stat.innerHTML = 'Not logged in &ndash; ' + link;
    } else if (result.type == 'guest') {
        stat.innerHTML = 'Using guest account ' + result['name'] + ' &ndash; ' + link;
    } else if (result.type == 'normal') {
        stat.innerHTML = 'Logged in as ' + result['name'];
    }
}

function init() {
    startLoginCheck();
    initServerList();
}

document.addEventListener('DOMContentLoaded', init);

