#ifndef CMD_KANA_UPDATE_VERSION_H
#define CMD_KANA_UPDATE_VERSION_H

#include <stdbool.h>
#include <stdint.h>

enum { CMD_KANA_VERSION_INVALID = 2 };

/* Stable release tags use vMAJOR.MINOR.PATCH or MAJOR.MINOR.PATCH. */
static inline bool cmd_kana_parse_version(const char *text, uint64_t parts[3]) {
    if (!text) return false;
    if (*text == 'v') text++;
    for (int i = 0; i < 3; i++) {
        if (*text < '0' || *text > '9') return false;
        if (*text == '0' && text[1] >= '0' && text[1] <= '9') return false;
        uint64_t value = 0;
        while (*text >= '0' && *text <= '9') {
            unsigned digit = (unsigned)(*text++ - '0');
            if (value > (UINT64_MAX - digit) / 10) return false;
            value = value * 10 + digit;
        }
        parts[i] = value;
        if (i < 2) {
            if (*text++ != '.') return false;
        } else if (*text != '\0') {
            return false;
        }
    }
    return true;
}

/* Returns -1, 0, 1, or CMD_KANA_VERSION_INVALID. */
static inline int cmd_kana_compare_versions(const char *candidate, const char *current) {
    uint64_t candidate_parts[3], current_parts[3];
    if (!cmd_kana_parse_version(candidate, candidate_parts) ||
        !cmd_kana_parse_version(current, current_parts)) return CMD_KANA_VERSION_INVALID;
    for (int i = 0; i < 3; i++) {
        if (candidate_parts[i] > current_parts[i]) return 1;
        if (candidate_parts[i] < current_parts[i]) return -1;
    }
    return 0;
}

#endif
