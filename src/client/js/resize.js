var Config = require('config').Config;

function calcScale(px) {
    var target = 1024;
    if (px < target) {
        return -Math.round(target / px);
    } else {
        return Math.round(px / target);
    }
}

function resizeUI(ui_div, w, h) {
    var scale1 = Config.scale_world.get() || calcScale(Math.max(w, h));
    var factor1 = scale1 > 0 ? scale1 : 1 / -scale1;

    // scale_ui is now applied as a multiple of scale_world.
    var scale2 = Config.scale_ui.get() || 1;
    var factor2 = scale2 > 0 ? scale2 : 1 / -scale2;

    var factor = factor1 * factor2;

    var virt_w = Math.ceil(w / factor);
    var virt_h = Math.ceil(h / factor);

    ui_div.style.width = virt_w + 'px';
    ui_div.style.height = virt_h + 'px';
    ui_div.style.transform = 'scale(' + factor + ')';

    document.body.dataset['uiScale'] = factor;
}

function handleResize(anim_canvas, ui_div, w, h) {
    resizeUI(ui_div, w, h);
}
exports.handleResize = handleResize;
