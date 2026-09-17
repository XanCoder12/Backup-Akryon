#include "fb.h"
#include "serial.h"

static boot_info_t current_fb_info;
static uint8_t fb_active = 0;

void fb_init(boot_info_t* info) {
    if (!info) {
        fb_active = 0;
        return;
    }

    if (info->magic == BOOT_INFO_MAGIC && info->is_graphical) {
        current_fb_info = *info;
        fb_active = 1;
        serial_puts("[Nyxara HAL] VBE Linear Framebuffer initialized: ");
        if (info->fb_width == 1024) {
            serial_puts("1024x768");
        } else if (info->fb_width == 800) {
            serial_puts("800x600");
        } else {
            serial_puts("Custom Res");
        }
        serial_puts(" @ 32bpp\n");
    } else {
        fb_active = 0;
        serial_puts("[Nyxara HAL] VBE not available, fallback to VGA text mode.\n");
    }
}

boot_info_t* fb_get_info(void) {
    return &current_fb_info;
}

uint8_t fb_is_active(void) {
    return fb_active;
}
