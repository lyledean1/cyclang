#include <stdint.h>
#include <time.h>

int64_t unix() {
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    return (int64_t)(ts.tv_sec * 1000) + (ts.tv_nsec / 1000000);
}

void sleep(int32_t ms) {
    if (ms <= 0) {
        return;
    }
    struct timespec ts;
    ts.tv_sec = ms / 1000;
    ts.tv_nsec = (ms % 1000) * 1000000L;
    nanosleep(&ts, 0);
}
