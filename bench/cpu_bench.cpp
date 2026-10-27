#include <iostream>
#include <vector>
#include <chrono>
#include <iomanip>
#include <cmath>
#include <omp.h>

// Include entry to get packopen_sim_seed
#include "src/cpu/entry.cpp"

struct BenchmarkResult {
    int eitem;
    double mean_mpacks;
    double stddev;
};

int main() {
    int num_threads = omp_get_max_threads();
    std::cout << "Starting CPU Benchmark (" << num_threads << " threads)...\n";
    std::cout << "Testing 12 expansion items, 5 iterations each.\n\n";

    // Testing a variety of items to cover different pack geometries
    int items[] = {0, 1, 2, 4, 6, 8, 10, 12, 14, 16, 18, 129};
    int num_items = sizeof(items) / sizeof(items[0]);
    unsigned long long count = 10000000ULL; // 10 million packs per iter
    int iters = 5;

    std::vector<BenchmarkResult> results;

    for (int i = 0; i < num_items; i++) {
        int eitem = items[i];
        std::vector<double> throughputs;

        for (int iter = 0; iter < iters; iter++) {
            double seconds = 0.0;
            // Send output to /dev/null
            const char* results_path = "/dev/null";
#ifdef _WIN32
            results_path = "NUL";
#endif
            const char* sold_path = results_path;

            unsigned long long seed = 0x12345678ULL + iter;
            int rc = packopen_sim_seed(eitem, count, num_threads, 340, results_path, sold_path, &seconds, seed);

            if (rc != 0 || seconds <= 0.0) {
                std::cerr << "Simulation failed for eitem " << eitem << " with rc=" << rc << "\n";
                return 1;
            }

            double mpacks_sec = (count / 1000000.0) / seconds;
            throughputs.push_back(mpacks_sec);
        }

        double sum = 0.0;
        for (double val : throughputs) sum += val;
        double mean = sum / iters;

        double sq_sum = 0.0;
        for (double val : throughputs) sq_sum += (val - mean) * (val - mean);
        double stddev = std::sqrt(sq_sum / iters);

        results.push_back({eitem, mean, stddev});
    }

    std::cout << "| Expansion Item | Mean Throughput (Mpacks/s) | StdDev |\n";
    std::cout << "|---|---|---|\n";
    for (const auto& r : results) {
        std::cout << "| " << r.eitem << " | " << std::fixed << std::setprecision(2) << r.mean_mpacks
                  << " | " << std::fixed << std::setprecision(2) << r.stddev << " |\n";
    }

    return 0;
}
