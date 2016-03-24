var glutil = require('graphics/glutil');

/** @constructor */
function AsmGl() {
    this.gl = null;

    this.buffer_list = [];
}
exports.AsmGl = AsmGl;

AsmGl.prototype.init = function(gl) {
    this.gl = gl;
};


AsmGl.prototype.genBuffer = function() {
    var handle = this.gl.createBuffer();

    var name = this.buffer_list.indexOf(null);
    if (name == -1) {
        name = this.buffer_list.length;
        this.buffer_list.push(handle);
    } else {
        this.buffer_list[name] = handle;
    }

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

AsmGl.prototype.bindBufferArray = function(name) {
    var gl = this.gl;
    var handle = this.buffer_list[name - 1];
    gl.bindBuffer(gl.ARRAY_BUFFER, handle);
};

AsmGl.prototype.bindBufferIndex = function(name) {
    var gl = this.gl;
    var handle = this.buffer_list[name - 1];
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, handle);
};

AsmGl.prototype.bufferDataAlloc = function(len) {
    var gl = this.gl;
    gl.bufferData(gl.ARRAY_BUFFER, len, gl.STATIC_DRAW);
};

AsmGl.prototype.bufferSubdata = function(offset, data) {
    var gl = this.gl;
    gl.bufferSubData(gl.ARRAY_BUFFER, offset, data);
};
