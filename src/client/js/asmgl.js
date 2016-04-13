var glutil = require('graphics/glutil');
var insertFree = require('util/misc').insertFree;

/** @constructor */
function AsmGl() {
    this.gl = null;
    this.assets = null;

    this.buffer_list = [];
    this.shader_list = [];
    this.shader_uniform_list = [];
    this.texture_list = [];
}
exports.AsmGl = AsmGl;

AsmGl.prototype.init = function(gl, assets) {
    this.gl = gl;
    this.assets = assets;
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

AsmGl.prototype.getBufferWrapper = function(name) {
    return new glutil.Buffer(this.gl, this.buffer_list[name - 1]);
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

AsmGl.prototype._compileShader = function(type, filename) {
    var gl = this.gl;
    var src = this.assets[filename];

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

AsmGl.prototype.loadShader = function(vert_name, frag_name) {
    var gl = this.gl;

    var vert = this._compileShader(gl.VERTEX_SHADER, vert_name);
    var frag = this._compileShader(gl.FRAGMENT_SHADER, frag_name);
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
        return handle;
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

AsmGl.prototype.deleteTexture = function(name) {
    this.texture_list[name - 1] = null;
};

AsmGl.prototype.activeTexture = function(unit) {
    this.gl.activeTexture(gl.TEXTURE0 + unit);
};

AsmGl.prototype.bindTexture = function(name) {
    var tex = this.gl.texture_list[name - 1];
    this.gl.bindTexture(gl.TEXTURE_2D, tex);
};


// Drawing

AsmGl.prototype.enableVertexAttribArray = function(index) {
    this.gl.enableVertexAttribArray(index);
};

AsmGl.prototype.disableVertexAttribArray = function(index) {
    this.gl.disableVertexAttribArray(index);
};

AsmGl.prototype.vertexAttribPointer = function(loc, count, ty, normalize, stride, offset) {
    this.gl.vertexAttribPointer(loc, count, ty, normalize, stride, offset);
};

AsmGl.prototype.drawArraysTriangles = function(start, count) {
    this.gl.drawArrays(this.gl.TRIANGLES, start, count);
};
