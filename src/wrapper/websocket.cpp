#include "server.hpp"
#include "websocket.hpp"

#include <iostream>

using namespace std;
// Avoid conflict between std and boost placeholders.
namespace ph = std::placeholders;
using namespace boost::asio;
using websocketpp::connection_hdl;


websocket::websocket(server& owner, boost::asio::io_service& ios, uint16_t port)
    : owner(owner),
      ws_server(),
      next_id(1),
      id_to_client(),
      clients() {
    ws_server.init_asio(&ios);
    ws_server.set_reuse_addr(true);

    ws_server.set_open_handler(bind(&websocket::handle_open, this, ph::_1));
    ws_server.set_message_handler(bind(&websocket::handle_message, this, ph::_1, ph::_2));
    ws_server.set_close_handler(bind(&websocket::handle_close, this, ph::_1));

    ws_server.listen(port);
    ws_server.start_accept();
}

void websocket::handle_open(connection_hdl conn) {
    while (next_id == 0 || id_to_client.count(next_id)) {
        ++next_id;
    }

    uint16_t id = next_id++;
    id_to_client.insert(make_pair(id, conn));
    client_data data;
    data.id = id;
    clients.insert(make_pair(conn, data));

    owner.handle_websocket_connect(id);
}

void websocket::handle_message(connection_hdl conn, ws_server_asio::message_ptr msg) {
    auto data_iter = clients.find(conn);
    if (data_iter == clients.end()) {
        return;
    }
    auto& data(data_iter->second);

    if (!data.backend_connected) {
        return;
    }

    const string& payload = msg->get_payload();
    if (payload.size() < 2) {
        cerr << "client " << data.id << ": message has no opcode" << endl;
        // TODO: send a kick message and shut down
        return;
    }

    auto begin = payload.begin();
    auto end = payload.end();
    uint16_t opcode = *(uint16_t*)&*begin;
    vector<uint8_t> msg_data(begin + 2, end);

    owner.handle_websocket_request(message(data.id, opcode, move(msg_data)));
}

void websocket::handle_close(connection_hdl conn) {
    auto data_iter = clients.find(conn);
    if (data_iter == clients.end()) {
        return;
    }
    auto& data(data_iter->second);

    data.client_connected = false;
    if (data.dead()) {
        id_to_client.erase(data.id);
        clients.erase(data_iter);
    } else {
        // Shut down the backend side as well.
        owner.handle_websocket_disconnect(data.id);
    }
}

void websocket::send_message(message msg) {
    auto conn_iter = id_to_client.find(msg.client_id);
    if (conn_iter == id_to_client.end()) {
        return;
    }
    auto& conn(conn_iter->second);

    auto data_iter = clients.find(conn);
    if (data_iter == clients.end()) {
        return;
    }
    auto& data(data_iter->second);

    if (!data.client_connected) {
        return;
    }

    vector<uint8_t> buf;
    buf.reserve(2 + msg.data.size());
    buf.resize(2);
    *(uint16_t*)&buf[0] = msg.opcode;
    buf.insert(buf.end(), msg.data.begin(), msg.data.end());

    std::error_code ec;
    ws_server.send(conn, buf.data(), buf.size(), websocketpp::frame::opcode::binary, ec);
    if (ec) {
        cerr << "error sending to " << msg.client_id << ": " << ec << endl;
    }
}

void websocket::handle_client_removed(uint16_t client_id) {
    auto conn_iter = id_to_client.find(client_id);
    if (conn_iter == id_to_client.end()) {
        return;
    }
    auto& conn(conn_iter->second);

    auto data_iter = clients.find(conn);
    if (data_iter == clients.end()) {
        return;
    }
    auto& data(data_iter->second);

    data.backend_connected = false;
    if (data.dead()) {
        id_to_client.erase(data.id);
        clients.erase(data_iter);
    } else {
        // Shut down the client connection as well.
        std::error_code ec;
        ws_server.close(conn, websocketpp::close::status::normal, "", ec);
        if (ec) {
            cerr << "error closing " << client_id << ": " << ec << endl;
        }
        // NB: handle_close may have invalidated one or both of the iterators.
    }
}
