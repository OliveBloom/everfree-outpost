var Config = require('config').Config;
var ConfigEditor = require('ui/configedit').ConfigEditor;

function $(x) { return document.getElementById(x); }

function init() {
    $('show-inventory-updates').addEventListener('change', function() {
        var value = $('show-inventory-updates').checked;
        Config.show_inventory_updates.set(value);
    });
    $('show-inventory-updates').checked = Config.show_inventory_updates.get();

    $('open-editor').addEventListener('click', function() {
        $('open-editor').disabled = true;
        var editor = new ConfigEditor();
        document.body.appendChild(editor.dom);
    });
    // Firefox saves the 'disabled' setting across refresh.
    $('open-editor').disabled = false;
}

document.addEventListener('DOMContentLoaded', init);
