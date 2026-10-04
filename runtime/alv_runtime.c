/*
 * আলভীলান runtime — linked into every compiled program.
 * Prints numbers with Bangla digits so output matches the source language:
 *   দেখাও(১০ + ২০)  →  ৩০
 */
#include <stdio.h>

static const char *const BN_DIGITS[10] = {
    "০", "১", "২", "৩", "৪", "৫", "৬", "৭", "৮", "৯",
};

static void put_bangla_line(const char *s) {
    for (; *s; s++) {
        if (*s >= '0' && *s <= '9')
            fputs(BN_DIGITS[*s - '0'], stdout);
        else
            putchar(*s);
    }
    putchar('\n');
}

void alv_print_int(long long v) {
    char buf[32];
    snprintf(buf, sizeof buf, "%lld", v);
    put_bangla_line(buf);
}

void alv_print_float(double v) {
    char buf[64];
    snprintf(buf, sizeof buf, "%.10g", v);
    put_bangla_line(buf);
}
