#include "src/cpu/dispatcher/sim_runner.hpp"
#include <filesystem>
#include <fstream>
#include <iterator>
#include <string>

#define CHECK(condition) do { if (!(condition)) return EXIT_FAILURE; } while (0)

extern "C" int packopen_sim_seed_ckpt(int, unsigned long long, int, int,
    const char *, const char *, const char *, double *, packopen_progress_cb,
    void *, unsigned long long);

static void progress(unsigned long long done, unsigned long long, void *ctx) {
    *static_cast<unsigned long long *>(ctx) = done;
}

static std::string read(const std::filesystem::path &path) {
    std::ifstream file(path, std::ios::binary);
    return std::string(std::istreambuf_iterator<char>(file), {});
}

int main() {
    CHECK(cpu_effective_chunks(1000, false, false, 16777216) == 1);
    CHECK(cpu_effective_chunks(1000, false, false, 16777217) == 1000);
    CHECK(cpu_effective_chunks(1000, true, false, 10) == 1000);
    CHECK(cpu_effective_chunks(1000, false, true, 10) == 1000);

    const auto dir = std::filesystem::temp_directory_path() /
        ("packsim-chunks-" + std::to_string(
            std::chrono::steady_clock::now().time_since_epoch().count()));
    CHECK(std::filesystem::create_directory(dir));
    struct Cleanup {
        std::filesystem::path dir;
        ~Cleanup() { std::filesystem::remove_all(dir); }
    } cleanup{dir};
    const auto kept = (dir / "kept").string();
    const auto sold = (dir / "sold").string();
    const auto progress_kept = (dir / "progress-kept").string();
    const auto progress_sold = (dir / "progress-sold").string();
    // Cross the folding threshold on one worker in both the direct and ghost paths.
    const unsigned long long count = 16777217;
    for (int eitem : {0, 17}) {
        double seconds = 0;
        unsigned long long done = 0;
        CHECK(packopen_sim_seed_ckpt(eitem, count, 1, 340, kept.c_str(),
            sold.c_str(), nullptr, &seconds, nullptr, nullptr, 42) == 0);
        CHECK(packopen_sim_seed_ckpt(eitem, count, 1, 340, progress_kept.c_str(),
            progress_sold.c_str(), nullptr, &seconds, progress, &done, 42) == 0);
        CHECK(done == count);
        CHECK(!read(kept).empty());
        CHECK(read(kept) == read(progress_kept));
        CHECK(read(sold) == read(progress_sold));
    }
}
