#include <stdatomic.h>
#include <stdint.h>
#include <pthread.h>
#include <unistd.h>
#include <errno.h>
#include <sys/socket.h>
#include <linux/netlink.h>
#include <linux/rtnetlink.h>

static _Atomic uint64_t generation = 1;
static _Atomic int ready = 0;
static pthread_once_t once = PTHREAD_ONCE_INIT;
static void *observe(void *arg) {
    int fd = (int)(intptr_t)arg;
    unsigned char buffer[16384];
    for (;;) {
        ssize_t length = recv(fd, buffer, sizeof(buffer), 0);
        if (length < 0 && errno == EINTR) continue;
        if (length <= 0) break;
        // Any link/address/route change invalidates exports before probing again.
        atomic_fetch_add_explicit(&generation, 1, memory_order_release);
    }
    atomic_store_explicit(&ready, 0, memory_order_release);
    atomic_fetch_add_explicit(&generation, 1, memory_order_release);
    close(fd);
    return NULL;
}
static void start(void) {
    int fd = socket(AF_NETLINK, SOCK_RAW | SOCK_CLOEXEC, NETLINK_ROUTE);
    if (fd < 0) return;
    struct sockaddr_nl address = {0};
    address.nl_family = AF_NETLINK;
    address.nl_groups = RTMGRP_LINK | RTMGRP_IPV4_IFADDR | RTMGRP_IPV4_ROUTE | RTMGRP_IPV6_ROUTE;
    if (bind(fd, (struct sockaddr *)&address, sizeof(address)) != 0) { close(fd); return; }
    pthread_t thread;
    atomic_store_explicit(&ready, 1, memory_order_release);
    if (pthread_create(&thread, NULL, observe, (void *)(intptr_t)fd) != 0) {
        atomic_store_explicit(&ready, 0, memory_order_release); close(fd); return;
    }
    pthread_detach(thread);
}
uint64_t localchat_network_generation(void) {
    pthread_once(&once, start);
    return atomic_load_explicit(&ready, memory_order_acquire)
        ? atomic_load_explicit(&generation, memory_order_acquire) : 0;
}
