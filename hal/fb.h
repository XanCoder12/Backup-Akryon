#ifndef NYXARA_HAL_FB_H
#define NYXARA_HAL_FB_H

#include "types.h"

#define BOOT_INFO_MAGIC 0x414B5259 // 'AKRY'

typedef struct {
    uint32_t magic;
    uint32_t fb_addr;
    uint16_t fb_width;
    uint16_t fb_height;
    uint16_t fb_pitch;
    uint8_t  fb_bpp;
    uint8_t  is_graphical;
    uint8_t  reserved[2];
} __attribute__((packed)) boot_info_t;

void fb_init(boot_info_t* info);
boot_info_t* fb_get_info(void);
uint8_t fb_is_active(void);

#endif // NYXARA_HAL_FB_H
