#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <errno.h>
#include <signal.h>

// Keep in sync with stdlib StringType
typedef struct {
    char *buffer;
    int32_t length;
    int32_t maxlen;
    int32_t factor;
} StringType;

StringType* stringInit(const char *data);

static int32_t last_listen_fd = -1;

int32_t cyc_net_listen(int32_t port) {
    signal(SIGPIPE, SIG_IGN);
    int32_t fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) {
        last_listen_fd = -1;
        return -errno;
    }

    int yes = 1;
    setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(yes));
#ifdef SO_NOSIGPIPE
    setsockopt(fd, SOL_SOCKET, SO_NOSIGPIPE, &yes, sizeof(yes));
#endif

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_ANY);
    addr.sin_port = htons((uint16_t)port);

    if (bind(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        close(fd);
        last_listen_fd = -1;
        return -errno;
    }

    if (listen(fd, 16) < 0) {
        close(fd);
        last_listen_fd = -1;
        return -errno;
    }

    last_listen_fd = fd;
    return fd;
}

int32_t cyc_net_accept(int32_t server_fd) {
    int32_t client = accept(server_fd, NULL, NULL);
#ifdef SO_NOSIGPIPE
    if (client >= 0) {
        int yes = 1;
        setsockopt(client, SOL_SOCKET, SO_NOSIGPIPE, &yes, sizeof(yes));
    }
#endif
    if (client < 0) {
        return -errno;
    }
    return client;
}

StringType* cyc_net_read(int32_t fd, int32_t max_bytes) {
    if (max_bytes <= 0) {
        return stringInit("");
    }

    char *buffer = (char *)malloc((size_t)max_bytes + 1);
    if (!buffer) {
        return stringInit("");
    }

    int flags = 0;
#ifdef MSG_DONTWAIT
    flags = MSG_DONTWAIT;
#endif
    ssize_t received = recv(fd, buffer, (size_t)max_bytes, flags);
    if (received <= 0) {
        free(buffer);
        return stringInit("");
    }

    buffer[received] = '\0';
    StringType *result = stringInit(buffer);
    free(buffer);
    return result;
}

int32_t cyc_net_write(int32_t fd, const StringType *data) {
    if (!data || !data->buffer || data->length <= 0) {
        return -1;
    }

    int32_t total_sent = 0;
    while (total_sent < data->length) {
        ssize_t sent = send(
            fd,
            data->buffer + total_sent,
            (size_t)(data->length - total_sent),
            0
        );
        if (sent <= 0) {
            break;
        }
        total_sent += (int32_t)sent;
    }

    if (total_sent > 0) {
        return total_sent;
    }

    return -errno;
}

void cyc_net_close(int32_t fd) {
    if (fd >= 0) {
        close(fd);
    }
}

int32_t cyc_net_errno() {
    return (int32_t)errno;
}

int32_t cyc_net_last_listen_fd() {
    return last_listen_fd;
}
