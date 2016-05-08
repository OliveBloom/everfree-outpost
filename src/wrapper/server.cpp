#include "opcode.hpp"
#include "server.hpp"
#include <cstdlib>

using namespace std;
using namespace boost::asio;


server::server(io_service& ios,
               const char* backend_path,
               platform::local_stream::endpoint control_addr,
               platform::local_stream::endpoint repl_addr,
               uint16_t ws_port)
    : backend_(new backend(*this, ios, backend_path)),
      control_(new control(*this, ios, control_addr)),
      repl_(new repl(*this, ios, repl_addr)),
      signals_(new signals(*this, ios)),
      websocket_(new websocket(*this, ios, ws_port)),
      restarting_(false) {
    backend_->start();
}

void server::handle_backend_response(message msg) {
    if (msg.client_id == 0) {
        if (msg.opcode == opcode::OP_CLIENT_REMOVED) {
            assert(msg.data.size() == 2);
            websocket_->handle_client_removed(*(uint16_t*)&msg.data[0]);
        } else if (msg.opcode == opcode::OP_REPL_RESULT) {
            repl_->handle_response(msg.data.begin(), msg.data.end());
        } else {
            cerr << "BUG: bad opcode from backend: " << hex << msg.opcode << dec << endl;
        }
    } else {
        websocket_->send_message(move(msg));
    }
}

void server::handle_backend_shutdown() {
    if (restarting_) {
        restarting_ = false;
        backend_->start();
        backend_->resume();
    } else {
        exit(0);
    }
}

void server::handle_repl_command(vector<uint8_t> command) {
    backend_->write(message(0, opcode::OP_REPL_COMMAND, move(command)));
}

void server::handle_control_command(uint16_t op) {
    vector<uint8_t> command(0);
    backend_->write(message(0, op, move(command)));

    if (op == opcode::OP_RESTART_SERVER || op == opcode::OP_RESTART_BOTH) {
        restarting_ = true;
        backend_->suspend();
    }
}

void server::handle_websocket_connect(uint16_t client_id) {
    vector<uint8_t> data(2);
    *(uint16_t*)&data[0] = client_id;
    backend_->write(message(0, opcode::OP_ADD_CLIENT, move(data)));
}

void server::handle_websocket_disconnect(uint16_t client_id) {
    vector<uint8_t> data(2);
    *(uint16_t*)&data[0] = client_id;
    backend_->write(message(0, opcode::OP_REMOVE_CLIENT, move(data)));
}

void server::handle_websocket_request(message msg) {
    backend_->write(move(msg));
}
