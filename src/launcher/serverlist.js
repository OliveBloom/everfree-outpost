
function makeLink(href, text, cls) {
    var link = document.createElement('a');
    link.textContent = text;
    link.href = href;
    if (cls) {
        link.classList.add(cls);
    }
    return link
}

function buildServerEntry(info) {
    var div = document.createElement('div');

    div.classList.add('server-entry');


    var img = document.createElement('img');
    var link = makeLink('launcher.html#s=' + info.url, info.name, 'server-link');

    img.src = ARROW_URL;
    img.classList.add('server-arrow');

    div.appendChild(img);
    div.appendChild(link);


    var more = document.createElement('div');
    var toggle = makeLink('#', 'more', 'server-more-toggle');

    toggle.onclick = function() { more.classList.toggle('active'); };
    more.classList.add('server-more');
    more.appendChild(makeLink('launcher.html#r=configedit;s=' + info.url, 'Settings'));

    div.appendChild(toggle);
    div.appendChild(more);

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

    function link(url, text) {
        return '<a href="' + new URL(url, AUTH_URL).href + '">' + text + '</a>'
    }
    if (result.type == 'none') {
        stat.innerHTML = 'Using guest account &ndash; ' + link('login', 'Log In');
    } else if (result.type == 'normal') {
        stat.innerHTML = 'Logged in as ' + result['name'] +
            ' &ndash; ' + link('logout', 'Log Out');
    }
}

function init() {
    startLoginCheck();
    initServerList();
}

document.addEventListener('DOMContentLoaded', init);

