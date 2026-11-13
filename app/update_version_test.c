#include "update_version.h"
#include <assert.h>
#include <stddef.h>

int main(void) {
    assert(cmd_kana_compare_versions("v0.10.0", "0.9.3") == 1);
    assert(cmd_kana_compare_versions("0.9.10", "0.9.9") == 1);
    assert(cmd_kana_compare_versions("v1.0.0", "0.99.99") == 1);
    assert(cmd_kana_compare_versions("v0.9.3", "0.9.3") == 0);
    assert(cmd_kana_compare_versions("v0.9.2", "0.9.3") == -1);
    const char *invalid[] = {
        NULL, "", "v", "0.9", "0.9.3.1", "0.9.3-beta.1", "0.9.3+build",
        "release-0.9.3", " 0.9.3", "0.9.3\n", "0.-9.3", "0.09.3",
        "18446744073709551616.0.0"
    };
    for (size_t i = 0; i < sizeof(invalid) / sizeof(invalid[0]); i++) {
        assert(cmd_kana_compare_versions(invalid[i], "0.9.3") == CMD_KANA_VERSION_INVALID);
        assert(cmd_kana_compare_versions("0.9.3", invalid[i]) == CMD_KANA_VERSION_INVALID);
    }
    return 0;
}
