#ifndef OUTPOST_WRAPPER_SERVER_HPP
#define OUTPOST_WRAPPER_SERVER_HPP

#include <boost/asio.hpp>
#include <memory>
#include <string>
#include <vector>

#include "game_backend.hpp"
#include "control.hpp"
#include "message.hpp"
#include "repl.hpp"
#include "signals.hpp"
#include "websocket.hpp"


class server {
    std::unique_ptr<game_backend> game_backend_;
    std::unique_ptr<control> control_;
    std::unique_ptr<repl> repl_;
    std::unique_ptr<signals> signals_;
    std::unique_ptr<websocket> websocket_;

public:
    server(boost::asio::io_service& ios,
           char** game_command,
           platform::local_stream::endpoint control_addr,
           platform::local_stream::endpoint repl_addr,
           uint16_t ws_port);

    void handle_backend_response(message msg);
    void handle_backend_shutdown();
    void handle_repl_command(std::vector<uint8_t> command);
    void handle_control_command(uint16_t opcode);

    void handle_websocket_connect(uint16_t client_id);
    void handle_websocket_disconnect(uint16_t client_id);
    void handle_websocket_request(message msg);
};

#endif // OUTPOST_WRAPPER_SERVER_HPP
