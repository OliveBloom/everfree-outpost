#include "auth_backend.hpp"
#include "opcode.hpp"
#include "server.hpp"

using namespace std;


void auth_backend::handle_message() {
    message msg(header_buf.client_id, header_buf.opcode, move(data_buf));
    owner.handle_auth_response(move(msg));
}

void auth_backend::handle_shutdown() {
    // TODO: try to recover automatically
    owner.handle_auth_shutdown();
}

void auth_backend::write(message msg) {
    backend::write(msg);
}
