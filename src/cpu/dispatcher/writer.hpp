#pragma once

#include "src/cpu/core/types.hpp"
#include "src/common/packopen_common.hpp"

/**
 * @brief Serializes final simulation histograms to disk in standardized text format.
 * Formatted as: `<exp> <sub_var> <destiny> <foil> <count>`.
 */
static inline void write_results(const char *results_path, const char *sold_path, const uint64_t *kept, const uint64_t *sold) {
    pack_write_results(results_path, sold_path, kept, sold);
}
