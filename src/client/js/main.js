var $ = document.getElementById.bind(document);


var loader = require('loader');
var Vec = require('util/vec').Vec;
var Config = require('config').Config;

var AnimCanvas = require('canvas').AnimCanvas;
var OffscreenContext = require('canvas').OffscreenContext;
var handleResize = require('resize').handleResize;

var Motion = require('entity').Motion;

var InventoryTracker = require('inventory').InventoryTracker;

var Keyboard = require('keyboard').Keyboard;
var Dialog = require('ui/dialog').Dialog;
var Banner = require('ui/banner').Banner;
var ChatWindow = require('ui/chat').ChatWindow;
var ContainerUI = require('ui/inventory').ContainerUI;
var CraftingUI = require('ui/crafting').CraftingUI;
var Iframe = require('ui/iframe').Iframe;
var KeyDisplay = require('ui/keydisplay').KeyDisplay;
var Menu = require('ui/menu').Menu;
var ConfigEditor = require('ui/configedit').ConfigEditor;
var MusicTest = require('ui/musictest').MusicTest;
var PonyEditor = require('ui/ponyedit').PonyEditor;
var KeybindingEditor = require('ui/keybinding').KeybindingEditor;
var widget = require('ui/widget');
var ErrorList = require('ui/errorlist').ErrorList;
var InventoryUpdateList = require('ui/invupdate').InventoryUpdateList;
var DIALOG_TYPES = require('ui/dialogs').DIALOG_TYPES;
var DNDState = require('ui/dnd').DNDState;

var Input = require('input').Input;

var ItemDef = require('data/items').ItemDef;
var RecipeDef = require('data/recipes').RecipeDef;

var LOCAL_SIZE = require('consts').LOCAL_SIZE;

var DynAsm = require('asmlibs').DynAsm;
var AsmClientInput = require('asmlibs').AsmClientInput;

var net = require('net');
var Timing = require('time').Timing;

var buildArray = require('util/misc').buildArray;
var checkBrowser = require('util/browser').checkBrowser;
var util = require('util/misc');


// Client objects

var asm_client;

var assets;

var conn;

var canvas;
var ui_div;
var dialog;
var banner;
var keyboard;
var dnd;
var chat;
var error_list;
var inv_update_list;
var music_test;

var input;

var main_menu;
var debug_menu;

var timing;
var inv_tracker;
var synced = net.SYNC_LOADING;
var item_inv;
var ability_inv;


// Top-level initialization function

function init() {
    // Set up error_list first to catch errors in other parts of init.
    error_list = new ErrorList();
    error_list.attach(window);
    document.body.appendChild(error_list.container);

    canvas = new AnimCanvas(frame, 'webgl', [
            'WEBGL_depth_texture',
            'WEBGL_draw_buffers',
    ]);

    asm_client = new DynAsm();

    ui_div = util.element('div', ['ui-container']);
    banner = new Banner();
    keyboard = new Keyboard(asm_client);
    dnd = new DNDState(keyboard);
    dialog = new Dialog(keyboard);
    chat = new ChatWindow();
    inv_update_list = new InventoryUpdateList();
    music_test = new MusicTest();

    input = new Input();
    input.handlers.push(new AsmClientInput(asm_client));

    canvas.canvas.addEventListener('webglcontextlost', function(evt) {
        throw 'context lost!';
    });

    initMenus();

    assets = null;

    conn = null;    // Initialized after assets are loaded.
    timing = null;  // Initialized after connection is opened.

    item_inv = null;
    ability_inv = null;


    buildUI();

    checkBrowser(dialog, function() {
        loadAssets(function() {
            asm_client.initClient(canvas.ctx, assets);

            // This should only happen after client init.
            function doResize() {
                handleResize(canvas, ui_div, window.innerWidth, window.innerHeight);
                asm_client.resizeWindow(window.innerWidth, window.innerHeight);
            }
            window.addEventListener('resize', doResize);
            doResize();


            var info = assets['server_info'];
            openConn(info, function() {
                timing = new Timing(conn);
                timing.scheduleUpdates(5, 30);
                inv_tracker = new InventoryTracker(conn, asm_client);
                asm_client.conn = conn;

                maybeRegister(info, function() {
                    conn.sendLogin(Config.login_name.get(), Config.login_secret.get());

                    // Show "Loading World..." banner.
                    handleSyncStatus(net.SYNC_LOADING);
                    canvas.start();
                });
            });
        });
    });
}

document.addEventListener('DOMContentLoaded', init);


// Major initialization steps.

function loadAssets(next) {
    loader.loadJson('server.json', function(server_info) {
        // TODO: remove this hack since it prevents all caching
        loader.loadPack('outpost.pack?' + Date.now(), function(loaded, total) {
            banner.update('Loading... (' + (loaded >> 10)+ 'k / ' + (total >> 10) + 'k)', loaded / total);
        }, function(assets_) {
            assets = assets_;
            assets['server_info'] = server_info;

            var items = assets['item_defs'];
            for (var i = 0; i < items.length; ++i) {
                ItemDef.register(i, items[i]);
            }

            var recipes = assets['recipe_defs'];
            for (var i = 0; i < recipes.length; ++i) {
                RecipeDef.register(i, recipes[i]);
            }

            var css = '.item-icon {' +
                'background-image: url("' + assets['items'] + '");' +
            '}';
            util.element('style', ['type=text/css', 'text=' + css], document.head);

            next();
        });
    });
}

function openConn(info, next) {
    var url = info['url'];
    if (url == null) {
        var elt = util.element('div', []);
        elt.innerHTML = info['message'];
        var w = new widget.Template('server-offline', {'msg': elt});
        var f = new widget.Form(w);
        f.oncancel = function() {};
        dialog.show(f);
        return;
    }

    banner.update('Connecting to server...', 0);
    conn = new net.Connection(url);
    conn.onOpen = next;
    conn.onClose = handleClose;
    conn.onInit = handleInit;
    conn.onTerrainChunk = handleTerrainChunk;
    conn.onEntityUpdate = handleEntityUpdate;
    conn.onUnloadChunk = handleUnloadChunk;
    conn.onOpenDialog = handleOpenDialog;
    conn.onOpenCrafting = handleOpenCrafting;
    conn.onChatUpdate = handleChatUpdate;
    conn.onEntityAppear = handleEntityAppear;
    conn.onEntityGone = handleEntityGone;
    conn.onStructureAppear = handleStructureAppear;
    conn.onStructureGone = handleStructureGone;
    conn.onMainInventory = handleMainInventory;
    conn.onAbilityInventory = handleAbilityInventory;
    conn.onPlaneFlags = handlePlaneFlags;
    conn.onGetInteractArgs = handleGetInteractArgs;
    conn.onGetUseItemArgs = handleGetUseItemArgs;
    conn.onGetUseAbilityArgs = handleGetUseAbilityArgs;
    conn.onSyncStatus = handleSyncStatus;
    conn.onStructureReplace = handleStructureReplace;
}

function maybeRegister(info, next) {
    if (Config.login_name.isSet() && Config.login_secret.isSet() &&
            Config.world_version.get() == info['world_version']) {
        console.log('secret already set');
        next();
        return;
    }

    var default_name = Config.login_name.get() || generateName();
    var secret = makeSecret();

    var editor = new PonyEditor(default_name, drawPony);

    var last_name = null;

    function send_register(name, app_info) {
        editor.onfinish = null;
        editor.setMessage("Registering...");
        last_name = name;

        var appearance = calcAppearance(app_info);
        saveAppearance(app_info);
        conn.onRegisterResult = handle_result;
        conn.sendRegister(name,
                          secret,
                          appearance);
    }

    function handle_result(code, msg) {
        conn.onRegisterResult = null;
        if (code == 0) {
            Config.login_name.set(last_name);
            Config.login_secret.set(secret);
            Config.world_version.set(info['world_version']);
            dialog.hide();
            next();
        } else {
            editor.setError(code, msg);
            editor.onfinish = send_register;
        }
    }

    editor.onsubmit = send_register;
    editor.oncancel = function() {};
    dialog.show(editor);
}


// Initialization helpers

function buildUI() {
    keyboard.attach(document);
    input.attach(document);
    setupKeyHandler();

    var key_list = $('key-list');

    ui_div.appendChild(key_list);
    ui_div.appendChild(chat.container);
    ui_div.appendChild(inv_update_list.container);
    ui_div.appendChild(banner.container);
    ui_div.appendChild(dialog.container);

    if (Config.show_key_display.get()) {
        var key_display = new KeyDisplay();
        ui_div.appendChild(key_display.container);
        keyboard.monitor = function(down, evt) {
            if (down) {
                key_display.onKeyDown(evt);
            } else {
                key_display.onKeyUp(evt);
            }
        };
    }

    if (!Config.show_controls.get()) {
        key_list.classList.add('hidden');
    }

    banner.show('Loading...', 0, keyboard, function() { return false; });

    document.body.appendChild(canvas.canvas);
    document.body.appendChild(ui_div);
}

function initMenus() {
    main_menu = new Menu([
            ['&Instructions', function() {
                dialog.show(new widget.Form(new Iframe('instructions.html', keyboard)));
            }],
            ['&Keyboard Controls', function() {
                dialog.show(new KeybindingEditor(keyboard));
            }],
            ['&Debug Menu', function() { dialog.show(debug_menu); }],
            ['&Credits', function() {
                dialog.show(new widget.Form(new Iframe('credits.html', keyboard)));
            }],
    ]);

    debug_menu = new Menu([
            ['&Config Editor', function() { dialog.show(new ConfigEditor()); }],
            ['&Music Test', function() { dialog.show(music_test); }],
    ]);
}

function generateName() {
    var number = '' + Math.floor(Math.random() * 10000);
    while (number.length < 4) {
        number = '0' + number;
    }

    return "Anon" + number;
}

function makeSecret() {
    console.log('producing secret');
    var secret_buf = [0, 0, 0, 0];
    if (window.crypto.getRandomValues) {
        var typedBuf = new Uint32Array(4);
        window.crypto.getRandomValues(typedBuf);
        for (var i = 0; i < 4; ++i) {
            secret_buf[i] = typedBuf[i];
        }
    } else {
        console.log("warning: window.crypto.getRandomValues is not available.  " +
                "Login secret will be weak!");
        for (var i = 0; i < 4; ++i) {
            secret_buf[i] = Math.floor(Math.random() * 0xffffffff);
        }
    }
    return secret_buf;
}

function calcAppearance(a) {
    var appearance =
        (a.eyes << 16) |
        (a.tail << 13) |
        (a.mane << 10) |
        (a.sex << 8) |
        (a.tribe << 6) |
        (a.red << 4) |
        (a.green << 2) |
        (a.blue << 0);

    return appearance;
}

function saveAppearance(a) {
    Config.last_appearance.set({
        'eyes': a.eyes,
        'tail': a.tail,
        'mane': a.mane,
        'sex': a.sex,
        'tribe': a.tribe,
        'red': a.red,
        'green': a.green,
        'blue': a.blue,
    });
}

function drawPony(ctx, app_info) {
    var app = calcAppearance(app_info);
    asm_client.ponyeditRender(app);

    ctx.clearRect(0, 0, 96, 96);
    var size = Math.round(96 * canvas.canvas.width / canvas.virtualWidth);
    var extra = Math.round(40 * canvas.canvas.width / canvas.virtualWidth);
    var cx = ((canvas.canvas.width - size) / 2)|0;
    var cy = ((canvas.canvas.height - size - extra) / 2)|0;
    ctx.drawImage(canvas.canvas, cx, cy, size, size, 0, 0, 96, 96);
}



var INPUT_LEFT =    0x0001;
var INPUT_RIGHT =   0x0002;
var INPUT_UP =      0x0004;
var INPUT_DOWN =    0x0008;
var INPUT_RUN =     0x0010;

var ACTION_USE =        1;
var ACTION_INVENTORY =  2;
var ACTION_USE_ITEM =   3;

function setupKeyHandler() {
    var dirs_held = {
        'move_up': false,
        'move_down': false,
        'move_left': false,
        'move_right': false,
        'run': false,
    };

    keyboard.pushHandler(function(down, evt) {
        if (down && evt.repeat) {
            return true;
        }

        var shouldStop = alwaysStop(evt);

        var binding = Config.keybindings.get()[evt.keyCode];
        if (binding == null || evt.ctrlKey || evt.altKey || evt.metaKey) {
            return shouldStop;
        }

        if (dirs_held.hasOwnProperty(binding)) {
            dirs_held[binding] = down;
            updateWalkDir();
            return true;
        } else if (down) {
            var time = timing.encodeSend(timing.nextArrival());

            switch (binding) {
                // UI actions
                case 'show_controls':
                    var show = Config.show_controls.toggle();
                    $('key-list').classList.toggle('hidden', !show);
                    break;
                case 'debug_test':
                    window.hideUI = !window.hideUI;
                    break;
                case 'chat':
                    chat.startTyping(keyboard, conn, '');
                    break;
                case 'chat_command':
                    chat.startTyping(keyboard, conn, '/');
                    break;
                case 'show_menu':
                    dialog.show(main_menu);
                    break;
                case 'toggle_cursor':
                    asm_client.toggleCursor();
                    break;

                case 'inventory':
                    if (item_inv == null) {
                        break;
                    }
                    asm_client.openInventoryDialog();
                    break;

                case 'abilities':
                    if (ability_inv == null) {
                        break;
                    }
                    asm_client.openAbilityDialog();
                    break;

                // Commands to the server
                case 'interact':
                    conn.sendInteract(time);
                    break;
                case 'use_item':
                    conn.sendUseItem(time, asm_client.getActiveItem());
                    break;
                case 'use_ability':
                    conn.sendUseAbility(time, asm_client.getActiveAbility());
                    break;

                default:
                    return shouldStop;
            }

            return true;
        } else {
            return shouldStop;
        }
    });

    function updateWalkDir() {
        var bits = 0;
        var target_velocity = new Vec(0, 0, 0);

        if (dirs_held['move_left']) {
            bits |= INPUT_LEFT;
            target_velocity.x -= 1;
        }
        if (dirs_held['move_right']) {
            bits |= INPUT_RIGHT;
            target_velocity.x += 1;
        }

        if (dirs_held['move_up']) {
            bits |= INPUT_UP;
            target_velocity.y -= 1;
        }
        if (dirs_held['move_down']) {
            bits |= INPUT_DOWN;
            target_velocity.y += 1;
        }

        if (dirs_held['run']) {
            bits |= INPUT_RUN;
            target_velocity = target_velocity.mulScalar(150);
        } else {
            target_velocity = target_velocity.mulScalar(50);
        }

        var arrival = timing.nextArrival() + Config.input_delay.get();
        conn.sendInput(timing.encodeSend(arrival), bits);

        asm_client.feedInput(arrival, target_velocity);
    }

    function alwaysStop(evt) {
        // Allow Ctrl + anything
        if (evt.ctrlKey) {
            return false;
        }
        // Allow F5-F12
        if (evt.keyCode >= 111 + 5 && evt.keyCode <= 111 + 12) {
            return false;
        }

        // Stop all other events.
        return true;
    }
}


// Connection message callbacks

function handleClose(evt, reason) {
    var reason_elt = document.createElement('p');
    if (reason != null) {
        reason_elt.textContent = 'Reason: ' + reason;
    }

    var w = new widget.Template('disconnected', {'reason': reason_elt});
    var f = new widget.Form(w);
    f.oncancel = function() {};
    dialog.show(f);
}

function handleInit(entity_id, now, cycle_base, cycle_ms) {
    asm_client.setPawnId(entity_id);
    var pst_now = timing.decodeRecv(now);
    asm_client.initDayNight(pst_now - cycle_base, cycle_ms);
}

function handleTerrainChunk(i, data) {
    var cx = (i % LOCAL_SIZE)|0;
    var cy = (i / LOCAL_SIZE)|0;
    asm_client.loadTerrainChunk(cx, cy, data);
}

function handleEntityUpdate(id, motion, anim) {
    var m = new Motion(motion.start_pos);
    m.end_pos = motion.end_pos;

    var now = timing.visibleNow();
    m.start_time = timing.decodeRecv(motion.start_time);
    m.end_time = timing.decodeRecv(motion.end_time);
    if (m.start_time > now + 2000) {
        m.start_time -= 0x10000;
    }
    if (m.end_time < m.start_time) {
        m.end_time += 0x10000;
    }

    m.anim_id = anim;

    asm_client.entityUpdate(id, m, anim);
}

function handleUnloadChunk(idx) {
}

function handleOpenDialog(idx, args) {
    if (idx == 0) {
        // Cancel server-side subscription.
        inv_tracker.unsubscribe(args[0]);
    } else if (idx == 1) {
        var inv1 = inv_tracker.get(args[0]);
        var inv2 = inv_tracker.get(args[1]);

        var ui = new ContainerUI(dnd, inv1, inv2);
        dialog.show(ui);
        ui.ontransfer = function(from_inventory, from_slot, to_inventory, to_slot, amount) {
            conn.sendMoveItem(from_inventory, from_slot, to_inventory, to_slot, amount);
        };

        ui.oncancel = function() {
            dialog.hide();
            inv1.unsubscribe();
            inv2.unsubscribe();
        };
    }
}

function handleOpenCrafting(station_type, station_id, inventory_id) {
    var inv = inv_tracker.get(inventory_id);

    var ui = new CraftingUI(station_type, station_id, inv, ability_inv);
    dialog.show(ui);

    ui.onaction = function(station_id, inventory_id, recipe_id, count) {
        conn.sendCraftRecipe(station_id, inventory_id, recipe_id, count);
    };

    ui.oncancel = function() {
        dialog.hide();
        inv.unsubscribe();
    };
}

function handleChatUpdate(msg) {
    chat.addMessage(msg);
}

function handleEntityAppear(id, appearance_bits, name) {
    asm_client.entityAppear(id, appearance_bits, name);
}

function handleEntityGone(id, time) {
    // TODO: actually delay until the specified time
    asm_client.entityGone(id);
}

function handleStructureAppear(id, template_id, x, y, z) {
    var now = timing.visibleNow();
    asm_client.structureAppear(id, x, y, z, template_id, now);
}

function handleStructureGone(id, time) {
    // TODO: pay attention to the time
    asm_client.structureGone(id);
}

function handleStructureReplace(id, template_id) {
    var now = timing.visibleNow();
    asm_client.structureReplace(id, template_id, now);
}

function handleMainInventory(iid) {
    if (item_inv != null) {
        item_inv.unsubscribe();
    }
    item_inv = inv_tracker.get(iid);
    if (Config.show_inventory_updates.get()) {
        inv_update_list.attach(item_inv.clone());
    }

    asm_client.inventoryMainId(iid);
}

function handleAbilityInventory(iid) {
    if (ability_inv != null) {
        ability_inv.unsubscribe();
    }
    ability_inv = inv_tracker.get(iid);

    asm_client.inventoryAbilityId(iid);
}

function handlePlaneFlags(flags) {
    asm_client.setPlaneFlags(flags);
}

function handleGetInteractArgs(dialog_id, parts) {
    handleGenericGetArgs(dialog_id, parts, function(time, args) {
        conn.sendInteractWithArgs(time, args);
    });
}

function handleGetUseItemArgs(item_id, dialog_id, parts) {
    handleGenericGetArgs(dialog_id, parts, function(time, args) {
        conn.sendUseItemWithArgs(time, item_id, args);
    });
}

function handleGetUseAbilityArgs(item_id, dialog_id, parts) {
    handleGenericGetArgs(dialog_id, parts, function(time, args) {
        conn.sendUseAbilityWithArgs(time, item_id, args);
    });
}

function handleGenericGetArgs(dialog_id, parts, cb) {
    var d = new (DIALOG_TYPES[dialog_id])(parts);
    d.onsubmit = function(args) {
        dialog.hide();
        var time = timing.encodeSend(timing.nextArrival());
        cb(time, args);
    };
    dialog.show(d);
}

function handleSyncStatus(new_synced) {
    synced = new_synced;
    if (synced == net.SYNC_REFRESH) {
        window.location.reload(true);
    } else if (synced == net.SYNC_LOADING) {
        banner.show('Loading World...', 0, keyboard, function() { return false; });
    } else if (synced == net.SYNC_RESET) {
        banner.show('Server restarting...', 0, keyboard, function() { return false; });

        if (synced == net.SYNC_RESET) {
            resetAll();
        }
    } else {
        banner.hide();
    }
}

// Reset (nearly) all client-side state to pre-login conditions.
function resetAll() {
    inv_tracker.reset();
    item_inv = null;
    ability_inv = null;

    if (dialog.isVisible()) {
        dialog.hide();
    }

    asm_client.resetClient();
}


// Rendering

function frame(ac, client_now) {
    if (synced != net.SYNC_OK) {
        return;
    }

    var now = timing.visibleNow();
    var future = now + timing.ping;
    asm_client.renderFrame(now, future);

    var frame_time = timing.visibleNow() - now;
    asm_client.debugRecord(frame_time, timing.ping);
}
