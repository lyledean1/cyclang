#include <stdint.h>
#include <time.h>

int64_t unix() {
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    return (int64_t)(ts.tv_sec * 1000) + (ts.tv_nsec / 1000000);
}
