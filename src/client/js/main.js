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


/** @constructor */
function OutpostClient() {
    this._init();
}

OutpostClient.prototype._init = function() {
    // Set up error_list first to catch errors in other parts of init.
    error_list = new ErrorList();
    error_list.attach(window);
    document.body.appendChild(error_list.container);

    canvas = null;

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

    initMenus();

    assets = null;

    conn = null;    // Initialized after assets are loaded.
    timing = null;  // Initialized after connection is opened.

    item_inv = null;
    ability_inv = null;
};

OutpostClient.prototype.loadData = function(blob, next) {
    var this_ = this;
    loader.loadPack(blob, function(assets_) {
        assets = assets_;

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
};

OutpostClient.prototype.handoff = function(old_canvas, ws) {
    canvas = document.createElement('canvas');
    console.log('orig', old_canvas);

    canvas.addEventListener('webglcontextlost', function(evt) {
        throw 'context lost!';
    });

    asm_client.initClient(canvas.getContext('webgl'), assets);

    // Don't handle any input until the client is inited.
    keyboard.attach(document);
    input.handlers.push(new AsmClientInput(asm_client));

    // This should only happen after client init.
    function doResize() {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
        asm_client.resizeWindow(window.innerWidth, window.innerHeight);
        handleResize(null, ui_div, window.innerWidth, window.innerHeight);
    }
    window.addEventListener('resize', doResize);
    doResize();


    conn = new net.Connection(ws);
    //conn.onOpen = next;   // TODO - probably remove?
    conn.onClose = handleClose;
    conn.onInit = handleInit;
    conn.onTerrainChunk = handleTerrainChunk;
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
    conn.onStructureReplace = handleStructureReplace;
    conn.onEntityMotionStart = handleEntityMotionStart;
    conn.onEntityMotionEnd = handleEntityMotionEnd;
    conn.onEntityMotionStartEnd = handleEntityMotionStartEnd;
    conn.onProcessedInputs = handleProcessedInputs;
    conn.onActivityChange = handleActivityChange;
    conn.onInitNoPawn = handleInitNoPawn;
    conn.onOpenPonyEdit = handleOpenPonyEdit;

    conn.onSyncStatus = function(new_synced) {
        // The first time the status becomes SYNC_OK, swap out the canvas and
        // start the requestAnimationFrame loop.
        if (new_synced == net.SYNC_OK) {
            console.log(old_canvas);
            document.body.replaceChild(canvas, old_canvas);
            console.log('building UI');
            buildUI();
            conn.onSyncStatus = handleSyncStatus;
            window.requestAnimationFrame(frame);
        }
        handleSyncStatus(new_synced);
    };

    conn.sendReady();

    timing = new Timing(conn);
    timing.scheduleUpdates(5, 30);
    inv_tracker = new InventoryTracker(conn, asm_client);
    asm_client.conn = conn;

    // Start the requestAnimationFrame loop
    frame();

    console.log('handoff complete');

    /*
    maybeRegister(info, function() {
        conn.sendLogin(Config.login_name.get(), Config.login_secret.get());

        // Show "Loading World..." banner.
        handleSyncStatus(net.SYNC_LOADING);
        canvas.start();
    });
    */
};

// Initialization helpers

function buildUI() {
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
    var scale = asm_client.calcScale(canvas.width, canvas.height);
    if (scale < 0) {
        scale = 1 / -scale;
    }

    ctx.clearRect(0, 0, 96, 96);
    var size = Math.round(96 * scale);
    var extra = Math.round(40 * scale);
    var cx = ((canvas.width - size) / 2)|0;
    var cy = ((canvas.height - size - extra) / 2)|0;
    ctx.drawImage(canvas, cx, cy, size, size, 0, 0, 96, 96);
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
                    if (!evt.shiftKey) {
                        asm_client.debugExport();
                    } else {
                        asm_client.debugImport();
                    }
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

        asm_client.feedInput(arrival, bits);
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

function handleInitNoPawn(x, y, z, now, cycle_base, cycle_ms) {
    asm_client.setDefaultCameraPos(x, y, z);
    var pst_now = timing.decodeRecv(now);
    asm_client.initDayNight(pst_now - cycle_base, cycle_ms);
}

function handleTerrainChunk(i, data) {
    var cx = (i % LOCAL_SIZE)|0;
    var cy = (i / LOCAL_SIZE)|0;
    asm_client.loadTerrainChunk(cx, cy, data);
}

function handleUnloadChunk(idx) {
}

function handleOpenDialog(idx, args) {
    if (idx == 0) {
        // Cancel server-side subscription.
        inv_tracker.unsubscribe(args[0]);
    } else if (idx == 1) {
        asm_client.openContainerDialog(args[0], args[1]);
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

function handleEntityMotionStart(id, m, anim) {
    var start_time = timing.decodeRecv(m.start_time);
    asm_client.entityMotionStart(id, start_time, m.start_pos, m.velocity, anim);
}

function handleEntityMotionEnd(id, end_time) {
    asm_client.entityMotionEnd(id, timing.decodeRecv(end_time));
}

function handleEntityMotionStartEnd(id, m, anim) {
    handleEntityMotionStart(id, m, anim);
    handleEntityMotionEnd(id, m.end_time);
}

function handleProcessedInputs(time, count) {
    asm_client.processedInputs(timing.decodeRecv(time), count);
}

function handleActivityChange(activity) {
    asm_client.activityChange(activity);
}

function handleOpenPonyEdit(name) {
    var editor = new PonyEditor(name, drawPony);

    function send_register(app_info) {
        var appearance = calcAppearance(app_info);
        saveAppearance(app_info);

        console.log('appearance: ' + appearance.toString(16));
        conn.sendCreateCharacter(appearance);
        dialog.hide();
    }

    editor.onsubmit = send_register;
    editor.oncancel = function() { handleOpenPonyEdit(name); };
    dialog.show(editor);
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

function frame() {
    window.requestAnimationFrame(frame);

    if (synced != net.SYNC_OK) {
        return;
    }

    var now = timing.visibleNow();
    var future = now + timing.ping;
    asm_client.renderFrame(now, future);

    var frame_time = timing.visibleNow() - now;
    asm_client.debugRecord(frame_time, timing.ping);
}
