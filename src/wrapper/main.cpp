#include <boost/asio.hpp>

#include "server.hpp"

using namespace std;
using namespace boost::asio;
using boost::system::error_code;


void append_arg(vector<char>& buf, vector<size_t>& idxs, const char* arg) {
    string s(arg);
    idxs.push_back(buf.size());
    buf.insert(buf.end(), s.begin(), s.end());
    buf.push_back('\0');
}

void build_cmd(char* base, vector<size_t>& idxs, vector<char*>& cmd) {
    for (size_t i : idxs) {
        cmd.push_back(base + i);
    }
    cmd.push_back(NULL);
}

int main(int argc, char *argv[]) {
    io_service ios;


    const char* port_str = getenv("OUTPOST_PORT");
    int port = 8888;
    if (port_str != NULL) {
        char* port_str_end = NULL;
        port = strtol(port_str, &port_str_end, 10);
        if (*port_str == '\0' || *port_str_end != '\0') {
            cerr << "invalid setting for OUTPOST_PORT" << endl;
            return 1;
        }
    }


#ifndef _WIN32
    local::stream_protocol::endpoint control_addr("control");
    local::stream_protocol::endpoint repl_addr("repl");
#else
    ip::tcp::endpoint control_addr(ip::address_v4::loopback(), port + 1);
    ip::tcp::endpoint repl_addr(ip::address_v4::loopback(), port + 2);
#endif


    vector<char> game_buf;
    vector<size_t> game_idxs;
    vector<char*> game_cmd;
    append_arg(game_buf, game_idxs, "bin/backend");
    append_arg(game_buf, game_idxs, ".");
    build_cmd(&game_buf[0], game_idxs, game_cmd);

    vector<char> auth_buf;
    vector<size_t> auth_idxs;
    vector<char*> auth_cmd;
    append_arg(auth_buf, auth_idxs, "python3");
    append_arg(auth_buf, auth_idxs, "bin/auth.py");
    build_cmd(&auth_buf[0], auth_idxs, auth_cmd);

    server s(ios,
             &game_cmd[0],
             &auth_cmd[0],
             control_addr,
             repl_addr,
             port);

    ios.run();
}
