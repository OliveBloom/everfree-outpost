#ifndef OUTPOST_WRAPPER_GAME_BACKEND_HPP
#define OUTPOST_WRAPPER_GAME_BACKEND_HPP

#include <boost/asio.hpp>

#include "backend.hpp"
#include "message.hpp"

class server;

class game_backend : public backend {
protected:
    virtual void handle_message();
    virtual void handle_shutdown();

public:
    game_backend(server& owner,
                 boost::asio::io_service& ios,
                 char** command) : backend(owner, ios, command) {}

    void write(message msg);
};

#endif // OUTPOST_WRAPPER_GAME_BACKEND_HPP
