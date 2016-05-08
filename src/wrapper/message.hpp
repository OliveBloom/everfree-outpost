#ifndef OUTPOST_WRAPPER_MESSAGE_HPP
#define OUTPOST_WRAPPER_MESSAGE_HPP

#include <vector>

struct message {
    uint16_t client_id;
    uint16_t opcode;
    std::vector<uint8_t> data;

    message() {}
    message(uint16_t client_id, uint16_t opcode, std::vector<uint8_t> data)
        : client_id(client_id), opcode(opcode), data(move(data)) {}
};

struct header {
    uint16_t client_id;
    uint16_t data_len;
    uint16_t opcode;
};

#endif // OUTPOST_WRAPPER_MESSAGE_HPP
