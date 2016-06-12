#include "game_backend.hpp"
#include "opcode.hpp"
#include "server.hpp"

using namespace std;


void game_backend::handle_message() {
    message msg(header_buf.client_id, header_buf.opcode, move(data_buf));
    owner.handle_game_response(move(msg));
}

void game_backend::handle_shutdown() {
    if (restarting) {
        restarting = false;
        start();
        resume();
    } else {
        owner.handle_game_shutdown();
    }
}

void game_backend::write(message msg) {
    backend::write(msg);

    if (msg.client_id == 0 &&
            (msg.opcode == opcode::OP_RESTART_SERVER ||
             msg.opcode == opcode::OP_RESTART_BOTH)) {
        restarting = true;
        suspend();
    }
}
