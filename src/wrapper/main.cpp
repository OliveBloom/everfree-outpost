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

#ifndef _WIN32
    local::stream_protocol::endpoint control_addr("control");
    local::stream_protocol::endpoint repl_addr("repl");
#else
    ip::tcp::endpoint control_addr(ip::address_v4::loopback(), 8890);
    ip::tcp::endpoint repl_addr(ip::address_v4::loopback(), 8891);
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
             8888);

    ios.run();
}
