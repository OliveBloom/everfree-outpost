#ifndef OUTPOST_WRAPPER_BACKEND_HPP
#define OUTPOST_WRAPPER_BACKEND_HPP

#include <boost/asio.hpp>
#include <vector>

#include "message.hpp"
#include "platform.hpp"


class server;

class backend {
    server& owner;
    const char* backend_path;
    platform::child_stream pipe_to;
    platform::child_stream pipe_from;

    header header_buf;
    std::vector<uint8_t> data_buf;

    bool suspended;
    std::vector<message> pending_msgs;

    void read_header();
    void read_data();
    void handle_message();
    void handle_shutdown();

public:
    backend(server& owner,
            boost::asio::io_service& ios,
            const char* backend_path);

    void start();

    void write(message msg);

    void suspend();
    void resume();
};

#endif // OUTPOST_WRAPPER_BACKEND_HPP
