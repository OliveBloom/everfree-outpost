#ifndef OUTPOST_TEST_TERRAIN_GEN_FFI_INCLUDED
#define OUTPOST_TEST_TERRAIN_GEN_FFI_INCLUDED

#include <stddef.h>
#include <stdint.h>

typedef struct tg_generator tg_generator;
typedef struct tg_chunk tg_chunk;
typedef struct tg_structure tg_structure;
typedef struct tg_extra_iter tg_extra_iter;
typedef struct tg_drawing tg_drawing;

typedef uint16_t block_id;
typedef uint32_t template_id;

tg_generator* generator_create(const char* path);
void generator_destroy(tg_generator* g);
tg_chunk* generator_generate_chunk(tg_generator* g, uint64_t pid, int32_t x, int32_t y);
tg_drawing* generator_test(tg_generator* g, uint64_t pid, int32_t x, int32_t y);

void chunk_free(tg_chunk* c);
size_t chunk_blocks_len(const tg_chunk* c);
block_id chunk_get_block(const tg_chunk* c, size_t idx);
size_t chunk_structures_len(const tg_chunk* c);
const tg_structure* chunk_get_structure(const tg_chunk* c, size_t idx);

void structure_get_pos(const tg_structure* s, int32_t* x_p, int32_t* y_p, int32_t* z_p);
template_id structure_get_template(const tg_structure* s);
size_t structure_extra_len(const tg_structure* s);
tg_extra_iter* structure_extra_iter(const tg_structure* s);

void extra_iter_free(tg_extra_iter* i);
int extra_iter_next(tg_extra_iter* i,
        const char** key_p, size_t* key_len_p,
        const char** value_p, size_t* value_len_p);

void drawing_free(tg_drawing* d);
void drawing_get_size(tg_drawing* d, uint32_t* width_p, uint32_t* height_p);
const uint8_t* drawing_get_height_map(tg_drawing* d);
size_t drawing_get_point_count(tg_drawing* d);
void drawing_get_point(tg_drawing* d,
        size_t i,
        int32_t* x_p, int32_t* y_p,
        const char** color_p, size_t* color_len_p);
size_t drawing_get_line_count(tg_drawing* d);
void drawing_get_line(tg_drawing* d,
        size_t i,
        int32_t* x0_p, int32_t* y0_p,
        int32_t* x1_p, int32_t* y1_p,
        const char** color_p, size_t* color_len_p);



#endif // OUTPOST_TEST_TERRAIN_GEN_FFI_INCLUDED
