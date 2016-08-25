var Config = require('config').Config;
var insertFree = require('util/misc').insertFree;

/** @constructor */
function AsmGl() {
    this.gl = null;
    this.glext_draw_buffers = null;
    this.assets = null;

    this.buffer_list = [];
    this.shader_list = [];
    this.shader_uniform_list = [];
    this.texture_list = [];
    this.framebuffer_list = [];
    this.renderbuffer_list = [];
}
exports.AsmGl = AsmGl;

AsmGl.prototype._getExtension = function(name) {
    if (Config.debug_block_webgl_extensions.get()[name]) {
        return null;
    }
    return this.gl.getExtension(name);
};

AsmGl.prototype.init = function(gl, assets) {
    this.gl = gl;
    this.glext_draw_buffers = this._getExtension('WEBGL_draw_buffers');
    this.assets = assets;
};

AsmGl.prototype.hasExtension = function(name) {
    return this._getExtension(name) != null;
};

// Buffers

AsmGl.prototype.genBuffer = function() {
    var handle = this.gl.createBuffer();

    var name = insertFree(this.buffer_list, handle);

    // 0 is not a valid name.
    return name + 1;
};

AsmGl.prototype.deleteBuffer = function(name) {
    this.buffer_list[name - 1] = null;
    // After this, the buffer handle is eligible for garbage collection.
};

AsmGl.prototype.getBuffer = function(name) {
    return this.buffer_list[name - 1];
};

AsmGl.prototype._bufferTarget = function(target_idx) {
    switch (target_idx) {
        case 0: return this.gl.ARRAY_BUFFER;
        case 1: return this.gl.ELEMENT_ARRAY_BUFFER;
        default: throw 'bad buffer target: ' + target_idx;
    }
};

AsmGl.prototype.bindBuffer = function(target_idx, name) {
    var gl = this.gl;
    var target = this._bufferTarget(target_idx);
    var handle = this.buffer_list[name - 1];
    gl.bindBuffer(target, handle);
};

AsmGl.prototype.bufferDataAlloc = function(target_idx, len) {
    var gl = this.gl;
    var target = this._bufferTarget(target_idx);
    gl.bufferData(target, len, gl.STATIC_DRAW);
};

AsmGl.prototype.bufferSubdata = function(target_idx, offset, data) {
    var gl = this.gl;
    var target = this._bufferTarget(target_idx);
    gl.bufferSubData(target, offset, data);
};


// Shaders

function reportShaderErrors(errs, filename) {
    var lines = errs.split('\n');
    for (var i = 0; i < lines.length; ++i) {
        window.onerror(lines[i], filename, 0, 0, null);
    }
};

AsmGl.prototype._compileShader = function(type, filename, defs) {
    var gl = this.gl;
    var src = defs + this.assets[filename];
    var this_ = this;
    src = src.replace(/^#include "(.*)"$/m, function(_, file) {
        return this_.assets[file];
    });

    var s = gl.createShader(type);
    gl.shaderSource(s, src);
    gl.compileShader(s);

    var log = gl.getShaderInfoLog(s);
    if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
        reportShaderErrors(log, filename);
        console.warn('SHADER ERRORS (' + filename + '):\n' + log);
    } else if (log != '') {
        console.warn('SHADER WARNINGS (' + filename + '):\n' + log);
    }

    return s;
};

AsmGl.prototype._linkProgram = function(vert, frag, name) {
    var gl = this.gl;

    var p = gl.createProgram();
    gl.attachShader(p, vert);
    gl.attachShader(p, frag);
    gl.linkProgram(p);

    var log = gl.getProgramInfoLog(p);
    if (!gl.getProgramParameter(p, gl.LINK_STATUS)) {
        reportShaderErrors(log, name);
        console.warn('PROGRAM ERRORS (' + name + '):\n' + log);
    } else if (log != '') {
        console.warn('PROGRAM WARNINGS (' + name + '):\n' + log);
    }

    gl.detachShader(p, vert);
    gl.detachShader(p, frag);

    return p;
};

AsmGl.prototype.loadShader = function(vert_name, frag_name, defs) {
    var gl = this.gl;

    var vert = this._compileShader(gl.VERTEX_SHADER, vert_name, defs);
    var frag = this._compileShader(gl.FRAGMENT_SHADER, frag_name, defs);
    var program = this._linkProgram(vert, frag, vert_name + '+' + frag_name);
    gl.deleteShader(vert);
    gl.deleteShader(frag);

    var name = insertFree(this.shader_list, program);
    return name + 1;
};

AsmGl.prototype.deleteShader = function(name) {
    this.shader_list[name - 1] = null;
    // TODO: should also delete uniform locations associated with this shader
    // (fortunately we don't actually delete shaders at the moment)
};

AsmGl.prototype.bindShader = function(name) {
    var shader = this.shader_list[name - 1];
    this.gl.useProgram(shader);
};

AsmGl.prototype.getUniformLocation = function(shader_name, var_name) {
    var shader = this.shader_list[shader_name - 1];
    var handle = this.gl.getUniformLocation(shader, var_name);
    if (handle == null) {
        return -1;
    } else {
        var loc = insertFree(this.shader_uniform_list, handle);
        // NB: 0 *is* a valid uniform location
        return loc;
    }
};

AsmGl.prototype.getAttribLocation = function(shader_name, var_name) {
    var shader = this.shader_list[shader_name - 1];
    return this.gl.getAttribLocation(shader, var_name);
};

AsmGl.prototype.setUniform1i = function(loc, value) {
    if (loc == -1) {
        return;
    }
    var handle = this.shader_uniform_list[loc];
    this.gl.uniform1i(handle, value);
};

AsmGl.prototype.setUniform1f = function(loc, value) {
    if (loc == -1) {
        return;
    }
    var handle = this.shader_uniform_list[loc];
    this.gl.uniform1f(handle, value);
};

AsmGl.prototype.setUniform2f = function(loc, value) {
    if (loc == -1) {
        return;
    }
    var handle = this.shader_uniform_list[loc];
    this.gl.uniform2fv(handle, value);
};

AsmGl.prototype.setUniform3f = function(loc, value) {
    if (loc == -1) {
        return;
    }
    var handle = this.shader_uniform_list[loc];
    this.gl.uniform3fv(handle, value);
};

AsmGl.prototype.setUniform3f = function(loc, value) {
    if (loc == -1) {
        return;
    }
    var handle = this.shader_uniform_list[loc];
    this.gl.uniform3fv(handle, value);
};


// Textures

AsmGl.prototype.loadTexture = function(name, size_out) {
    var gl = this.gl;
    var img = this.assets[name];

    var tex = gl.createTexture();
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, img);

    size_out[0] = img.width;
    size_out[1] = img.height;

    var name = insertFree(this.texture_list, tex);
    return name + 1;
};

AsmGl.prototype.genTexture = function(width, height, kind) {
    var gl = this.gl;

    var tex = gl.createTexture();
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    this.textureImage(width, height, kind, null);

    var name = insertFree(this.texture_list, tex);
    return name + 1;
};

AsmGl.prototype.importTextureHACK = function(tex) {
    var name = insertFree(this.texture_list, tex);
    return name + 1;
};

AsmGl.prototype.deleteTexture = function(name) {
    this.texture_list[name - 1] = null;
};

AsmGl.prototype.activeTexture = function(unit) {
    var gl = this.gl;
    gl.activeTexture(gl.TEXTURE0 + unit);
};

AsmGl.prototype.bindTexture = function(name) {
    var gl = this.gl;
    var tex = this.texture_list[name - 1];
    gl.bindTexture(gl.TEXTURE_2D, tex);
};

AsmGl.prototype.textureImage = function(width, height, kind, data) {
    var gl = this.gl;
    switch (kind) {
        case 0:
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, width, height, 0,
                    gl.RGBA, gl.UNSIGNED_BYTE, data);
            break;
        case 1:
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.DEPTH_COMPONENT, width, height, 0,
                    gl.DEPTH_COMPONENT, gl.UNSIGNED_INT, data);
            break;
        case 2:
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.LUMINANCE, width, height, 0,
                    gl.LUMINANCE, gl.UNSIGNED_BYTE, data);
            break;
        default:
            throw 'bad texture kind: ' + kind;
    }
};

AsmGl.prototype.textureSubimage = function(off_x, off_y, width, height, kind, data) {
    var gl = this.gl;
    switch (kind) {
        case 0:
            gl.texSubImage2D(gl.TEXTURE_2D, 0, off_x, off_y, width, height,
                    gl.RGBA, gl.UNSIGNED_BYTE, data);
            break;
        case 1:
            gl.texSubImage2D(gl.TEXTURE_2D, 0, off_x, off_y, width, height,
                    gl.DEPTH_COMPONENT, gl.UNSIGNED_INT, data);
            break;
        case 2:
            gl.texSubImage2D(gl.TEXTURE_2D, 0, off_x, off_y, width, height,
                    gl.LUMINANCE, gl.UNSIGNED_BYTE, data);
            break;
        default:
            throw 'bad texture kind: ' + kind;
    }
};


// Framebuffers

AsmGl.prototype.genFramebuffer = function() {
    var fb = this.gl.createFramebuffer();
    var name = insertFree(this.framebuffer_list, fb);
    return name + 1;
};

AsmGl.prototype.deleteFramebuffer = function(name) {
    this.framebuffer_list[name - 1] = null;
};

AsmGl.prototype.bindFramebuffer = function(name) {
    var gl = this.gl;
    var fb = this.framebuffer_list[name - 1];
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
};

AsmGl.prototype.genRenderbuffer = function(width, height, is_depth) {
    var gl = this.gl;

    var rb = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, rb);
    if (!is_depth) {
        // Only crappy RGBA formats are supported in webgl 1.  Fortunately we
        // don't actually use color renderbuffers at the moment.
        gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGB5_A1, width, height);
    } else {
        gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH_COMPONENT16, width, height);
    }

    var name = insertFree(this.renderbuffer_list, rb);
    return name + 1;
};

AsmGl.prototype.deleteRenderbuffer = function(name) {
    this.renderbuffer_list[name - 1] = null;
};

AsmGl.prototype._fbAttachment = function(att) {
    if (att >= 0) {
        return this.gl.COLOR_ATTACHMENT0 + att;
    } else if (att == -1) {
        return this.gl.DEPTH_ATTACHMENT;
    } else {
        throw 'bad attachment index: ' + att;
    }
};

AsmGl.prototype.framebufferTexture = function(tex_name, attachment) {
    var gl = this.gl;

    var tex = this.texture_list[tex_name - 1];
    var attach = this._fbAttachment(attachment);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, attach, gl.TEXTURE_2D, tex, 0);
};

AsmGl.prototype.framebufferRenderbuffer = function(rb_name, attachment) {
    var gl = this.gl;

    var rb = this.renderbuffer_list[rb_name - 1];
    var attach = this._fbAttachment(attachment);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, attach, gl.RENDERBUFFER, rb);
};

AsmGl.prototype.checkFramebufferStatus = function() {
    var gl = this.gl;
    var stat = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
    return stat == gl.FRAMEBUFFER_COMPLETE;
};

AsmGl.prototype.drawBuffers = function(num_attachments) {
    var gl = this.gl;
    var arr = new Array(num_attachments);
    for (var i = 0; i < arr.length; ++i) {
        arr[i] = gl.COLOR_ATTACHMENT0 + i;
    }
    this.glext_draw_buffers.drawBuffersWEBGL(arr);
};


// Drawing

AsmGl.prototype.viewport = function(x, y, w, h) {
    this.gl.viewport(x, y, w, h);
};

AsmGl.prototype.clearColor = function(r, g, b, a) {
    this.gl.clearColor(r, g, b, a);
};

AsmGl.prototype.clearDepth = function(d) {
    this.gl.clearDepth(d);
};

AsmGl.prototype.clear = function() {
    var gl = this.gl;
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
};

AsmGl.prototype.setDepthTest = function(mode) {
    var gl = this.gl;
    switch (mode) {
        case 0:
            gl.disable(gl.DEPTH_TEST);
            break;
        case 1:
            gl.enable(gl.DEPTH_TEST);
            gl.depthFunc(gl.GEQUAL);
            break;
        case 2:
            gl.enable(gl.DEPTH_TEST);
            gl.depthFunc(gl.ALWAYS);
            break;
    }
};

AsmGl.prototype.setBlendMode = function(mode) {
    var gl = this.gl;
    switch (mode) {
        case 0:     // None
            gl.disable(gl.BLEND);
            break;
        case 1:     // Alpha
            gl.enable(gl.BLEND);
            gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
            break;
        case 2:     // Add
            gl.enable(gl.BLEND);
            gl.blendFunc(gl.ONE, gl.ONE);
            break;
        case 3:     // MultiplyInv
            gl.enable(gl.BLEND);
            gl.blendFunc(gl.ONE_MINUS_DST_COLOR, gl.ONE);
            break;
    }
};

AsmGl.prototype.enableVertexAttribArray = function(index) {
    this.gl.enableVertexAttribArray(index);
};

AsmGl.prototype.disableVertexAttribArray = function(index) {
    this.gl.disableVertexAttribArray(index);
};

AsmGl.prototype._attribType = function(ty) {
    switch (ty) {
        case 0: return this.gl.UNSIGNED_BYTE;
        case 1: return this.gl.UNSIGNED_SHORT;
        case 2: return this.gl.UNSIGNED_INT;
        case 3: return this.gl.BYTE;
        case 4: return this.gl.SHORT;
        case 5: return this.gl.INT;
        default: throw 'bad attrib type: ' + ty;
    }
};

AsmGl.prototype.vertexAttribPointer = function(loc, count, ty, normalize, stride, offset) {
    var gl_ty = this._attribType(ty);
    this.gl.vertexAttribPointer(loc, count, gl_ty, normalize, stride, offset);
};

AsmGl.prototype.drawArraysTriangles = function(start, count) {
    this.gl.drawArrays(this.gl.TRIANGLES, start, count);
};
