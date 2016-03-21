#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include <sys/mman.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>
#include <unistd.h>


struct FileHeader {
    uint16_t minor;
    uint16_t major;
    uint32_t header_offset;
    uint32_t header_count;
    uint32_t _reserved0;
};

struct SectionHeader {
    uint32_t tag;
    uint32_t offset;
    uint32_t count;
    uint32_t _reserved0;
};

struct CV3 {
    int32_t x;
    int32_t y;
    int32_t z;
};

struct CAttachment {
    uint8_t tag;
    uint32_t data;
};

struct FlatVec {
    uint32_t off;
    uint32_t len;
};

struct FlatExtra {
    uint8_t tag;
    uint8_t a;
    uint16_t b;
    uint32_t data;
};

struct FlatEntity {
    uint64_t stable_plane;

    int64_t motion_start_time;
    uint16_t motion_duration;
    struct CV3 motion_start_pos;
    struct CV3 motion_end_pos;

    uint16_t anim;
    struct CV3 facing;
    struct CV3 target_velocity;
    uint32_t appearance;

    struct FlatExtra extra;
    uint64_t stable_id;
    struct CAttachment attachment;
    struct FlatVec child_inventories;
};

#define TAG(str)    \
    ((str[0] << 0) | \
     (str[1] << 8) | \
     (str[2] << 16) | \
     (str[3] << 24))


void process(char* ptr) {
    struct FileHeader* fhdr = (struct FileHeader*)ptr;
    //printf("  version %d.%d, %d sections\n",
    //        fhdr->major, fhdr->minor, fhdr->header_count);
    if (fhdr->major != 1 || fhdr->minor != 0) {
        printf("  unsupported version (need 1.0), aborting\n");
        return;
    }

    for (int i = 0; i < fhdr->header_count; ++i) {
        struct SectionHeader* shdr = (struct SectionHeader*)
            (ptr + fhdr->header_offset + i * sizeof(struct SectionHeader));
        uint32_t tag = shdr->tag;
        //printf("  section %c%c%c%c: %d entries\n",
        //        (tag >> 0) & 0xff,
        //        (tag >> 8) & 0xff,
        //        (tag >> 16) & 0xff,
        //        (tag >> 24) & 0xff,
        //        shdr->count);

        if (tag == TAG("WEnt")) {
            for (int j = 0; j < shdr->count; ++j) {
                struct FlatEntity* entity = (struct FlatEntity*)
                    (ptr + shdr->offset + j * sizeof(struct FlatEntity));
                //printf("  found entity with appearance %08x\n", entity->appearance);

                int hat = (entity->appearance >> 18) & 0xf;
                if (hat != 0) {
                    // Change equip0 to 4, indicating witch hat.
                    entity->appearance =
                        (entity->appearance & ~(0xf << 18)) |
                        (4 << 18);
                    printf("  * changed hat: %d -> %d\n", hat, 4);
                }
            }
        }
    }
}


int main(int argc, char *argv[]) {
    if (argc != 2) {
        printf("usage: %s <save_dir>\n");
        return 2;
    }

    char buf[256];
    if (snprintf(buf, 256, "%s/clients/", argv[1]) >= 256) {
        printf("save_dir name is too long for buffer\n");
        return 1;
    }

    int dir_fd = open(buf, O_RDONLY | O_DIRECTORY);
    if (dir_fd < 0) {
        perror("open (dir)");
        return 1;
    }

    DIR* dir = fdopendir(dup(dir_fd));
    if (dir == NULL) {
        perror("fdopendir");
        return 1;
    }

    struct dirent* ent;
    while ((ent = readdir(dir))) {
        if (ent->d_name[0] == '.') {
            continue;
        }
        printf("processing %s\n", ent->d_name);

        int fd = openat(dir_fd, ent->d_name, O_RDWR);
        if (fd < 0) {
            perror("openat");
            continue;
        }

        struct stat st;
        if (fstat(fd, &st) < 0) {
            perror("fstat");
            close(fd);
            continue;
        }

        void* ptr = mmap(NULL, st.st_size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
        if (ptr == NULL) {
            perror("mmap");
            close(fd);
            continue;
        }

        process(ptr);

        if (munmap(ptr, st.st_size) < 0) {
            perror("munmap");
            // Keep cleaning up
        }

        if (close(fd) < 0) {
            perror("close");
            // Keep cleaning up
        }
    }

    if (closedir(dir) < 0) {
        perror("closedir");
        // Keep cleaning up
    }

    if (close(dir_fd) < 0) {
        perror("close");
        // Keep cleaning up
    }

    return 0;
}
