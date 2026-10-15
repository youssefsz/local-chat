// Inject failures at the OS poll boundary, using the real Bonjour adapter.
#include <assert.h>
#include <errno.h>
#include <poll.h>
#include <stdatomic.h>
#include <stdio.h>
#include <unistd.h>

static _Atomic int fault, injected, reported;
static int failing_poll(struct pollfd *fds, nfds_t count, int timeout) {
    (void)count; (void)timeout;
    atomic_fetch_add(&injected, 1);
    int mode = atomic_load(&fault);
    if (mode == EINTR) {
        atomic_store(&fault, POLLHUP);
        errno = EINTR;
        return -1;
    }
    if (mode == EIO) { errno = EIO; return -1; }
    fds[0].revents = (short)mode;
    return 1;
}
#define poll failing_poll
#include "../src-tauri/src/platform/bonjour.c"
#undef poll

static void observed(void *user, int kind, const char *name, const char *peer,
                     const char *cert, const char *ip, unsigned short port) {
    (void)user; (void)name; (void)peer; (void)cert; (void)ip; (void)port;
    if (kind == 2) atomic_fetch_add(&reported, 1);
}

int main(int argc, char **argv) {
    assert(argc == 2);
    const int cases[] = {POLLHUP, POLLERR, POLLNVAL, EIO, EINTR};
    int failed = 0;
    for (size_t i = 0; i < sizeof(cases)/sizeof(cases[0]); i++) {
        atomic_store(&fault, cases[i]);
        atomic_store(&injected, 0);
        atomic_store(&reported, 0);
        void *handle = localchat_bonjour_start(argv[1], "lc-recovery-test",
            "_lct-poll._tcp", 12345, NULL, 0, observed, NULL);
        assert(handle);
        for (int tries = 0; tries < 100 && !atomic_load(&reported); tries++) usleep(10000);
        localchat_bonjour_stop(handle);
        int passed = atomic_load(&reported) == 1 && atomic_load(&injected) > 0;
        if (cases[i] == EINTR) passed &= atomic_load(&injected) >= 2;
        printf("Bonjour poll fault %d: %s\n", cases[i], passed ? "PASS" : "FAIL: adapter stopped responding without notifying recovery");
        failed |= !passed;
    }
    return failed;
}
