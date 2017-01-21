#include "opcode.hpp"
#include "server.hpp"
#include <cstdlib>

using namespace std;
using namespace boost::asio;


void server::remove_client(uint16_t client_id) {
    websocket_->handle_client_removed(client_id);
    client_authed.erase(client_id);
}

void server::dispatch_backend(message msg) {
    uint16_t client_id = msg.client_id;
    dispatch_backend(move(msg), client_id);
}

void server::dispatch_backend(message msg, uint16_t client_id) {
    auto auth_iter = client_authed.find(client_id);
    if (auth_iter == client_authed.end()) {
        cerr << "BUG: dispatching message for client " << client_id <<
            ", but that client is not in client_authed" << endl;
        return;
    }

    bool authed = auth_iter->second;
    if (authed) {
        game_backend_->write(move(msg));
    } else {
        auth_backend_->write(move(msg));
    }
}


server::server(io_service& ios,
               char** game_command,
               char** auth_command,
               platform::local_stream::endpoint control_addr,
               platform::local_stream::endpoint repl_addr,
               boost::asio::ip::tcp::endpoint ws_addr)
    : game_backend_(new game_backend(*this, ios, game_command)),
      auth_backend_(new auth_backend(*this, ios, auth_command)),
      control_(new control(*this, ios, control_addr)),
      repl_(new repl(*this, ios, repl_addr)),
      signals_(new signals(*this, ios)),
      websocket_(new websocket(*this, ios, ws_addr)) {
    game_backend_->start();
    auth_backend_->start();
}


void server::handle_game_response(message msg) {
    if (msg.client_id == 0) {
        if (msg.opcode == opcode::OP_CLIENT_REMOVED) {
            assert(msg.data.size() == 2);
            remove_client(*(uint16_t*)&msg.data[0]);
        } else if (msg.opcode == opcode::OP_REPL_RESULT) {
            repl_->handle_response(msg.data.begin(), msg.data.end());
        } else {
            cerr << "BUG: bad opcode from game: " << hex << msg.opcode << dec << endl;
        }
    } else {
        websocket_->send_message(move(msg));
    }
}

void server::handle_game_shutdown() {
    exit(0);
}


void server::handle_auth_response(message msg) {
    if (msg.client_id == 0) {
        if (msg.opcode == opcode::OP_CLIENT_REMOVED) {
            assert(msg.data.size() == 2);
            remove_client(*(uint16_t*)&msg.data[0]);
        } else if (msg.opcode == opcode::OP_AUTH_DONE) {
            assert(msg.data.size() >= 2);
            uint16_t real_client_id = *(uint16_t*)&msg.data[0];
            client_authed[real_client_id] = true;

            game_backend_->write(message(0, opcode::OP_ADD_CLIENT, move(msg.data)));
        } else {
            cerr << "BUG: bad opcode from backend: " << hex << msg.opcode << dec << endl;
        }
    } else {
        websocket_->send_message(move(msg));
    }
}

void server::handle_auth_shutdown() {
    cerr << "auth backend crashed" << endl;
    exit(0);
}


void server::handle_repl_command(vector<uint8_t> command) {
    game_backend_->write(message(0, opcode::OP_REPL_COMMAND, move(command)));
}

void server::handle_control_command(uint16_t op) {
    vector<uint8_t> command(0);
    game_backend_->write(message(0, op, move(command)));
}


void server::handle_websocket_connect(uint16_t client_id) {
    client_authed.insert(make_pair(client_id, false));

    vector<uint8_t> data(2);
    *(uint16_t*)&data[0] = client_id;
    auth_backend_->write(message(0, opcode::OP_ADD_CLIENT, move(data)));
}

void server::handle_websocket_disconnect(uint16_t client_id) {
    vector<uint8_t> data(2);
    *(uint16_t*)&data[0] = client_id;
    dispatch_backend(message(0, opcode::OP_REMOVE_CLIENT, move(data)), client_id);

    client_authed.erase(client_id);
}

void server::handle_websocket_request(message msg) {
    dispatch_backend(move(msg));
}
