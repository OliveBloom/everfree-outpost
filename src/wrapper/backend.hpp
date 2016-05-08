#ifndef OUTPOST_WRAPPER_BACKEND_HPP
#define OUTPOST_WRAPPER_BACKEND_HPP

#include <boost/asio.hpp>
#include <vector>

#include "message.hpp"
#include "platform.hpp"


class server;

class backend {
    char** command;
    platform::child_stream pipe_to;
    platform::child_stream pipe_from;

    std::vector<message> pending_msgs;

    void read_header();
    void read_data();

protected:
    server& owner;

    header header_buf;
    std::vector<uint8_t> data_buf;

    bool suspended;
    bool restarting;

    void suspend();
    void resume();

    virtual void handle_message() = 0;
    virtual void handle_shutdown() = 0;

public:
    backend(server& owner,
            boost::asio::io_service& ios,
            char** command);
    virtual ~backend() {}

    void start();

    void write(message msg);
};

#endif // OUTPOST_WRAPPER_BACKEND_HPP
