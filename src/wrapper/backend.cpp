#include "backend.hpp"
#include "opcode.hpp"
#include "server.hpp"

using namespace std;
using namespace boost::asio;


void backend::read_header() {
    async_read(pipe_from, buffer(&header_buf, sizeof(header)),
        [this] (boost::system::error_code ec, size_t len) {
            if (!ec) {
                read_data();
            } else {
                cerr << "error reading header from backend: " << ec << endl;
                handle_shutdown();
            }
        });
}

void backend::read_data() {
    // NB: backend includes opcode in length
    data_buf.resize((size_t)header_buf.data_len - 2);
    async_read(pipe_from, buffer(data_buf),
        [this] (boost::system::error_code ec, size_t len) {
            if (!ec) {
                handle_message();
                read_header();
            } else {
                cerr << "error reading data from backend: " << ec << endl;
                handle_shutdown();
            }
        });
}

void backend::suspend() {
    suspended = true;
}

void backend::resume() {
    suspended = false;
    for (auto&& msg : pending_msgs) {
        write(msg);
    }
    pending_msgs.clear();
}

backend::backend(server& owner,
                 io_service& ios,
                 char** command)
  : owner(owner), command(command), pipe_from(ios), pipe_to(ios),
    suspended(false), restarting(false) {
}

void backend::start() {
    auto fds = platform::spawn_backend(command);
    pipe_from = platform::child_stream(pipe_from.get_io_service(), fds.first);
    pipe_to = platform::child_stream(pipe_to.get_io_service(), fds.second);
    read_header();
}

void backend::write(message msg) {
    if (suspended) {
        pending_msgs.emplace_back(move(msg));
        return;
    }

    auto header_ptr = make_shared<header>();
    header_ptr->client_id = msg.client_id;
    assert(msg.data.size() <= UINT16_MAX);
    // NB: backend includes opcode in length
    header_ptr->data_len = 2 + msg.data.size();
    header_ptr->opcode = msg.opcode;

    auto data_ptr = make_shared<vector<uint8_t>>(move(msg.data));

    array<mutable_buffer, 2> bufs {{
        { &*header_ptr, sizeof(*header_ptr) },
        { &(*data_ptr)[0], data_ptr->size() },
    }};

    async_write(pipe_to, bufs,
        [header_ptr, data_ptr] (boost::system::error_code ec, size_t len) {
            if (ec) {
                cerr << "error writing to backend: " << ec << endl;
                assert(0);
            }
        });
}
