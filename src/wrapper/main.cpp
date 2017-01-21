#include <climits>
#include <boost/asio.hpp>
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/ini_parser.hpp>

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

    boost::property_tree::ptree config;
    boost::property_tree::read_ini("outpost.ini", config);
    string host_str = config.get("network.bind_addr", "");
    string port_str = config.get("network.bind_port", "");


    int port;
    if (port_str != "") {
        // Parse the port number provided by the user.
        const char* port_str_begin = port_str.c_str();
        char* port_str_end = NULL;
        auto raw_port = strtol(port_str_begin, &port_str_end, 10);
        if (port_str_end - port_str_begin != port_str.size()) {
            cerr << "error parsing bind_port" << endl;
            return 1;
        }
        if (raw_port < 1 || raw_port > 65535) {
            cerr << "bind_port is out of range (1-65535)" << endl;
            return 1;
        }
        port = raw_port;
    } else {
        // Use the default port, 8888
        port = 8888;
    }

    boost::asio::ip::tcp::endpoint ws_addr;
    if (host_str != "") {
        // Bind on the indicated address
        boost::system::error_code ec;
        auto host = boost::asio::ip::address::from_string(host_str, ec);
        if (ec) {
            cerr << "error parsing bind_addr: " << ec << endl;
            return 1;
        }
        ws_addr = boost::asio::ip::tcp::endpoint(host, port);
    } else {
        // Bind on IPv6 wildcard address (also handles IPv4)
        ws_addr = boost::asio::ip::tcp::endpoint(boost::asio::ip::tcp::v6(), port);
    }

    cout << "listening on " << ws_addr << endl;


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
             ws_addr);

    ios.run();
}
