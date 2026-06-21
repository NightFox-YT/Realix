/* The Realix VFS
    © 2026 Alexander Silaev
*/
#ifndef __realix_vfs__
#define __realix_vfs__

#include "../libc/stdint.h"
#include "../include/slab.h"

struct vfs_node;
struct vfs_file;

#define MAX_MOUNTS 4
#define VFS_MAX_NAME_LEN 32
#define VFS_TYPE_FILE 1
#define VFS_TYPE_DIR 2
#define VFS_TYPE_DEV 3
#define MAX_OPEN_FILES_PER_PROCESS 16

struct vfs_dirent
{
    char name[VFS_MAX_NAME_LEN]; // имя файла в формате FILENAME.EXT
    uint32_t size; // размер в байтах
    uint8_t is_dir; // 1- папка, 0-файл
};

struct vfs_node_ops {
    int (*open)(struct vfs_file *file, const char *path);
    int (*read)(struct vfs_file *file, uint8_t *buf, uint32_t size);
    int (*write)(struct vfs_file *file, const uint8_t *buf, uint32_t size);
    int (*close)(struct vfs_file *file);
    int (*create)(struct vfs_node *node, const char *path);
    int (*readdir)(struct vfs_file *file, struct vfs_dirent *dir, uint32_t index);
};

struct vfs_node {
    char name[VFS_MAX_NAME_LEN]; // имя точки монтирования типа dskX
    uint8_t type;                // тип узла
    struct vfs_node_ops *oprs;   // операции для этого узла
    void *priv_data;             // ссылка на внутренние структуры
    struct vfs_node *next;       // ссылка на следующий смонтированный диск
};

struct vfs_file {
    struct vfs_node *node;  // к какой фс относится файл
    uint32_t offset;        // текущая позиция чтения/записи
    uint32_t size;          // размер
    void *priv_file_data;   // ссылка на конкретный файл
};

extern struct vfs_node *mount_table[MAX_MOUNTS];
extern struct vfs_node *mount_list_head;
extern struct vfs_file* global_fd_table[MAX_OPEN_FILES_PER_PROCESS];

void vfs_init(void);
int vfs_mount(const char *path, struct vfs_node_ops *ops, void *priv_data);
struct vfs_file *vfs_open(const char *path);
int vfs_read(struct vfs_file *file, uint8_t *buf, uint32_t size);
int vfs_write(struct vfs_file *file, const uint8_t *buf, uint32_t size);
int vfs_create(const char *path);
int vfs_close(struct vfs_file *file);
int vfs_readdir(struct vfs_file *file, struct vfs_dirent *dir, uint32_t index);
int alloc_fd(struct vfs_file *file);
struct vfs_file* get_file_by_fd(int fd);
void free_fd(int fd);

#ifdef __RVFS_INTERNAL_FUNCTIONS__
int vfs_get_first_part(const char *path, char *output);
const char* vfs_get_pure_path(const char *path);
struct vfs_node* vfs_find_mount(const char *path);
#endif

#endif /*__realix_vfs__*/