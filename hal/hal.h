#ifndef AKRYON_HAL_H
#define AKRYON_HAL_H

#include "io.h"
#include "vga.h"
#include "gdt.h"
#include "idt.h"
#include "isr.h"
#include "timer.h"
#include "keyboard.h"
#include "serial.h"
#include "pci.h"
#include "rtl8139.h"
#include "rtc.h"
#include "fb.h"

void hal_init(boot_info_t* info);

#endif // AKRYON_HAL_H
