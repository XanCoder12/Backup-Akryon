#include "types.h"

void* memset(void* dest, int val, size_t count) {
    uint8_t* dp = (uint8_t*)dest;
    uint8_t v = (uint8_t)val;
    if (count >= 16 && ((uintptr_t)dp & 3) == 0) {
        uint32_t v32 = (uint32_t)v | ((uint32_t)v << 8) | ((uint32_t)v << 16) | ((uint32_t)v << 24);
        uint32_t* dp32 = (uint32_t*)dp;
        size_t words = count / 4;
        for (size_t i = 0; i < words; i++) {
            dp32[i] = v32;
        }
        for (size_t i = words * 4; i < count; i++) {
            dp[i] = v;
        }
        return dest;
    }
    for (size_t i = 0; i < count; i++) {
        dp[i] = v;
    }
    return dest;
}

void* memcpy(void* dest, const void* src, size_t count) {
    uint8_t* dp = (uint8_t*)dest;
    const uint8_t* sp = (const uint8_t*)src;
    if (count >= 16 && (((uintptr_t)dp | (uintptr_t)sp) & 3) == 0) {
        uint32_t* dp32 = (uint32_t*)dp;
        const uint32_t* sp32 = (const uint32_t*)sp;
        size_t words = count / 4;
        for (size_t i = 0; i < words; i++) {
            dp32[i] = sp32[i];
        }
        for (size_t i = words * 4; i < count; i++) {
            dp[i] = sp[i];
        }
        return dest;
    }
    for (size_t i = 0; i < count; i++) {
        dp[i] = sp[i];
    }
    return dest;
}

void* memmove(void* dest, const void* src, size_t count) {
    uint8_t* dp = (uint8_t*)dest;
    const uint8_t* sp = (const uint8_t*)src;
    if (dp < sp) {
        if (count >= 16 && (((uintptr_t)dp | (uintptr_t)sp) & 3) == 0) {
            uint32_t* dp32 = (uint32_t*)dp;
            const uint32_t* sp32 = (const uint32_t*)sp;
            size_t words = count / 4;
            for (size_t i = 0; i < words; i++) {
                dp32[i] = sp32[i];
            }
            for (size_t i = words * 4; i < count; i++) {
                dp[i] = sp[i];
            }
            return dest;
        }
        for (size_t i = 0; i < count; i++) {
            dp[i] = sp[i];
        }
    } else if (dp > sp) {
        if (count >= 16 && (((uintptr_t)dp | (uintptr_t)sp) & 3) == 0) {
            uint32_t* dp32 = (uint32_t*)dp;
            const uint32_t* sp32 = (const uint32_t*)sp;
            size_t words = count / 4;
            for (size_t i = count; i > words * 4; i--) {
                dp[i - 1] = sp[i - 1];
            }
            for (size_t i = words; i > 0; i--) {
                dp32[i - 1] = sp32[i - 1];
            }
            return dest;
        }
        for (size_t i = count; i > 0; i--) {
            dp[i - 1] = sp[i - 1];
        }
    }
    return dest;
}

int memcmp(const void* s1, const void* s2, size_t count) {
    const uint8_t* p1 = (const uint8_t*)s1;
    const uint8_t* p2 = (const uint8_t*)s2;
    for (size_t i = 0; i < count; i++) {
        if (p1[i] != p2[i]) {
            return p1[i] - p2[i];
        }
    }
    return 0;
}

int bcmp(const void* s1, const void* s2, size_t count) {
    return memcmp(s1, s2, count);
}

size_t strlen(const char* str) {
    size_t len = 0;
    if (!str) return 0;
    while (str[len] != '\0') {
        len++;
    }
    return len;
}

int strcmp(const char* s1, const char* s2) {
    if (!s1 || !s2) return -1;
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(const unsigned char*)s1 - *(const unsigned char*)s2;
}

int strncmp(const char* s1, const char* s2, size_t n) {
    if (!s1 || !s2 || n == 0) return 0;
    while (n-- && *s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return (n == (size_t)-1) ? 0 : (*(const unsigned char*)s1 - *(const unsigned char*)s2);
}
